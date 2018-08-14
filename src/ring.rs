/// A FIFO buffer with fixed length
///
/// Example:
///
/// ```rust
/// use j2ds::*;
///
/// let mut rb = RingBuffer::new(100, 0u8);
/// rb.push_back(1);
/// rb.push_back_slice(&[2, 3]);
/// // ...
/// let mut buf = [0u8; 3];
/// rb.pop_front_slice(&mut buf);
/// assert_eq!(buf, [1, 2, 3]);
/// ```
pub struct RingBuffer<T: Clone> {
    buffer: Box<[T]>,
    read: usize,
    write: usize,
}

impl<T: Clone> RingBuffer<T> {
    /// Create a new ring buffer that can hold up to `size` elements
    /// and use `value` as the default value
    pub fn new(size: usize, value: T) -> RingBuffer<T> {
        // We waste one element in exchange for faster code that
        // doesn't need to handle the the queue being completely full
        let mut tmp_buf = Vec::with_capacity(size + 1);
        tmp_buf.resize(size + 1, value);

        RingBuffer {
            buffer: tmp_buf.into_boxed_slice(),
            read: 0,
            write: 0,
        }
    }

    /// Add `value` to the end of the queue. Returns false if there is
    /// not enough room in the queue
    pub fn push_back(&mut self, value: T) -> bool {
        let next_write = self.advance_index(self.write, 1);
        if self.capacity() == 0 {
            false
        } else {
            self.buffer[self.write] = value;
            self.write = next_write;
            true
        }
    }

    /// Remove the first value from the queue, or returns `None` if
    /// there are no values in the buffer
    pub fn pop_front(&mut self) -> Option<T> {
        if self.len() == 0 {
            None
        } else {
            let old_read = self.read;
            self.read = self.advance_index(self.read, 1);
            Some(self.buffer[old_read].clone())
        }
    }

    /// Copy the first value from the queue but does not remove it;
    /// returns `None` if there are no values in the buffer
    pub fn peek_front(&self) -> Option<T> {
        if self.read == self.write {
            None
        } else {
            Some(self.buffer[self.read].clone())
        }
    }

    /// Add all `values` to the buffer. If there is not enough room in
    /// the queue then no values are added and the return value is
    /// false
    pub fn push_back_slice(&mut self, values: &[T]) -> bool {
        if self.capacity() < values.len() {
            false
        } else {
            for v in values.iter() {
                let r = self.push_back(v.clone());
                assert!(r);
            }
            true
        }
    }

    /// Remove enough values from the buffer to fill the given
    /// slice. If there are not enough values in the queue then the
    /// output buffer is not modified and the function returns false
    pub fn pop_front_slice(&mut self, values: &mut [T]) -> bool {
        if self.len() < values.len() {
            false
        } else {
            for v in values.iter_mut() {
                *v = self.pop_front().unwrap();
            }
            true
        }
    }

    #[inline]
    fn advance_index(&self, index: usize, amount: isize) -> usize {
        assert!((amount.abs() as usize) < self.buffer.len());
        (index as isize + amount) as usize % self.buffer.len()
    }

    /// Returns the number of values in the buffer
    #[inline]
    pub fn len(&self) -> usize {
        self.max_len() - self.capacity()
    }

    /// Returns the number of free slots in the buffer
    #[inline]
    pub fn capacity(&self) -> usize {
        if self.read > self.write {
            self.read - self.write - 1
        } else {
            self.max_len() - (self.write - self.read)
        }
    }

    /// Returns the max number of values that can ever be stored in
    /// the buffer
    #[inline]
    pub fn max_len(&self) -> usize {
        self.buffer.len() - 1
    }
}

