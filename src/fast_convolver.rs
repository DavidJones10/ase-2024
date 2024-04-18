//use core::slice::SlicePattern;
use std::mem::size_of;

use crate::ring_buffer::RingBuffer;
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};
use rustfft::{num_complex::{Complex, Complex32}, num_traits::PrimInt, Fft, FftPlanner};


struct FastConvolver {
    ir: Vec<f32>,
    tail: RingBuffer<f32>,
    conv_mode: ConvolutionMode,
    fft: Option<std::sync::Arc<dyn Fft<f32>>>,
    ifft: Option<std::sync::Arc<dyn Fft<f32>>>,
    baked_ir: Option<Vec<Vec<Complex32>>>,
    inputs: Option<RingBuffer<f32>>,
    overlap: Option<Vec<f32>>,
    fft_length: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
pub enum ConvolutionMode {
    TimeDomain,
    FrequencyDomain { block_size: usize },
}

impl FastConvolver {
    // Block Size will be defined as the size of the blocks of input
    pub fn new(impulse_response: &[f32], mode: ConvolutionMode) -> Self {
        let buffer = RingBuffer::<f32>::new(impulse_response.len()-1);
        let mut plan = FftPlanner::<f32>::new();
        let mut fft = None;
        let mut size = None;
        let mut lap = None;
        let mut baked = None;
        let mut ins = None;
        let ifft = match mode {
            ConvolutionMode::TimeDomain =>{
                None
            },
            ConvolutionMode::FrequencyDomain { block_size }=>{
                size = Some(block_size*2);   
                lap = Some(vec![0.0;block_size]);           
                fft = Some(plan.plan_fft_forward(size.unwrap()));
                baked = Some(FastConvolver::bake_ir_1(impulse_response, block_size));
                ins = Some(RingBuffer::<f32>::new(impulse_response.len()*2));
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
            inputs: ins,
            overlap: lap,
            fft_length: size,
        }

    }

    fn bake_ir(impulse_response: &[f32], block_size: usize)-> Vec<Vec<Complex32>>{
        let mut bake = vec![vec![]];
        let mut chunks_vec= vec![vec![0.0]];
        let mut chunks = impulse_response.chunks_exact(block_size);
        chunks.clone().for_each(|chunk|{chunks_vec.push(chunk.to_vec())});
        
        if impulse_response.len() % block_size != 0{
            let remainder = chunks.remainder().to_vec();
            let mut final_chunk = vec![0.0;block_size];
            final_chunk[0..remainder.len()].copy_from_slice(remainder.as_slice());
            chunks_vec.push(final_chunk);
        }
        let mut plan = RealFftPlanner::<f32>::new();
        for chunk_id in 1..chunks_vec.len(){
            let mut padded = vec![0.0;block_size*2];
    
            padded[0..block_size].copy_from_slice(&chunks_vec[chunk_id]);

            let fft = plan.plan_fft_forward(block_size*2);
            let mut complex_values = vec![Complex32::new(0.0,0.0);block_size+1];
            fft.process(&mut padded,&mut complex_values).unwrap();
            for i in 1..block_size{
                complex_values.push(complex_values[block_size-i].conj()); // Adds the other half of the spectrum using the complex conjugate
            }
            bake.push(complex_values);
        }
        bake
    }
    fn bake_ir_1(impulse_response: &[f32], block_size: usize)-> Vec<Vec<Complex32>>{
        let mut plan = FftPlanner::<f32>::new();
        let fft = plan.plan_fft_forward(block_size*2);
        impulse_response.chunks(block_size)
            .map(|chunk| {
                let mut input = vec![Complex::new(0.0, 0.0); block_size];
                input.iter_mut().zip(chunk).for_each(|(a, &b)| *a = Complex::new(b, 0.0));
                let mut padded = [input,vec![Complex::<f32>::new(0.0, 0.0);block_size]].concat();
                // Use the fft plan directly
                fft.process(&mut padded);
                padded
            })
            .collect()
    }

    pub fn reset(&mut self) {
        self.tail.reset();
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        match self.conv_mode{
            ConvolutionMode::TimeDomain =>{self.conv_time_domain(input, output);},
            ConvolutionMode::FrequencyDomain { block_size }=>{
                self.conv_freq_domain_1(input, output, block_size);
            }
        }
    }

    pub fn flush(&mut self, output: &mut [f32]) {
        // Flushes remaining samples into the output
        for out_samp in output.iter_mut(){ //0..self.ir.len()-1
            *out_samp = self.tail.pop();
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
        for k in 0..self.tail.capacity(){
            complete_buffer[k] += self.tail.get(k);
        }
        output.copy_from_slice(&complete_buffer[0..output.len()]);
        self.tail.set_write_index(0);
        for m in input.len()..complete_buffer.len(){
            self.tail.push(complete_buffer[m]);
        }
    }
    fn conv_freq_domain_1(&mut self, input: &[f32], output: &mut [f32], block_size: usize) {
        
        //self.overlap.as_mut().unwrap().fill(0.0);
    
        
        for (i, chunk) in input.chunks(block_size).enumerate() {
            let mut input_block = vec![Complex::new(0.0, 0.0); block_size];
            
            input_block.iter_mut().zip(chunk).for_each(|(result, &val)| *result = Complex::new(val, 0.0));
            let mut padded = [input_block,vec![Complex::<f32>::new(0.0,0.0);block_size]].concat();
            self.fft.as_mut().unwrap().process(&mut padded);
    
            
            let mut out_block = vec![Complex::new(0.0, 0.0); self.fft_length.unwrap()];
            
            for (j, (input_value, ir_value)) in padded.iter().zip(
                self.baked_ir.as_mut().unwrap().get(i).unwrap_or(
                    &vec![Complex::new(0.0, 0.0); self.fft_length.unwrap()],
                ).iter()
            ).enumerate() {
                out_block[j] = *input_value * *ir_value / self.fft_length.unwrap() as f32;
            }
    
            
            self.ifft.as_mut().unwrap().process(&mut out_block);
    
            // Update the overlap buffer with the result
            for (j, &comp_val) in out_block.iter().enumerate() {
                let idx = i * self.fft_length.unwrap() + j;
                let overlap_len = self.overlap.as_mut().unwrap().len();
                self.overlap.as_mut().unwrap()[idx % overlap_len] += comp_val.re;
            }
        }
    
        
        output.copy_from_slice(&self.overlap.as_mut().unwrap()[..input.len()]);
    }
    




    fn conv_freq_domain(&mut self, input: &[f32],output: &mut [f32],block_size: usize){
            for sample in input.iter(){
                self.inputs.as_mut().unwrap().push(*sample);
                if self.inputs.as_mut().unwrap().len() == block_size{
                    let mut fft_vec = vec![Complex::<f32>::new(0.0, 0.0);block_size*2];
                    for i in 0..block_size{
                        fft_vec[i] = Complex::new(self.inputs.as_mut().unwrap().get(i),
                                                    self.inputs.as_mut().unwrap().get(i+block_size));
                    }
                    self.fft.as_mut().unwrap().process(&mut fft_vec);
                    let mut ifft_vecs = vec![];
                     let mut flattened_output = vec![0.0;self.baked_ir.as_mut().unwrap().len()*block_size*2];

                     for (block_id,block) in self.baked_ir.as_mut().unwrap().iter().enumerate(){
                         ifft_vecs.push(vec![]);
                         for j in 0..block_size*2{
                             //println!("blocklen {}, fft_vec {}, block_id {}", block.capacity(),fft_vec.len(),block_id);
                             ifft_vecs[block_id].push( fft_vec[j]* block[j]);//block[j]*fft_vec[j]
                             println!("Complex value: {}", ifft_vecs[block_id][j].re);
                         }
                         self.ifft.as_mut().unwrap().process(&mut ifft_vecs[block_id]);
                         for k in 0..self.fft_length.unwrap(){
                             ifft_vecs[block_id][k] /= self.fft_length.unwrap() as f32;
                             flattened_output[block_id*block_size+k] = ifft_vecs[block_id][k].re;
                             flattened_output[(block_id+1)*block_size+k] = ifft_vecs[block_id][k].im;
                         }
                     }
                     self.tail.set_write_index(0);
                     let mut idx = 0;
                     for i in 0..flattened_output.len(){
                         let value = self.tail.get(i);
                         let sum = value + flattened_output[i];
                         if i < output.len(){
                             output[i] = sum;
                         }else if i < (block_size-2)*self.baked_ir.as_mut().unwrap().len(){
                            let last_write_ptr = self.tail.get_write_index();
                            self.tail.set_write_index(i);
                            self.tail.put(sum);
                            self.tail.set_write_index(last_write_ptr);
                         }
                         else{self.tail.push(sum);}
                     }
                }
            }
            
            
     }

}
fn next_pow_2(mut val: usize) -> usize {
    if val == 0{
        return 2
    }
    val -= 1;
    val |= val >> 1;
    val |= val >> 2;
    val |= val >> 4;
    val |= val >> 8;
    val |= val >> 16;
    val + 1
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
        conv.process(&input, &mut output[0..input.len()]);
        let elapsed = now.elapsed();
        println!("{:.2?}: time elapsed",elapsed);
        for i in 0..input.len(){
            println!("i: {}, output: {}, ir: {}", i,output[i],ir[i]);
            /* if i > 3{
                assert_eq!(output[i],ir[i-3]);
            } */
        }
        /* let now = std::time::Instant::now();
        conv.process(&input, &mut output);
        let elapsed = now.elapsed();
        println!("{:.2?}: time elapsed",elapsed); */
        /* criterion_group!();
        criterion_main!(); */
    }

    #[test]
    fn testPow2(){
        let i = 25;
        let j = 1020;
        let k = 0;
        let l = 1025;
        assert_eq!(next_pow_2(i),32);
        assert_eq!(next_pow_2(j),1024);
        assert_eq!(next_pow_2(k), 2);
        assert_eq!(next_pow_2(l),2048);
    }
}

//cargo flamegraph -- <parameters>
//google-chrome flamegraph.svg
// cargo flamegraph -f -- <parameters>  //adds a lot more samples
// cargo run 

// godbolt.org

// use simd


/* 
fn conv_freq_domain(&mut self, input: &[f32],output: &mut [f32],block_size: usize){ 
    if self.ir.len() <= block_size{
        let mut ir_copy = vec![0.0;self.fft_length.unwrap()];
        let mut input_copy = vec![0.0;self.fft_length.unwrap()];
        ir_copy[..self.ir.len()].copy_from_slice(&self.ir);
        input_copy[..input.len()].copy_from_slice(&input);
        let mut ir_comp = self.fft.as_mut().unwrap().make_output_vec();
        let mut in_comp = self.fft.as_mut().unwrap().make_output_vec();
        self.fft.as_mut().unwrap().process(&mut ir_copy, &mut ir_comp);
        self.fft.as_mut().unwrap().process(&mut input_copy, &mut in_comp);
        for i in 0..ir_comp.len(){
             in_comp[i] *= ir_comp[i]
        }
        let mut ifft_buff = self.ifft.as_mut().unwrap().make_output_vec();
        self.ifft.as_mut().unwrap().process(&mut in_comp, &mut ifft_buff);
        for i in 0..output.len(){
            output[i] = 0.0;
        }

    }else{

    }
    for i in 0..block_size{

    }
    /* let mut ir_copy = vec![0.0;block_size];
    let mut input_copy = vec![0.0;block_size];
    let mut output_samples = real.make_output_vec();
    let mut output_samples_ir = real.make_output_vec();
    input_copy[..block_size.min(input.len())].copy_from_slice(&input[..block_size.min(input.len())]);
    ir_copy[..block_size.min(self.ir.len())].copy_from_slice(&self.ir[..block_size.min(self.ir.len())]);

    for i in 0..output_samples.len(){
        output_samples[i] *= output_samples_ir[i];
    } */
    
} */