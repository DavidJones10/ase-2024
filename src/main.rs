use std::{fs::File, io::Write};

use vibrato::Vibrato;

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
    if args.len() < 3 {
        eprintln!("Usage: {} <input wave filename> <output text filename>", args[0]);
        return
    }

    // Open the input wave file
    let mut reader = hound::WavReader::open(&args[1]).unwrap();
    let spec = reader.spec();
    let channels = spec.channels;
    let sr = spec.sample_rate;
    let new_spec = hound::WavSpec {
        channels: spec.channels,
        sample_rate: spec.sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(&args[2], new_spec).unwrap();
    let mut vibe = Vibrato::new(channels as usize, sr as f32, 50.0);
    let mut delay = 8.0;
    let mut rate = 5.0;
    let width = 0.1;
    let dry_wet = 1.0;
    vibe.set_delay(delay);
    vibe.set_rate(rate);
    vibe.set_width(width);
    vibe.set_dry_wet(dry_wet);
    
    for (i, sample) in reader.samples::<i32>().enumerate() {
        //let sample = sample.unwrap() as f32 / i8::MAX as f32; // For i8
        //let sample = sample.unwrap() as f32 / (1 << 15) as f32; // for i16
        let sample = sample.unwrap() as f32 / 16777215.0; // for i24
        /* if i % 22580 == 0{
            delay += 0.1;
            vibe.set_delay(delay);
        } */
        let processed = vibe.process(sample,i%channels as usize);
        writer.write_sample(processed);
    }
    writer.finalize().unwrap();
}
