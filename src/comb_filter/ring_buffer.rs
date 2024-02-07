pub struct RingBuffer<T> {
    // TODO: fill this in.
    buffer :  Vec<T>,
    read_ptr : usize,
    write_ptr : usize
}

impl<T: Copy + Default> RingBuffer<T> {
    pub fn new(length: usize) -> Self {
        // Create a new RingBuffer with `length` slots and "default" values.
        // Hint: look into `vec!` and the `Default` trait.
        //todo!();
        RingBuffer::<T>{buffer: vec![T::default(); length],
                        read_ptr: 0,
                        write_ptr: 0  }
    }

    pub fn reset(&mut self) {
        // Clear internal buffer and reset indices.
        //todo!()
        for value in self.buffer.iter_mut() {
            *value = T::default();
        }
        self.read_ptr = 0;
        self.write_ptr = 0;
    }

    // `put` and `peek` write/read without advancing the indices.
    pub fn put(&mut self, value: T) {
        //todo!()
        if let Some(slot) = self.buffer.get_mut(self.write_ptr) {
            *slot = value;
        }
    }

    pub fn peek(&self) -> T {
        //todo!()
        self.buffer.get(self.read_ptr).copied().unwrap_or_default()
    }

    pub fn get(&self, offset: usize) -> T {
        //todo!()
        self.buffer.get(offset).copied().unwrap_or_default()
    }

    // `push` and `pop` write/read and advance the indices.
    pub fn push(&mut self, value: T) {
        //todo!()
        self.put(value);
        self.write_ptr = (self.write_ptr + 1) % self.buffer.len();
    }

    pub fn pop(&mut self) -> T {
        //todo!()
        let val = self.peek();
        self.read_ptr = (self.read_ptr + 1) % self.buffer.len();
        val

    }

    pub fn get_read_index(&self) -> usize {
        //todo!()
        self.read_ptr
    }

    pub fn set_read_index(&mut self, index: usize) {
        //todo!()
        self.read_ptr = index % self.buffer.len();
    }

    pub fn get_write_index(&self) -> usize {
        //todo!()
        self.write_ptr
    }

    pub fn set_write_index(&mut self, index: usize) {
        //todo!()
        self.write_ptr = index % self.buffer.len();
    }

    pub fn len(&self) -> usize {
        // Return number of values currently in the buffer.
        //todo!()
        self.buffer.len()
    }

    pub fn capacity(&self) -> usize {
        // Return the length of the internal buffer.
        //todo!()
        self.buffer.capacity()
    }
}