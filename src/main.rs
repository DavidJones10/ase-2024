use std::{f32::consts::PI, fs::File, io::Write};

use ring_buffer::RingBuffer;
mod ring_buffer;


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
    let lfo_freq = 7.0;
    let center_del: f32 = 0.0009*sr as f32;
    let depth: f32 = 0.0005*sr as f32;
    let max_delay_size = (3 * sr) as usize;
    let dryWet = 0.5;
    let mut left_buffer = RingBuffer::<f32>::new(max_delay_size);
    let mut right_buffer = RingBuffer::<f32>::new(max_delay_size);

    // Read audio data and write it to the output text file (one column per channel)
    //let mut out = File::create(&args[2]).expect("Unable to create file");
    for (i, sample) in reader.samples::<i32>().enumerate() {
        //let sample = sample.unwrap() as f32 / i8::MAX as f32; // For i8
        //let sample = sample.unwrap() as f32 / (1 << 15) as f32; // for i16
        let sample = sample.unwrap() as f32 / 16777215.0; // for i24
        let mut processed = 0.0;
        let mut lfo: f32 = 0.0;
        if i % 2 == 0{
            lfo = depth*(((i as f32)/2.0) * PI * 2.0 * (lfo_freq / (sr as f32))).cos();
            let del = lfo + center_del;
            processed = process_sample(sample, &mut left_buffer, del, dryWet);
        }else{
            lfo = depth*((((i-1)as f32)/2.0) * PI * 2.0 * (lfo_freq / (sr as f32))).cos();
            let del = lfo + center_del;
            processed = process_sample(sample, &mut right_buffer, del, dryWet);
        }
        writer.write_sample(processed);
        //write!(out, "{}{}", sample, if i % channels as usize == (channels - 1).into() { "\n" } else { " " }).unwrap();
    }
}
fn process_sample(sample: f32, buffer: &mut RingBuffer<f32>, delay: f32, dry_wet: f32)->f32{
    //let output = buffer.get_frac((buffer.get_write_index()as f32-delay+buffer.capacity()as f32)%buffer.capacity()as f32) * feedback + sample;
    buffer.push(sample);
    let output = buffer.pop_frac(delay)*dry_wet + sample*(1.0-dry_wet);
    output

}
