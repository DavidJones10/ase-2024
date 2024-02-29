use core::num;
use std::{f32::consts::PI};
macro_rules! assert_close {
    ($left:expr, $right:expr, $epsilon:expr) => {{
        let (left, right, epsilon) = ($left, $right, $epsilon);
        assert!(
            (left - right).abs() <= epsilon,
            "{} is not close to {} within an epsilon of {}",
            left,
            right,
            epsilon
        );
    }};
}

pub struct LFO{
    frequency: f32,
    amplitude: f32,
    waveType: WaveType,
    buffer: Vec<f32>,
    min_frequency: f32,
    sample_rate: f32,
    read_ptr: usize,
    num_samples: usize
}
pub enum LfoParams{
    Frequency,
    Amplitude,
}

pub enum WaveType{
    Sine,
    Tri,
    Saw,
    Square,
}

impl LFO{
    pub fn new(sample_rate_: f32, waveType_: WaveType, minFrequency: f32, frequency_: f32, amplitude_: f32)->Self{
        assert!(frequency_ >= minFrequency);
        let buffer_size = 2*(sample_rate_/minFrequency) as usize;// maximum size of array for minimum frequency
        let buff = vec![0.0;buffer_size];
        let mut lfo = LFO{
            waveType: waveType_,
            frequency: frequency_,
            amplitude: amplitude_,
            min_frequency: minFrequency,
            sample_rate: sample_rate_,
            buffer: buff,
            read_ptr: 0,
            num_samples: buffer_size,
        };
        lfo.set_frequency(frequency_);
        lfo.set_amplitude(amplitude_); 
        lfo.fill_waveform();
        lfo
    }
    pub fn pop(&mut self)->f32{
        let out = self.buffer[self.read_ptr]*self.amplitude;
        self.read_ptr = (self.read_ptr + 1) % self.num_samples;
        out
    }
    /** Sets LFO frequency and calculates new wavetable
     */
    pub fn set_frequency(&mut self, frequency:f32){
        if frequency == 0.0{
            self.frequency = frequency;
            self.num_samples = 0;
        }else if frequency > self.min_frequency{
            self.frequency = frequency;
            self.num_samples = 2* (self.sample_rate / self.frequency) as usize;
        }else{
            self.frequency = self.min_frequency;
            self.num_samples = 2* (self.sample_rate / self.frequency) as usize;
        }
        self.fill_waveform();
    }
    /** Sets amplitude of the LFO output
     */
    pub fn set_amplitude(&mut self, amplitude: f32){
        self.amplitude = amplitude;
    }
    /** Sets LFO WaveType
     */
    pub fn set_wave_type(&mut self, waveType_: WaveType){
        self.waveType = waveType_;
        self.fill_waveform();
        self.read_ptr = 0;
    }
    pub fn reset(&mut self){
        self.fill_waveform();
        self.read_ptr = 0;
    }
    pub fn get_param(&mut self, param: LfoParams)->f32{
        match param{
            LfoParams::Frequency=> self.frequency,
            LfoParams::Amplitude=>self.amplitude,
        }
    }
    /** Fills internal wavetable buffer with current waveform
     */
    fn fill_waveform(&mut self){
        let w = 2.0 * PI * self.frequency / self.sample_rate;
        for i in 0..self.num_samples{
            self.buffer[i] = match self.waveType{
                WaveType::Sine => (w * i as f32).sin(),
                WaveType::Tri =>{
                    let slope = 2.0 / self.frequency;
                    let t_mod = (i as f32 / self.sample_rate) % (1.0 / self.frequency);
                    if t_mod < slope{
                        t_mod * slope
                    }else{
                        2.0 - t_mod * slope
                    }
                },
                WaveType::Saw => {
                    let t = (i as f32 / self.sample_rate);
                    2.0 * (t * self.frequency - (t * self.frequency).floor())
                },
                WaveType::Square =>{
                    let t = (i as f32 / self.sample_rate);
                    if (t * self.frequency).sin() >= 0.0{
                        1.0
                    }else{
                        -1.0
                    }
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;   
    #[test]
    fn test_lfo(){
        let mut lfo = LFO::new(44100.0, WaveType::Sine, 
                                0.001,8.0, 1.0);
        for i in 0..88200{
            
            let w = 2.0 * PI * 8.0 / 44100.0;
            let sine = (i as f32 * w).sin();
            let popped = lfo.pop();
            //println!("sine: {}, popped: {}", sine, popped);
            assert_close!(sine, popped, 0.01);
        }
    }
}

