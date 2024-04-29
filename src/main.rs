use std::{fs::File, io::Write};

use vibrato::Vibrato;
use hound::SampleFormat;
mod ring_buffer;
mod vibrato;
mod lfo;

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main() {
   show_info();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 6 {
        eprintln!("Usage: {} <input wave filename> <output text filename> <Delay in ms> <Modulation rate in Hz> <Modulation Width 0-1> <DryWet 0-1>", args[0]);
        std::process::Command::new("cargo").arg("test").status().unwrap();
        return
    }

    // Open the input wave file
    let mut reader = hound::WavReader::open(&args[1]).unwrap();
    let spec = reader.spec();
    let channels = spec.channels;
    let sample_rate = spec.sample_rate;
    let sample_format = spec.sample_format;
    let length = reader.len();
    let new_spec = hound::WavSpec {
        channels: spec.channels,
        sample_rate: spec.sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let delay: f32 = match args[3].parse() {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Invalid delay in ms: {}", args[4]);
            return;
        }
    };
    let rate: f32 = match args[4].parse() {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Invalid rate in Hz: {}", args[4]);
            return;
        }
    };
    let width: f32 = match args[5].parse() {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Invalid width: {}", args[4]);
            return;
        }
    };
    let dry_wet: f32 = match args[6].parse() {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Invalid dry_wet: {}", args[4]);
            return;
        }
    };

    let mut writer = hound::WavWriter::create(&args[2], new_spec).unwrap();
    let mut vibe = Vibrato::new(channels as usize, sample_rate as f32, 50.0);
    
    vibe.set_delay(delay);
    vibe.set_rate(rate);
    vibe.set_width(width);
    vibe.set_dry_wet(dry_wet);
    
    // For accurately converting input files to floats
    let conversion_factor = match sample_format {
        SampleFormat::Float => 1.0, 
        SampleFormat::Int => {
            match reader.spec().bits_per_sample {
                8 => 1.0 / (i8::MAX as f32),
                16 => 1.0 / (i16::MAX as f32),
                24 => 1.0 / (8388608 as f32), 
                _ => panic!("Unsupported bit depth"),
            }
        }
    };
    match sample_format{
        SampleFormat::Float => {
            let mut samples = reader.samples::<f32>();
            for i in 0..(length as usize) {
                if let Some(sample) = samples.next() {
                    let sample_float = sample.unwrap() * conversion_factor;
                    let processed = vibe.process(sample_float,i%channels as usize);
                    writer.write_sample(processed);
                    
                }
            }
        }, 
        SampleFormat::Int => {
            let mut samples = reader.samples::<i32>();
            for i in 0..(length as usize) {
                if let Some(sample) = samples.next() {
                    let sample_float = sample.unwrap() as f32 * conversion_factor;
                    let processed = vibe.process(sample_float,i%channels as usize);
                    writer.write_sample(processed);
                    
                }
            }
        }
    }
    writer.finalize().unwrap();
}
