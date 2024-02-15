#![allow(warnings)]
use std::{fs::File, io::Write, sync::mpsc::channel};
mod comb_filter;
use comb_filter::CombFilter;
use hound::WavReader;
use std::convert::From;

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn scale_f32_to_i16(sample: f32) -> i16 {
    let dequantized_sample = sample * (i16::MAX as f32);
    dequantized_sample.round() as i16
}

fn main() {
   show_info();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 6 {
        eprintln!("Usage: {} <input wave filename> <output wav filename> <filter type: IIR or FIR> <delay in seconds> <Gain 0-1>", args[0]);
        std::process::Command::new("cargo").arg("test").status().unwrap();
        return
    }

    // Open the input wave file
    let mut reader = hound::WavReader::open(&args[1]).unwrap();
    let spec = reader.spec();
    let channels = spec.channels;
    let sr = spec.sample_rate;
    let samp_format = spec.sample_format;
    let bps = spec.bits_per_sample;
    let delay_secs: f32 = match args[4].parse() {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Invalid delay in seconds: {}", args[4]);
            return;
        }
    };
    let gain: f32 = match args[5].parse(){
        Ok(value) => value,
        Err(_) => {
            eprintln!("Invalid delay in seconds: {}", args[5]);
            return;
        }
    };
    let mut writer = hound::WavWriter::create(&args[2], spec).unwrap();
    let mut comb_filter;
    if &args[3] == "FIR"{
        comb_filter = CombFilter::new(comb_filter::FilterType::FIR,delay_secs,
            sr as f32, channels as usize);
    } else if &args[3] == "IIR"{
        comb_filter = CombFilter::new(comb_filter::FilterType::IIR,delay_secs,
            sr as f32, channels as usize);
    } else {
        eprintln!("FilterType should be either FIR or IIR");
        return
    }
    let _ = comb_filter.set_param(comb_filter::FilterParam::Delay, delay_secs);
    let _ = comb_filter.set_param(comb_filter::FilterParam::Gain, gain);
    // TODO: Modify this to process audio in blocks using your comb filter and write the result to an audio file.
    //       Use the following block size:

    let block_size = 1024;
    let mut input_buffer = vec![0.0; block_size*channels as usize];
    let mut output_buffer: Vec<f32> = vec![0.0; block_size*channels as usize];
    input_buffer.clear();
    for sample in reader.samples::<i16>() {
        
        let sample = sample.unwrap() as f32 / (1 << 15) as f32;
        // Push the sample into the input buffer
        input_buffer.push(sample);
        // Process a block when the input buffer is full
        if input_buffer.len() == (block_size * channels as usize) {
            // Split the block into chunks of size: channels
            let input_slice: Vec<_> = input_buffer.chunks(channels as usize).collect();
            let mut output_slice: Vec<_> = output_buffer.chunks_mut(channels as usize).collect();
            
            // Process the block
            comb_filter.process(&input_slice, &mut output_slice);

            // Write processed samples to the text file
            for channel in output_slice {
                for sample in channel {
                    let sampleInt = scale_f32_to_i16(*sample);
                    writer.write_sample(sampleInt).unwrap();
                }
            }
            // Clear the input buffer
            input_buffer.clear();
        }
    }
    /* */
    if ! (input_buffer.len()==0){
            let input_slice: Vec<_> = input_buffer[0..input_buffer.len()].chunks(channels as usize).collect();
            let mut output_slice: Vec<_> = output_buffer[0..input_buffer.len()].chunks_mut(channels as usize).collect();
            
            // Process the block
            comb_filter.process(&input_slice, &mut output_slice);

            // Write processed samples to the text file
            for channel in output_slice {
                for sample in channel {
                    let sampleInt = scale_f32_to_i16(*sample);
                    writer.write_sample(sampleInt).unwrap();
                    
                }
            }
            input_buffer.clear();
    }
    assert_eq!(input_buffer.len(),0);
    /**/
    writer.finalize().unwrap();
    println!("Yay! No errors! Use this file wisely...");
    /* 
    // Read audio data and write it to the output text file (one column per channel)
    let mut out = File::create(&args[2]).expect("Unable to create file");
    for (i, sample) in reader.samples::<i16>().enumerate() {
        let sample = sample.unwrap() as f32 / (1 << 15) as f32;
        write!(out, "{}{}", sample, if i % channels as usize == (channels - 1).into() { "\n" } else { " " }).unwrap();
    }
    */
}
// if not good inputs, run cargo test
//tests

// test (a-b).abs() < epsilon epsilon = .00001

// does it work for differnt block sizes
//===========================================================
//                         TESTS                           //
//===========================================================
#[cfg(test)]
mod tests {
    use std::{f32::consts::PI};
    
    

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
    // DO THIS ASAP use same principle as 5
    #[test]
    fn test2(){
        todo!();
    }
    // Tests with different block sizes NOT DONE!!!
    #[test]
    fn test3(){
        todo!();
        let fs: f32 = 10.0;
        let freq: f32 = 2.0;
        let delay: f32 = 1.0; // Delay is 1/2 the period
        let channels = 1;
        let mut comb_filter_1 = CombFilter::new(comb_filter::FilterType::FIR, 1.0, 10.0, channels);
        let mut comb_filter_2= CombFilter::new(comb_filter::FilterType::IIR, 1.0, 10.0, channels);
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
#[allow(warnings)]
fn do_nothing(){
    
}
