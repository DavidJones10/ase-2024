use std::{fmt::write, fs::File, io::{BufWriter, Write}, iter::Map};

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}
fn normalize_sample(sample: &i16)->f32
{   
    let floatSample = *sample as f32;
    if floatSample < 0_f32
    {
        return floatSample / 32768_f32;
    }
    floatSample / 32767_f32
}

fn main() {
   show_info();

    // Parse command line arguments
    // First argument is input .wav file, second argument is output text file.
    let args: Vec<String> = std::env::args().collect();
    // TODO: your code here

    // Open the input wave file and determine number of channels
    // TODO: your code here; see `hound::WavReader::open`.
    let mut reader = hound::WavReader::open(&args[1]).unwrap();
    let spec = reader.spec();
    let num_channels = spec.channels;
    print!("Number of channels: {}",num_channels);
    // Read audio data and write it to the output text file (one column per channel)
    // TODO: your code here; we suggest using `hound::WavReader::samples`, `File::create`, and `write!`.
    //       Remember to convert the samples to floating point values and respect the number of channels!
    let output = File::create(&args[2]).expect("Couldn't open the file");
    let mut file_writer = BufWriter::new(output);
    writeln!(file_writer, "Channel 1, Channel 2").expect("Failed to write Channel names");
    for (index,sample) in reader.samples::<i16>().enumerate()
    {
        let normalized = normalize_sample(&sample.unwrap());
        if index % 2 == 0
        {
            write!(file_writer, "{}, ",normalized).expect("Failed to write sample");
        }
        else 
        {
            write!(file_writer,"{}", normalized).expect("Failed to write sample");
            writeln!(file_writer).expect("Failed to write sample");
        }   
    }
}

// i16 -32768 - 32767 to f32 -1. to 1.