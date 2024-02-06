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

    pub fn get_write_index(&self) -> T {
        todo!()
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

pub fn run_tests ()
{
    let mut buff = RingBuffer::<f32>::new(5);
    let mut int_buff = RingBuffer::<i32>::new(10);

    test1(&mut buff);
    test2(&mut buff);
    test3(&mut buff);
    test4(&mut int_buff);
    test5(&mut int_buff);
}

// Tests basic push, pop, and put functionality
fn test1 (buffer: &mut RingBuffer<f32>)
{
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
// Tests getters and puahing and popping an incrementing value
fn test2 (buffer: &mut RingBuffer<f32>)
{
    buffer.reset();
    for i in 0..5
    {
        buffer.push(i as f32 * 0.1);
        dbg!(buffer.get(i));
        dbg!(buffer.pop());
        dbg!(buffer.get_read_index());
        dbg!(buffer.get_write_index());
    }
    dbg!(buffer.len());
}

// Tests setters for values in and out of range
fn test3 (buffer: &mut RingBuffer<f32>)
{
    buffer.reset();
    for i in 0..buffer.len()
    {
        buffer.set_read_index(i+3);
        buffer.set_write_index(i+4);
        buffer.put(0.1);
        dbg!(buffer.peek());
        dbg!(buffer.get(i));
    }
}
// Tests pushing with set + get for read and write index with int buffer
fn test4 (buffer: &mut RingBuffer<i32>)
{
    for i in 0..10
    {
        buffer.push(i);
        buffer.set_read_index(buffer.get_write_index()+5);
        dbg!(buffer.peek());
    }   
}
// Tests manual index setting and putting and peeking with int buffer
fn test5 (buffer: &mut RingBuffer<i32>)
{
    buffer.reset();
    for i in 0..10
    {
        buffer.set_write_index(i+500);
        buffer.put(i as i32 +500);
        dbg!(buffer.get_write_index());
        buffer.set_read_index(buffer.get_write_index());
        dbg!(buffer.peek());
    }
}
