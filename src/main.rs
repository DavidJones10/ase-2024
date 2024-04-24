use std::{fs::File, io::Write, iter::zip};

mod ring_buffer;
mod fast_convolver;
use hound::SampleFormat;
use fast_convolver::FastConvolver;

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main() {
   show_info();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 5 {
        eprintln!("Usage: {} <input wave filename> <output text filename> <block size> 
        <ir length in samples> <Domain- 0: Time, 1: Frequency", args[0]);
        return
    }

    // Open the input wave file
    let block_size: usize = match args[3].parse(){
        Ok(value)=>value,
        Err(_) => {
            eprintln!("Invalid BlockSize: {}", args[3]);
            return;
        }
    };
    let ir_len: usize = match args[4].parse() {
        Ok(value)=>value,
        Err(_) => {
            eprintln!("Invalid ir_len: {}", args[4]);
            return;
        }
    };
    let conv_mode: i32 = match args[5].parse() {
        Ok(value)=>value,
        Err(_) => {
            eprintln!("Invalid conv_mode: {}", args[5]);
            return;
        }
    };
    let (input,sample_rate) = create_buffer(args[1].as_str());
    let new_spec = hound::WavSpec {
        channels: 1,
        sample_rate: sample_rate as u32,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(&args[2], new_spec).unwrap();

    let mut impulse_response = vec![0.0;ir_len];
    impulse_response[ir_len/2] = 1.0;
    let mut convolver: FastConvolver;

    if conv_mode == 0 {
        convolver = FastConvolver::new(&impulse_response, fast_convolver::ConvolutionMode::TimeDomain);
    } else if conv_mode == 1 {
        convolver = FastConvolver::new(&impulse_response, fast_convolver::ConvolutionMode::FrequencyDomain { block_size });
    } else {
        // Handle invalid conv_mode value
        panic!("Invalid convolution mode");
    }

    let mut output: Vec<f32> = vec![0.0;convolver.get_output_size(input.len())];
    let input_chunks = input.chunks(block_size);
    let mut output_chunks = output[0..input.len()].chunks_mut(block_size);
    let now = std::time::Instant::now();
    for (in_chunk,out_chunk) in zip(input_chunks, output_chunks){
        if in_chunk.len() >= block_size{
            convolver.process(in_chunk, out_chunk);
        }   
    }
    convolver.flush(&mut output[input.len()..]);
    let elapsed = now.elapsed();
    println!("{:.2?}: time elapsed",elapsed);
    for i in 0..output.len(){
        writer.write_sample(output[i]);
    }
    writer.finalize().unwrap();
}




fn create_buffer(path: &str) -> (Vec<f32>, f32) {
    let mut reader = hound::WavReader::open(path).unwrap();
    let sample_format = reader.spec().sample_format;
    let num_channels = reader.spec().channels as usize;
    let sample_rate = reader.spec().sample_rate as f32;
    let length = reader.len();

    // Create a Vec to store samples
    let mut buffer = Vec::<f32>::with_capacity(length as usize);

    // Determine the conversion factor based on sample format
    let conversion_factor = match sample_format {
        SampleFormat::Float => 1.0, // No conversion needed
        SampleFormat::Int => {
            match reader.spec().bits_per_sample {
                8 => 1.0 / (i8::MAX as f32),
                16 => 1.0 / (i16::MAX as f32),
                24 => 1.0 / (8388608 as f32), 
                _ => panic!("Unsupported bit depth"),
            }
        }
    };

    // Read samples and store them in the buffer Vec
    match sample_format {
        SampleFormat::Float => {
            let mut samples = reader.samples::<f32>();
            for _ in 0..length {
                if let Some(sample) = samples.next() {
                    if let Ok(sample_value) = sample {
                        let sample_float = sample_value * conversion_factor;
                        buffer.push(sample_float);
                    }
                }
            }
        }
        SampleFormat::Int => {
            let mut samples = reader.samples::<i32>();
            for _ in 0..length {
                if let Some(sample) = samples.next() {
                    if let Ok(sample_value) = sample {
                        let sample_float = (sample_value as f32) * conversion_factor;
                        buffer.push(sample_float);
                    }
                }
            }
        }
    }

    (buffer, sample_rate)
}
