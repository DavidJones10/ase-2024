use nih_plug::params::enums::Enum;

use crate::ring_buffer::RingBuffer;

pub struct CombFilter {
    buffers : Vec<RingBuffer::<f32>>,
    gain : f32,
    delay : f32,
    pub filterType : FilterType,
    numChannels : usize,
    max_delay: f32,
    sample_rate : f32
}

#[derive(Debug, Clone, Copy, PartialEq, Enum)]
pub enum FilterType {
    FIR,
    IIR,
}

#[derive(Debug, Clone, Copy)]
pub enum FilterParam {
    Gain,
    Delay,
}

#[derive(Debug, Clone)]
pub enum Error {
    InvalidValue { param: FilterParam, value: f32 }
}

impl CombFilter {
    pub fn new(filter_type: FilterType, max_delay_secs: f32, sample_rate_hz: f32, num_channels: usize) -> Self {

        let max_delay_ = (max_delay_secs * sample_rate_hz) as usize;
        let buffers_ = (0..num_channels)
            .map(|_| RingBuffer::<f32>::new(max_delay_ + 1))
            .collect();
        CombFilter{filterType: filter_type, 
                    buffers: buffers_,
                    gain: 0.0,
                    delay: 0.0,
                    numChannels : num_channels,
                    max_delay : max_delay_ as f32,
                    sample_rate : sample_rate_hz
                    }
        
    }

    pub fn reset(&mut self) {
        let index = self.find_read_index((self.delay*self.sample_rate) as i32);
        for buffer in self.buffers.iter_mut(){
            buffer.reset();
            buffer.set_read_index(index);
        }
    }

    pub fn process(&mut self, sample: &mut f32, channel_id: usize) {
        self.calculate_filter(sample, channel_id);
    }

    pub fn set_param(&mut self, param: FilterParam, value: f32) -> Result<(), Error> {
        use FilterParam::*;
        match param
        {
            Delay => {
                match self.filterType {
                    FilterType::FIR => {
                        if value > self.max_delay || value < 0.0{
                            println!("Delay must be greater than 0 and less than or equal to max delay size!");
                            return Err(Error::InvalidValue { param, value });
                        }
                    },
                    FilterType::IIR => {
                        if value <= 0.0 {
                            println!("Delay cannot be zero or less than zero in an IIR filter!");
                            return Err(Error::InvalidValue { param, value });
                        }
                        if value > self.max_delay{
                            println!("Delay must be less than or equal to max delay size!");
                            return Err(Error::InvalidValue { param, value });
                        }
                    }
                }
                self.delay = value;
                let index = self.find_read_index((value*self.sample_rate) as i32);
                for buffer in self.buffers.iter_mut(){
                    buffer.set_read_index(index);
                }
            },
            Gain => {
                if value < 0.0 || value > 1.0{
                    println!("Gain must be between 0.0 and 1.0!");
                    return Err(Error::InvalidValue { param, value });
                }
                self.gain = value
            },
            _ => {
                println!("Invalid Parameter!");
                return Err(Error::InvalidValue { param, value });
            }
        }
        Ok(())
    }

    pub fn get_param(&self, param: FilterParam) -> f32 {
        use FilterParam::*;
        match param
        {
            Delay => return self.delay,
            Gain => return self.gain
        }
    }
    fn find_read_index(&mut self, del_in_samps: i32) -> usize {
        let write_index = self.buffers[0].get_write_index() as i32;
        let mut calc_index = if write_index >= del_in_samps {
            write_index - del_in_samps
        } else {
            write_index - del_in_samps + self.buffers[0].capacity() as i32
        };
        if calc_index >= self.max_delay as i32 {
            calc_index -= self.buffers[0].capacity() as i32;
        }
        calc_index as usize
    }
    fn calculate_filter(&mut self, sample: &mut f32, channel: usize)
    {
        use FilterType::*;
        match self.filterType
                {
                    FIR => {
                        self.buffers[channel].push(*sample);
                        *sample += (self.gain * self.buffers[channel].pop());
                    }
                    IIR => {
                        let last_sample = self.buffers[channel].pop();
                        *sample += last_sample * self.gain;
                        self.buffers[channel].push(*sample);
                    }
                }
    }
}

