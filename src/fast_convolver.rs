use std::mem::size_of;

use crate::ring_buffer::RingBuffer;
//use realfft::

struct FastConvolver {
    ir: Vec<f32>,
    tail: RingBuffer<f32>,
    conv_mode: ConvolutionMode,
    flush_me: bool,
    input_len: usize,
    //input_fft: 
}

#[derive(Debug, Clone, Copy)]
pub enum ConvolutionMode {
    TimeDomain,
    FrequencyDomain { block_size: usize },
}

impl FastConvolver {
    pub fn new(impulse_response: &[f32], mode: ConvolutionMode) -> Self {
        let buffer = RingBuffer::<f32>::new(impulse_response.len()-1);
        FastConvolver{
            ir: impulse_response.to_vec(),
            tail: buffer,
            conv_mode: mode,
            flush_me: false,
            input_len: 0,
        }
    }

    pub fn reset(&mut self) {
        self.tail.reset();
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        let block_size = input.len();
        self.input_len = block_size;
        match self.conv_mode{
            ConvolutionMode::TimeDomain =>{self.conv_time_domain(input, output);},
            ConvolutionMode::FrequencyDomain { block_size }=>{

            }
        }
    }

    pub fn flush(&mut self, output: &mut [f32]) {
        // Flushes remaining samples into the output
        for i in 0..self.ir.len()-1{
            output[i] = self.tail.get(i);
        }

    }

    pub fn get_output_size(&mut self, input_size: usize)->usize{
        input_size + self.ir.len() -1
    }
    fn conv_time_domain(&mut self, input: &[f32], output: &mut [f32]){
        let mut complete_buffer = vec![0.0;input.len()+self.ir.len() -1];
        for i in 0..input.len(){
            for j in 0..self.ir.len(){
                complete_buffer[i+j] += input[i] * self.ir[j];
            }
        } 
        for k in 0..self.ir.len(){
            complete_buffer[k] += self.tail.get(k);
        }
        output.copy_from_slice(&complete_buffer[0..output.len()]);
        self.tail.set_write_index(0);
        for m in input.len()..complete_buffer.len(){
            self.tail.push(complete_buffer[m]);
        }
    }

}

#[cfg(test)]
mod tests {
    use std::{borrow::BorrowMut, iter::zip};

    use super::*;
    use rand::Rng;
    use criterion::{black_box,criterion_group,criterion_main,Criterion};
    #[test]
    fn identity_time_domain(){
        let mut rng = rand::thread_rng();
        let mut ir = vec![0.0;51];
        let input = [0.0,0.0,0.0,1.0,0.0,0.0,0.0,0.0,0.0,0.0];
        for value in ir.iter_mut(){
            *value = (rng.gen_range(-10.0..10.0) as f32).trunc();
        }
        let mut conv = FastConvolver::new(&ir, ConvolutionMode::TimeDomain);
        let mut output = vec![0.0;conv.get_output_size(input.len())];
        let now = std::time::Instant::now();
        conv.process(&input, &mut output[0..input.len()]);
        let elapsed = now.elapsed();
        println!("{:.2?}: time elapsed",elapsed);
        for i in 0..input.len(){
            if i > 3{
                assert_eq!(output[i],ir[i-3]);
            }
        }
        /* let now = std::time::Instant::now();
        conv.process(&input, &mut output);
        let elapsed = now.elapsed();
        println!("{:.2?}: time elapsed",elapsed); */
        /* criterion_group!();
        criterion_main!(); */
    }
    #[test]
    fn flush_time_domain(){
        let mut rng = rand::thread_rng();
        let mut ir = vec![0.0;51];
        let input = [0.0,0.0,0.0,1.0,0.0,0.0,0.0,0.0,0.0,0.0];
        for value in ir.iter_mut(){
            *value = (rng.gen_range(-10.0..10.0) as f32).trunc();
        }
        let mut conv = FastConvolver::new(&ir, ConvolutionMode::TimeDomain);
        let mut output = vec![0.0;conv.get_output_size(input.len())];
        let now = std::time::Instant::now();
        conv.process(&input, &mut output[0..input.len()]);
        conv.flush(&mut output[input.len()..]);
        let elapsed = now.elapsed();
        println!("{:.2?}: time elapsed",elapsed);
        for i in 0..output.len(){
            if i > 3 && i < ir.len()+3{
                assert_eq!(output[i],ir[i-3]);
            }
        }
    }
    #[test]
    fn block_size_time_domain(){
        let mut rng = rand::thread_rng();
        let mut ir = vec![0.0,0.0,0.0,0.0,0.0,0.0,1.0]; // delay of 6
        let mut input: Vec<f32> = vec![0.0;10000];
        for value in input.iter_mut(){
            *value = (rng.gen_range(-10.0..10.0) as f32).trunc();
        }
        let mut conv = FastConvolver::new(&ir, ConvolutionMode::TimeDomain);
        let mut output: Vec<f32> = vec![0.0;conv.get_output_size(input.len())];
        let block_sizes = [1, 13, 1023, 2048,30,17, 5000, 1897];
        for &block_size in block_sizes.iter(){
            let new_output =  vec![0.0;conv.get_output_size(input.len())];
            output.copy_from_slice(&new_output);
            let input_chunks = input.chunks(block_size);
            let mut output_chunks = output[0..input.len()].chunks_mut(block_size);
            for (in_chunk,out_chunk) in zip(input_chunks, output_chunks){
                conv.process(in_chunk, out_chunk);
            }
            conv.flush(&mut output[input.len()..]);
            println!("Block size: {}", block_size);
            for i in 0..output.len(){
                dbg!(i);
                if i > 6 && i < input.len()+6{
                    assert_eq!(output[i],input[i-6])
                }
            }
        }
    }
}

//cargo flamegraph -- <parameters>
//google-chrome flamegraph.svg
// cargo flamegraph -f -- <parameters>  //adds a lot more samples
// cargo run 

// godbolt.org
