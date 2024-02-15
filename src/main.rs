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
    let mut count = 0;
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
// test sinusoid delayed by Period/2 ignore first half P then assert 0
// test (a-b).abs() < epsilon epsilon = .00001
// do all channels work if channels > 1 or 2
// does it work for differnt block sizes
