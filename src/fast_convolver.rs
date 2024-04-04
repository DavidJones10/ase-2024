use std::mem::size_of;

use crate::ring_buffer::RingBuffer;
//use realfft::

struct FastConvolver {
    ir_buffer: Vec<f32>,
    audio_buffer: RingBuffer<f32>,
    conv_mode: ConvolutionMode,
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
            ir_buffer: impulse_response.to_vec(),
            audio_buffer: buffer,
            conv_mode: mode,
        }
    }

    pub fn reset(&mut self) {
        self.audio_buffer.reset();
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        let block_size = input.len();
        //todo!("implement");
        match self.conv_mode{
            ConvolutionMode::TimeDomain =>{
                for (i, &x) in input.iter().enumerate() {
                    self.audio_buffer.push(x);
                    let y = self.convolve_time_domain();
                    output[i] = y;
                }
            },
            ConvolutionMode::FrequencyDomain { block_size }=>{

            }
        }
        // First handle time domain, then do frequency domain
    }

    pub fn flush(&mut self, output: &mut [f32]) {
        todo!("implement")
        // Flushes remaining samples into the output
    }
    fn convolve_time_domain(&self) -> f32 {
        let mut result = 0.0;
        for (i, &h) in self.ir_buffer.iter().enumerate() {
            let x = self.audio_buffer.get(i);
            result += h * x;
        }
        result
    }

    // TODO: feel free to define other functions for your own use
}

// TODO: feel free to define other types (here or in other modules) for your own use