//===========================================================
//                         TESTS                           //
//===========================================================
/* 
#[cfg(test)]

mod tests {
    use std::{f32::consts::PI};
    use crate::comb_filter;
    use super::*;   
    // Tests for zero output in periodic functions with delay equal to 1/2 the period
    #[test]
    fn test1() {
        let fs: f32 = 10.0;
        let freq: f32 = 3.0;
        let delay: f32 = 1.5; // Delay is 1/2 the period
        let channels = 1;
        let mut comb_filter_1 = CombFilter::new(comb_filter::FilterType::FIR, delay, fs, channels);
        comb_filter_1.set_param(comb_filter::FilterParam::Delay, delay);
        comb_filter_1.set_param(comb_filter::FilterParam::Gain, 1.0);
    
        let mut signal = vec![f32::default(); 50];
        let blockSize = 5;
        // Create slices for input and output
        let mut input_buffer = vec![0.0; blockSize*channels as usize];
        let mut output_buffer: Vec<f32> = vec![0.0; blockSize*channels as usize];
    
        // Fill the signal vector with samples
        for i in 0..50 {
            signal[i] = f32::cos(PI * 2.0 * freq * i as f32 / fs);
        }
        input_buffer.clear();
        let mut count = 0;
        for samp in signal.iter() {
            input_buffer.push(*samp);
            if input_buffer.len() == (blockSize * channels){
                let input_slice: Vec<_> = input_buffer[0..input_buffer.len()].chunks(channels as usize).collect();
                let mut output_slice: Vec<_> = output_buffer[0..input_buffer.len()].chunks_mut(channels as usize).collect();
                comb_filter_1.process(&input_slice, &mut output_slice);

                for channel in output_slice {
                    for sample in channel {
                        if count > (delay * fs * channels as f32) as usize
                        {
                            assert!((*sample).abs() < 0.00001);
                        }
                    }   
                }
                input_buffer.clear();
             }
            count += 1;
        }
    }
    // Tests for IIR magnitude increase when input freq matches feedbak 
    #[test]
    fn test2(){
        let fs: f32 = 8.0;
        let delay: f32 = 1.0; 
        let channels = 2;
        let mut comb_filter_1 = CombFilter::new(comb_filter::FilterType::IIR, delay, fs, channels);
        comb_filter_1.set_param(comb_filter::FilterParam::Delay, delay);
        comb_filter_1.set_param(comb_filter::FilterParam::Gain, 1.0);
        
        let signal = vec![0.0,1.0,0.0,1.0,0.0,1.0,0.0,1.0,0.0,1.0,
                                    0.0,1.0,0.0,1.0,0.0,1.0,0.0,1.0,0.0,1.0,
                                    0.0,1.0,0.0,1.0,0.0,1.0,0.0,1.0,0.0,1.0,
                                    0.0,1.0,0.0,1.0,0.0,1.0,0.0,1.0,0.0,1.0,
                                    0.0,1.0,0.0,1.0,0.0,1.0,0.0,1.0,
                                    0.0,1.0,0.0,1.0,0.0,1.0,0.0,1.0,0.0,1.0,
                                    0.0,1.0,0.0,1.0,0.0,1.0,0.0,1.0,0.0,1.0,
                                    0.0,1.0,0.0,1.0,0.0,1.0,0.0,1.0];
        let blockSize = 8;
        // Create slices for input and output
        let mut input_buffer = vec![0.0; blockSize*channels as usize];
        let mut output_buffer: Vec<f32> = vec![0.0; blockSize*channels as usize];
        
        input_buffer.clear();
        let mut count = 0;
        let mut block_count = 1;
        for samp in signal.iter() {
            input_buffer.push(*samp);
            if input_buffer.len() == (blockSize * channels){
                let input_slice: Vec<_> = input_buffer[0..input_buffer.len()].chunks(channels as usize).collect();
                let mut output_slice: Vec<_> = output_buffer[0..input_buffer.len()].chunks_mut(channels as usize).collect();
                comb_filter_1.process(&input_slice, &mut output_slice);
                let mut count2 = 0;
                for channel in output_slice.iter_mut() {
                    for (i,sample) in channel.iter_mut().enumerate() {
                            if count2 > (delay*fs*channels as f32) as i32 || count > 30{
                                if i == 0{
                                    assert_eq!(*sample, 0.0); // even indices are 0
                                }else{
                                    assert_eq!(*sample, block_count as f32);// odd indices are added to each other every fs*delay samples
                                }                                           // which is equal to block_count because blockSize = fs*delay
                            }
                            count2 +=1;
                    }   
                }
                block_count +=1;
                input_buffer.clear();
             }
            count += 1;
        }
    }
    // Tests with different block sizes 
    #[test]
    fn test3(){
        let fs: f32 = 5.0;
        let delay: f32 = 1.0; // Delay is 1/2 the period
        let channels = 2;
        let mut comb_filter_fir = CombFilter::new(comb_filter::FilterType::FIR, delay, fs, channels);
        let mut comb_filter_iir = CombFilter::new(comb_filter::FilterType::IIR, delay, fs, channels);
        
        let signal: Vec<f32> = (0..100).map(|i| (i % 2) as f32).collect();
        let block_sizes = [10, 25, 50]; // Varying block sizes
        for block_size in block_sizes.iter() {
            // Create slices for input and output
            let mut input_buffer = vec![0.0; (*block_size * channels) as usize];
            let mut output_buffer_fir = vec![0.0; (*block_size * channels) as usize];
            let mut output_buffer_iir = vec![0.0; (*block_size * channels) as usize];
        
            input_buffer.clear();
            comb_filter_fir.reset();
            comb_filter_iir.reset();
            comb_filter_fir.set_param(comb_filter::FilterParam::Delay, delay);
            comb_filter_fir.set_param(comb_filter::FilterParam::Gain, 1.0);
            comb_filter_iir.set_param(comb_filter::FilterParam::Delay, delay);
            comb_filter_iir.set_param(comb_filter::FilterParam::Gain, 1.0);
            let mut count = 0;
            let mut count3 = 0;
            let mut sample_compare = 1.0;
            for samp in signal.iter() {
                input_buffer.push(*samp);
                if input_buffer.len() == (*block_size * channels) as usize {
                    let input_slice: Vec<_> = input_buffer[..].chunks(channels as usize).collect();
                    let mut output_slice_fir: Vec<_> = output_buffer_fir[..].chunks_mut(channels as usize).collect();
                    let mut output_slice_iir: Vec<_> = output_buffer_iir[..].chunks_mut(channels as usize).collect();

                    comb_filter_fir.process(&input_slice, &mut output_slice_fir);
                    comb_filter_iir.process(&input_slice, &mut output_slice_iir);
                    let mut count2 = 0;
                    for (channel_fir, channel_iir) in output_slice_fir.iter_mut().zip(output_slice_iir.iter_mut()) {
                        for (i,(sample_fir, sample_iir)) in channel_fir.iter_mut().zip(channel_iir.iter_mut()).enumerate() {
                            //dbg!(count);
                            if count2 > (delay*fs*channels as f32) as i32 || count > *block_size*2{
                                if i != 0{
                                    //if count
                                    assert_eq!(*sample_fir,2.0);// Fir will be the same after delay kicks in 
                                    let val = count3 % ((delay*fs) as i32);
                                    if val == 0{
                                        sample_compare += 1.0;
                                    }
                                    assert_eq!(*sample_iir, sample_compare);// Odd indices will be +1 every delay*fs samples
                                    count3 += 1;
                                }
                                else{
                                    assert_eq!(*sample_fir, 0.0);// All even indices will be zero
                                    assert_eq!(*sample_fir, 0.0);
                                }
                            }
                            //print!("Sample: {}, Block Size: {} \n", *sample_fir, *block_size);
                            //assert_eq!(*sample_fir, 0.0);
                            //assert_eq!(*sample_iir, 0.0);
                            count2 += 1;
                        }   
                    }
                    input_buffer.clear();
                }
                count += 1;
            }
            output_buffer_fir = vec![0.0; (*block_size * channels) as usize];
            output_buffer_iir = vec![0.0; (*block_size * channels) as usize];
        }
    }

    // Tests if zero valued input results in zero valued output
    #[test]
    fn test4(){
        let fs: f32 = 10.0;
        let delay: f32 = 1.0;
        let channels = 1;
        let mut comb_filter_1 = CombFilter::new(comb_filter::FilterType::FIR, delay, fs, channels);
        let mut comb_filter_2= CombFilter::new(comb_filter::FilterType::IIR, delay, fs, channels);
        comb_filter_1.set_param(comb_filter::FilterParam::Delay, delay);
        comb_filter_1.set_param(comb_filter::FilterParam::Gain, 1.0);
        comb_filter_2.set_param(comb_filter::FilterParam::Delay, delay);
        comb_filter_2.set_param(comb_filter::FilterParam::Gain, 1.0);
        
        let mut signal = vec![f32::default(); 30];
        let blockSize = 5;
        // Create slices for input and output
        let mut input_buffer = vec![0.0; blockSize*channels as usize];
        let mut output_buffer: Vec<f32> = vec![0.0; blockSize*channels as usize];
        let mut output_buffer_1: Vec<f32> = vec![0.0; blockSize*channels as usize];
    
        
        input_buffer.clear();
        
        for samp in signal.iter() {
            input_buffer.push(*samp);
            if input_buffer.len() == (blockSize * channels){
                let input_slice: Vec<_> = input_buffer[0..input_buffer.len()].chunks(channels as usize).collect();
                let mut output_slice: Vec<_> = output_buffer[0..input_buffer.len()].chunks_mut(channels as usize).collect();
                let mut output_slice_1: Vec<_> = output_buffer_1[0..input_buffer.len()].chunks_mut(channels as usize).collect();
                comb_filter_1.process(&input_slice, &mut output_slice);
                comb_filter_2.process(&input_slice, &mut output_slice_1);

                
                for (channel,channel_) in output_slice.iter_mut().zip(output_slice_1.iter_mut()) {
                    for (sample,sample_) in channel.iter_mut().zip(channel_.iter_mut()) {
                        assert_eq!(*sample, 0.0);
                        assert_eq!(*sample_, 0.0);
                    }   
                }
                input_buffer.clear();
             }
        }
    }

    // Tests for correct values with high number of channels with FIR
    #[test]
    fn test5(){
        let fs: f32 = 5.0;
        let delay: f32 = 1.0; 
        let channels = 5;
        let mut comb_filter_1 = CombFilter::new(comb_filter::FilterType::FIR, delay, fs, channels);
        comb_filter_1.set_param(comb_filter::FilterParam::Delay, delay);
        comb_filter_1.set_param(comb_filter::FilterParam::Gain, 1.0);
        
        let signal = vec![0.0,1.0,2.0,3.0,4.0,0.0,1.0,2.0,3.0,4.0,
                                    0.0,1.0,2.0,3.0,4.0,0.0,1.0,2.0,3.0,4.0,
                                    0.0,1.0,2.0,3.0,4.0,0.0,1.0,2.0,3.0,4.0,
                                    0.0,1.0,2.0,3.0,4.0,0.0,1.0,2.0,3.0,4.0,
                                    0.0,1.0,2.0,3.0,4.0,0.0,1.0,2.0,3.0,4.0,
                                    0.0,1.0,2.0,3.0,4.0,0.0,1.0,2.0,3.0,4.0,
                                    0.0,1.0,2.0,3.0,4.0,0.0,1.0,2.0,3.0,4.0,
                                    0.0,1.0,2.0,3.0,4.0,0.0,1.0,2.0,3.0,4.0,
                                    0.0,1.0,2.0,3.0,4.0,0.0,1.0,2.0,3.0,4.0,
                                    0.0,1.0,2.0,3.0,4.0,0.0,1.0,2.0,3.0,4.0,];
        let blockSize = 10;
        // Create slices for input and output
        let mut input_buffer = vec![0.0; blockSize*channels as usize];
        let mut output_buffer: Vec<f32> = vec![0.0; blockSize*channels as usize];
        
        input_buffer.clear();
        let mut count = 0;
        for samp in signal.iter() {
            input_buffer.push(*samp);
            if input_buffer.len() == (blockSize * channels){
                let input_slice: Vec<_> = input_buffer[0..input_buffer.len()].chunks(channels as usize).collect();
                let mut output_slice: Vec<_> = output_buffer[0..input_buffer.len()].chunks_mut(channels as usize).collect();
                comb_filter_1.process(&input_slice, &mut output_slice);
                let mut count2 = 0;
                for channel in output_slice.iter_mut() {
                    for (i,sample) in channel.iter_mut().enumerate() {
                            if count2 > (delay*fs*channels as f32) as i32 || count > 79{
                                assert_eq!(*sample, (2*i) as f32);
                            }
                            count2 +=1;
                    }   
                }
                input_buffer.clear();
             }
            count += 1;
        }
    }
        
}
*/