#[test]
fn test_singles() {
    let mut rb = RingBuffer::new(5, 0u8);
    assert_eq!(rb.pop_front(), None);
    assert!(rb.push_back(1));
    assert!(rb.push_back(2));
    assert!(rb.push_back(3));
    assert!(rb.push_back(4));
    assert!(rb.push_back(5));
    assert!(!rb.push_back(6));

    assert_eq!(rb.pop_front(), Some(1));
    assert_eq!(rb.pop_front(), Some(2));
    assert_eq!(rb.peek_front(), Some(3));
    assert_eq!(rb.pop_front(), Some(3));
    assert_eq!(rb.pop_front(), Some(4));
    assert_eq!(rb.pop_front(), Some(5));
    assert_eq!(rb.pop_front(), None);

    assert!(rb.push_back(7));
    assert!(rb.push_back(8));
    assert!(rb.push_back(9));
    assert_eq!(rb.pop_front(), Some(7));
    assert_eq!(rb.pop_front(), Some(8));
    assert_eq!(rb.pop_front(), Some(9));
    assert_eq!(rb.pop_front(), None);
}

#[test]
fn test_slices() {
    let mut rb = RingBuffer::new(5, 0u8);

    let mut buf1 = [0u8; 1];
    let mut buf2 = [0u8; 2];
    let mut buf3 = [0u8; 3];

    assert!(!rb.pop_front_slice(&mut buf2));
    assert!(rb.push_back_slice(&[1, 2, 3]));
    assert!(!rb.push_back_slice(&[4, 5, 6]));
    assert!(rb.pop_front_slice(&mut buf2));
    assert_eq!(buf2, [1, 2]);
    assert!(rb.push_back_slice(&[7, 8]));
    assert!(rb.pop_front_slice(&mut buf2));
    assert_eq!(buf2, [3, 7]);
    assert!(!rb.pop_front_slice(&mut buf2));
    assert!(rb.pop_front_slice(&mut buf1));
    assert_eq!(buf1, [8]);

    for i in 0..100 {
        if i % 2 == 0 {
            assert!(rb.push_back_slice(&[1, 2, 3]));
            assert!(rb.pop_front_slice(&mut buf2));

            assert_eq!(rb.len(), 1);
            assert_eq!(rb.capacity(), 4);
        } else {
            assert!(rb.push_back_slice(&[4, 5]));
            assert!(rb.pop_front_slice(&mut buf3));
            assert_eq!(rb.len(), 0);
            assert_eq!(rb.capacity(), 5);
        }
    }
}

/// A FIFO buffer with a fixed length that adjusts to requests that
/// would otherwise overflow or underflow.
///
/// When an `ElasticRingBuffer` doesn't have enough elements to
/// satisfy a request, it will "stretch" the values it does have by
/// repeating them to fill the request.
///
/// And when the buffer is getting too full (past its ideal max
/// length), elements will be uniformly dropped to return the queue to
/// its ideal length.
pub struct ElasticRingBuffer<T: Clone> {
    rb: RingBuffer<T>,
    ideal_max: usize,
    default_value: T,
}

/// Indicates what happened when the queue tried to satisfy the
/// request for elements
pub enum ElasticPopResult {
    /// The buffer is completely empty, and so the default value is
    /// used
    Empty,
    /// The buffer had enough elements to satisfy the request, but is
    /// still below the ideal max length threshold; and so no elements
    /// were dropped or duplicated
    Exact,
    /// The buffer had some elements, but not enough to satisfy the
    /// request; some elements were repeated to fill the request; the
    /// value is how many "real" elements were present in the buffer
    Upsampled(usize),
    /// The buffer had more elements than the ideal max; some elements
    /// were dropped while filling the request; the value is how many
    /// "real" elements were removed from the queue
    Downsampled(usize),
}

impl<T: Clone> ElasticRingBuffer<T> {
    /// Create a new `ElasticRingBuffer` with the given size. `value`
    /// will be used as the default value for the
    /// queue. `ideal_max_len` is the threshold where the buffer will
    /// begin dropping elements during requests
    pub fn new(size: usize, value: T, ideal_max_len: usize) -> ElasticRingBuffer<T> {
        ElasticRingBuffer {
            rb: RingBuffer::new(size, value.clone()),
            default_value: value,
            ideal_max: ideal_max_len,
        }
    }

