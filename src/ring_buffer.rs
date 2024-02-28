use std::borrow::Borrow;

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
        self.buffer[self.write_ptr] = value;
    }

    pub fn peek(&self) -> T {
        //todo!()
        self.buffer[self.read_ptr]
    }

    pub fn get(&self, offset: usize) -> T {
        //todo!()
        let safe_offset = offset % self.buffer.capacity();
        self.buffer.get(safe_offset).copied().unwrap_or_default()
    }

    // `push` and `pop` write/read and advance the indices.
    pub fn push(&mut self, value: T) {
        //todo!()
        self.put(value);
        self.write_ptr = (self.write_ptr + 1) % self.capacity();
    }

    pub fn pop(&mut self) -> T {
        //todo!()
        let val = self.peek();
        self.read_ptr = (self.read_ptr + 1) % self.capacity();
        val

    }

    pub fn get_read_index(&self) -> usize {
        //todo!()
        self.read_ptr
    }

    pub fn set_read_index(&mut self, index: usize) {
        //todo!()
        self.read_ptr = index % self.capacity();
    }

    pub fn get_write_index(&self) -> usize {
        //todo!()
        self.write_ptr
    }

    pub fn set_write_index(&mut self, index: usize) {
        //todo!()
        self.write_ptr = index % self.capacity();
    }

    pub fn len(&self) -> usize {
        // Return number of values currently in the buffer.
        //todo!()
        if self.write_ptr >= self.read_ptr {
            self.write_ptr - self.read_ptr
        } else {
            // Handle the case where write pointer has wrapped around
            self.capacity() - (self.read_ptr - self.write_ptr)
        }
    }

    pub fn capacity(&self) -> usize {
        // Return the length of the internal buffer.
        //todo!()
        self.buffer.len()
    }
}
impl RingBuffer<f32>{
    // returns a value at a a non-integer offset for fractional delays
    pub fn get_frac(&self, offset: f32)->f32{
        let floor = offset.trunc();
        let floor_samp = self.get(floor as usize);
        let ceil_samp = self.get(floor as usize + 1);
        let frac = offset - floor;
        floor_samp * (1.0 - frac) + ceil_samp * frac
    }
    // meant to be used similarly to pop, simply put in a offset and it will calculate the 
    // read pointer's position based on the write pointer
    pub fn pop_frac(& self, offset: f32)->f32{
        let mut read_int = self.write_ptr as i32 - offset.trunc() as i32;
        let mut read_point = 0;
        if read_int < 0 {
            read_int += self.capacity() as i32;
            read_point = read_int as usize;
        }else{
            read_point = read_int as usize;
        }
        
        let floor_samp = self.get(read_point);
        let ceil_samp = self.get(read_point + 1_usize);
        let frac = offset - offset.floor();
        floor_samp * (1.0 - frac) + ceil_samp * frac
    }
}

#[cfg(test)]
mod tests {
    use super::*;   
    #[test]
    // Tests basic push, pop, and put functionality
    fn test1 ()
    {
        let mut buffer = RingBuffer::<f32>::new(5);
        buffer.reset();
        assert_eq!(buffer.capacity(), 5);
        buffer.push(0.1);
        assert_eq!(buffer.peek(), 0.1);
        buffer.put(0.3);
        buffer.set_read_index(1);
        assert_eq!(buffer.pop(), 0.3);
        buffer.push(0.7);
        buffer.set_read_index(1);
        assert_eq!(buffer.pop(),0.7);
        println!("Test 1 Passed!");
    }
    #[test]
    // Tests getters and puahing and popping an incrementing value
    fn test2 ()
    {
        let mut buffer = RingBuffer::<f32>::new(5);
        buffer.reset();
        for i in 0..4
        {
            let value = i as f32 * 0.1;
            buffer.push(value);
            assert_eq!(buffer.get(i), value);
            assert_eq!(buffer.pop(), value);
            assert_eq!(buffer.get_read_index(), i+1);
            assert_eq!(buffer.get_write_index(), i+1);
        }
        assert_eq!(buffer.len(),0);
        println!("Test 2 passed!");
    }
    #[test]
    // Tests setters for values in and out of range
    fn test3 ()
    {
        let mut buffer = RingBuffer::<f32>::new(5);
        buffer.reset();
        let has_been = false;
        for i in 0..buffer.len()
        {
            buffer.set_read_index(i+3);
            buffer.set_write_index(i+4);
            buffer.put(0.1);
            if i > 0{
                assert_eq!(buffer.peek(),0.1);
            } else {
                assert_eq!(buffer.peek(), 0.0);
            }
            if i==4{
                assert_eq!(buffer.get(i),0.1);
            } else {
                assert_eq!(buffer.get(i),0.0);
            }
        }
        println!("Test 3 passed!");
    }
    #[test]
    // Tests pushing with set + get for read and write index with int buffer and delay
    fn test4 ()
    {
        let mut buffer = RingBuffer::<i32>::new(10);
        for i in 0..10
        {
            buffer.push(i);
            if i ==0{
                assert_eq!(buffer.peek(),i);
            }else if i < 5{
                assert_eq!(buffer.peek(),Default::default());
            }
            else{
                assert_eq!(buffer.peek(), i-5);
            }
            buffer.set_read_index(buffer.get_write_index() as usize +5);
        }   
        println!("Test 4 passed!");
    }
    #[test]
    // Tests manual index setting and putting and peeking with int buffer
    fn test5 ()
    {
        let mut buffer = RingBuffer::<i32>::new(10);
        buffer.reset();
        for i in 0..10
        {
            buffer.set_write_index(i+500);
            buffer.put(i as i32 +500);
            assert_eq!(buffer.get_write_index(), (i + 500) % buffer.capacity());
            buffer.set_read_index(buffer.get_write_index() as usize);
            assert_eq!(buffer.peek(), i as i32 +500);
        }
        println!("Test 5 passed!");
    }

}

