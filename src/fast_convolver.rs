//use core::slice::SlicePattern;
//use core::slice::SlicePattern;
use std::{mem::size_of, process::id};

use crate::ring_buffer::RingBuffer;
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};
use rustfft::{num_complex::{Complex, Complex32}, num_traits::PrimInt, Fft, FftPlanner};

/// A time-domain and frequency-domain convolver
pub struct FastConvolver {
    ir: Vec<f32>,
    tail: RingBuffer<f32>,
    conv_mode: ConvolutionMode,
    fft: Option<std::sync::Arc<dyn Fft<f32>>>,
    ifft: Option<std::sync::Arc<dyn Fft<f32>>>,
    baked_ir: Option<Vec<Vec<Complex32>>>,
    fft_length: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
pub enum ConvolutionMode {
    TimeDomain, 
    FrequencyDomain { block_size: usize },
}

impl FastConvolver {
    /// Creates a new FastConvolver Object.
    /// 
    /// Block Size for FrequencyDomain should be the size of the slices into the proces function
    pub fn new(impulse_response: &[f32], mode: ConvolutionMode) -> Self {
        let buffer = RingBuffer::<f32>::new(impulse_response.len()-1);
        let mut fft = None;
        let mut size = None;
        let mut baked = None;
        let ifft = match mode {
            ConvolutionMode::TimeDomain =>{
                None
            },
            ConvolutionMode::FrequencyDomain { block_size }=>{
                let mut plan = FftPlanner::<f32>::new();
                size = Some(block_size*2);           
                fft = Some(plan.plan_fft_forward(size.unwrap()));
                baked = Some(FastConvolver::bake_ir(impulse_response, block_size));
                Some(plan.plan_fft_inverse(size.unwrap()))
            }
        };
        FastConvolver{
            ir: impulse_response.to_vec(),
            tail: buffer,
            conv_mode: mode,
            fft: fft,
            ifft: ifft,
            baked_ir: baked,
            fft_length: size,
        }

    }
    /// Takes the impulse response, splits it into chunks, takes an fft of each chunk,
    /// and returns a vec of those fft chunks
    fn bake_ir(impulse_response: &[f32], block_size: usize)-> Vec<Vec<Complex32>>{
        let mut bake = vec![vec![]];
        let mut chunks_vec= vec![vec![0.0]];
        let chunks = impulse_response.chunks(block_size);
        chunks.clone().for_each(|chunk|{chunks_vec.push(chunk.to_vec())});
        
        let mut plan = FftPlanner::<f32>::new();
        for chunk_id in 1..chunks_vec.len() {
            let mut complex_values = Vec::with_capacity(block_size * 2);
            for &value in &chunks_vec[chunk_id] {
                complex_values.push(Complex::new(value, 0.0));
            }
            let mut padded = vec![Complex::<f32>::new(0.0,0.0); block_size * 2];
            padded[..chunks_vec[chunk_id].len()].copy_from_slice(&complex_values);
            let fft = plan.plan_fft_forward(block_size * 2);
            fft.process(&mut padded);
            bake.push(padded);
        }
        //println!("before everything: {:?}", bake[1..bake.len()].to_vec());
        bake[1..bake.len()].to_vec()
    }
    /// Clears the overlap/tail buffer
    pub fn reset(&mut self) {
        self.tail.reset();
    }
    /// Computes the convolution between the input slice and the impulse
    /// response and copys the result to the output slice
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        match self.conv_mode{
            ConvolutionMode::TimeDomain =>{self.conv_time_domain(input, output);},
            ConvolutionMode::FrequencyDomain { block_size }=>{
                self.conv_freq_domain(input, output, block_size);
            }
        }
    }
    /// Copys the tail of the impulse response to the output
    /// 
    /// This should be called after the last process call
    pub fn flush(&mut self, output: &mut [f32]) {
        // Flushes remaining samples into the output
        for out_samp in output.iter_mut(){ //0..self.ir.len()-1
            *out_samp = self.tail.pop();
        }
    }
    /// Returns the necessary size of an output buffer given the size of 
    /// the input buffer
    pub fn get_output_size(&mut self, input_size: usize)->usize{
        input_size + self.ir.len() -1
    }
    /// Computes the convolution in the time domain
    fn conv_time_domain(&mut self, input: &[f32], output: &mut [f32]){
        let mut complete_buffer = vec![0.0;input.len()+self.ir.len() -1];
        for i in 0..input.len(){
            for j in 0..self.ir.len(){
                complete_buffer[i+j] += input[i] * self.ir[j];
            }
        } 
        for k in 0..self.tail.capacity(){
            complete_buffer[k] += self.tail.get(k);
        }
        output.copy_from_slice(&complete_buffer[0..output.len()]);
        self.tail.set_write_index(0);
        for m in input.len()..complete_buffer.len(){
            self.tail.push(complete_buffer[m]);
        }
    }
    /// Computes the convolution in the frequency domain
    fn conv_freq_domain(&mut self, input: &[f32], output: &mut [f32], block_size: usize) {
        assert!(input.len() <= block_size);
        let mut total_conv = vec![vec![Complex::<f32>::new(0.0, 0.0);self.fft_length.unwrap()];self.baked_ir.as_ref().unwrap().len()];
        let mut complex_input = vec![Complex::<f32>::new(0.0, 0.0); input.len()];
        complex_input.iter_mut().enumerate().for_each(|(i,sample)| 
                    {sample.re = input[i]; sample.im = 0.0}); 
        let mut input_block = vec![Complex::<f32>::new(0.0, 0.0); self.fft_length.unwrap()];
        input_block[0..input.len()].copy_from_slice(&complex_input);
        self.fft.as_mut().unwrap().process(&mut input_block);
        for chunk_id in 0..self.baked_ir.as_mut().unwrap().len(){
            for idx in 0..input_block.len(){
                total_conv[chunk_id][idx] = input_block[idx] * self.baked_ir.as_mut().unwrap()[chunk_id][idx] / self.fft_length.unwrap() as f32; // maybe divide by fft_size
            }
            self.ifft.as_mut().unwrap().process(&mut total_conv[chunk_id]);
            for i in 0..block_size{
                let popped = self.tail.pop();
                let mut push_val = 0.0;
                if chunk_id < 1{
                    output[i] = total_conv[chunk_id][i].re + popped;
                }else{
                    push_val = total_conv[chunk_id][i].re + total_conv[chunk_id-1][i+block_size].re + popped;
                }
                self.tail.push(push_val)
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use std::{borrow::BorrowMut, iter::zip};

    use super::*;
    use rand::Rng;
    use criterion::{black_box,criterion_group,criterion_main,Criterion};
    use rustfft::num_complex::ComplexFloat;
    
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
        let mut ir = vec![0.0,0.0,0.0,1.0,0.0,0.0,0.0]; // delay of 3
        let mut input: Vec<f32> = vec![0.0;10000];
        for value in input.iter_mut(){
            *value = (rng.gen_range(-10.0..10.0) as f32).trunc();
        }
        let mut conv = FastConvolver::new(&ir, ConvolutionMode::TimeDomain);
        let mut output: Vec<f32> = vec![0.0;conv.get_output_size(input.len())];
        let block_sizes = [1,13, 1023, 2048,30,17, 5000, 1897];
        for &block_size in block_sizes.iter(){
            let new_output =  vec![0.0;conv.get_output_size(input.len())];
            output.copy_from_slice(&new_output);
            let input_chunks = input.chunks(block_size);
            let mut output_chunks = output[0..input.len()].chunks_mut(block_size);
            for (in_chunk,out_chunk) in zip(input_chunks, output_chunks){
                conv.process(in_chunk, out_chunk);
            }
            conv.flush(&mut output[input.len()..]);
                for i in 0..output.len(){
                    //println!("output: {}, input: {}, count: {}",output[i],input[i],i);
                    if i > 3 && i < input.len()+3{
                        assert_eq!(output[i],input[i-3]);
                    }
                }
            println!("Block size: {} Passed!", block_size);
        }
    }
    #[test]
    fn identity_frequency_domain(){
        let mut rng = rand::thread_rng();
        let mut ir = vec![0.0;51];
        let input = [0.0,0.0,0.0,1.0,0.0,0.0,0.0,0.0,0.0,0.0];
        for value in ir.iter_mut(){
            *value = (rng.gen_range(-10.0..10.0) as f32).trunc();
        }
        let mut conv = FastConvolver::new(&ir, ConvolutionMode::FrequencyDomain { block_size: 10 });
        let mut output = vec![0.0;conv.get_output_size(input.len())];
        let now = std::time::Instant::now();
        //conv.print_baked();
        conv.process(&input, &mut output[0..input.len()]);
        let elapsed = now.elapsed();
        println!("{:.2?}: time elapsed",elapsed);
        for i in 0..input.len(){
            //println!("i: {}, output: {}, ir: {}", i,output[i],ir[i]);
            if i > 3{
                assert!(output[i].abs()-ir[i-3].abs() < 0.01);
            }
        }
    }

    #[test]
    fn flush_freq_domain(){
        let mut rng = rand::thread_rng();
        let mut ir = vec![0.0;51];
        let input = [0.0,0.0,0.0,1.0,0.0,0.0,0.0,0.0,0.0,0.0];
        for value in ir.iter_mut(){
            *value = (rng.gen_range(-10.0..10.0) as f32).trunc();
        }
        let mut conv = FastConvolver::new(&ir, ConvolutionMode::FrequencyDomain { block_size: 10 });
        let mut output = vec![0.0;conv.get_output_size(input.len())];
        let now = std::time::Instant::now();
        conv.process(&input, &mut output[0..input.len()]);
        conv.flush(&mut output[input.len()..]);
        let elapsed = now.elapsed();
        println!("{:.2?}: time elapsed",elapsed);
        for i in 0..output.len(){
            if i > 3 && i < ir.len()+3{
                //println!("i: {}, output: {}, ir: {}", i,output[i],ir[i-3]);
                assert!(output[i].abs()-ir[i-3].abs() < 0.001);
            }
        }
    }

    #[test]
    fn block_size_freq_domain(){
        let mut rng = rand::thread_rng();
        let mut ir = vec![0.0,0.0,0.0,1.0,0.0,0.0,0.0]; // delay of 3
        let mut input: Vec<f32> = vec![0.0;10000];
        for value in input.iter_mut(){
            *value = (rng.gen_range(-10.0..10.0) as f32).trunc();
        }
        let block_sizes = [1,13, 1023, 2048,30,17, 5000, 1897];
        
        for &block_size in block_sizes.iter(){
            let mut conv = FastConvolver::new(&ir, ConvolutionMode::FrequencyDomain { block_size });
            let mut output: Vec<f32> = vec![0.0;conv.get_output_size(input.len())];
            let new_output =  vec![0.0;conv.get_output_size(input.len())];
            output.copy_from_slice(&new_output);
            let input_chunks = input.chunks(block_size);
            let mut output_chunks = output[0..input.len()].chunks_mut(block_size);
            for (in_chunk,out_chunk) in zip(input_chunks, output_chunks){
                if in_chunk.len() < block_size{
                    
                }else{
                    conv.process(in_chunk, out_chunk);
                }
            }
            conv.flush(&mut output[input.len()..]);
                for i in 0..output.len(){
                    if i > 3 && i < input.len()+3{
                       // println!("output: {}, input: {}, count: {}",output[i],input[i-3],i);
                        assert!(output[i].abs()-input[i-3].abs() < 0.001);
                    }
                }
        }
    }
}