    /// Fill `values` with elements. See `ElasticPopResult` for the
    /// possible outcomes of this request.
    pub fn pop_front_slice(&mut self, values: &mut [T]) -> ElasticPopResult {
        let buffer_len = self.rb.len();
        let values_len = values.len();
        if values_len <= buffer_len {
            if buffer_len - values_len < self.ideal_max {
                let r = self.rb.pop_front_slice(values);
                assert!(r);
                ElasticPopResult::Exact
            } else {
                let total_sample_size = (buffer_len - self.ideal_max) + values_len;
                self.sample_n(values, total_sample_size)
            }
        } else {
            self.sample_n(values, buffer_len)
        }
    }

    fn sample_n(&mut self, values: &mut [T], n: usize) -> ElasticPopResult {
        if n == 0 {
            for i in values.iter_mut() {
                *i = self.default_value.clone();
            }
            ElasticPopResult::Empty
        } else {
            let values_len = values.len();
            for (index, i) in values.iter_mut().enumerate() {
                let peek_index = self
                    .rb
                    .advance_index(self.rb.read, (index * n / values_len) as isize);
                *i = self.rb.buffer[peek_index].clone();
            }

            self.rb.read = self.rb.advance_index(self.rb.read, n as isize);

            if values_len > n {
                ElasticPopResult::Upsampled(n)
            } else {
                ElasticPopResult::Downsampled(n)
            }
        }
    }

    /// Add all `values` to the buffer. If there is not enough room in
    /// the queue then no values are added and the return value is
    /// false
    pub fn push_back_slice(&mut self, values: &[T]) -> bool {
        self.rb.push_back_slice(values)
    }

    /// Add `value` to the end of the queue. Returns false if there is
    /// not enough room in the queue
    pub fn push_back(&mut self, value: T) -> bool {
        self.rb.push_back(value)
    }

    /// Returns the number of values in the buffer
    pub fn len(&self) -> usize {
        self.rb.len()
    }

    /// Returns the number of free slots in the buffer
    pub fn capacity(&self) -> usize {
        self.rb.capacity()
    }

    /// Returns the max number of values that can ever be stored in
    /// the buffer
    pub fn max_len(&self) -> usize {
        self.rb.max_len()
    }
}

#[test]
fn test_elastic_exact() {
    let mut erb = ElasticRingBuffer::new(5, 0u8, 3);

    erb.push_back_slice(&[1, 2, 3, 4]);

    let mut buf4 = [0; 4];
    erb.pop_front_slice(&mut buf4);
    assert_eq!(buf4, [1, 2, 3, 4]);
}

#[test]
fn test_elastic_empty() {
    let mut erb = ElasticRingBuffer::new(5, 0u8, 3);

    erb.push_back_slice(&[1, 2, 3, 4]);
    let mut buf4 = [0; 4];
    erb.pop_front_slice(&mut buf4);

    erb.pop_front_slice(&mut buf4);
    assert_eq!(buf4, [0, 0, 0, 0]);
}

#[test]
fn test_elastic_upscale() {
    let mut erb = ElasticRingBuffer::new(5, 0u8, 3);

    erb.push_back_slice(&[1, 2]);
    let mut buf4 = [0; 4];
    erb.pop_front_slice(&mut buf4);

    assert_eq!(buf4, [1, 1, 2, 2]);
}

#[test]
fn test_elastic_downscale() {
    let mut erb = ElasticRingBuffer::new(20, 0u8, 8);

    erb.push_back_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
    let mut buf4 = [0; 4];
    erb.pop_front_slice(&mut buf4);

    assert_eq!(buf4, [1, 3, 5, 7]);
    assert!(erb.len() <= erb.ideal_max);
}
