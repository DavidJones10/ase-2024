use core::num;
use std::{f32::consts::PI};

use crate::ring_buffer::RingBuffer;
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
/// A simple wavetable LFO generator and playback object
pub struct LFO{
    frequency: f32,
    amplitude: f32,
    waveType: WaveType,
    buffer: RingBuffer<f32>,
    sample_rate: f32,
    phase_offset: f32
}
/// Frequency and Amplitude
pub enum LfoParams{
    Frequency,
    Amplitude,
}
/// Sine, Tri, Saw, Square
pub enum WaveType{
    Sine,
    Tri,
    Saw,
    Square,
}

impl LFO{
    /// Creates a new LFO
    pub fn new(sample_rate_: f32, waveType_: WaveType, frequency_: f32, amplitude_: f32)->Self{
        let buff = RingBuffer::<f32>::new(sample_rate_ as usize);
        let mut lfo = LFO{
            waveType: waveType_,
            frequency: frequency_,
            amplitude: amplitude_,
            sample_rate: sample_rate_,
            buffer: buff,
            phase_offset: 0.0
        };
        lfo.set_frequency(frequency_);
        lfo.set_amplitude(amplitude_); 
        lfo.fill_waveform();
        lfo
    }
    /// Reads one sample at a time from the wavetable buffer
    pub fn process(&mut self) -> f32 {
        //dbg!(self.phase_offset);
        let val = self.buffer.get_frac(self.phase_offset)*self.amplitude;
        self.phase_offset += self.frequency;
        if self.phase_offset >= self.buffer.capacity() as f32{
            self.phase_offset -= self.buffer.capacity() as f32; // keeps the offset from going to infinity lol
        }
        val
    }
    /// Gets sample from LFO wavetable at given offset
    pub fn get(&self, phase_offset: f32) -> f32 {
        self.buffer.get_frac(phase_offset) * self.amplitude
    }
    /// Sets LFO frequency and calculates new wavetable
    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }
    /// Sets amplitude of the LFO output
    pub fn set_amplitude(&mut self, amplitude: f32){
        self.amplitude = amplitude;
    }
    /// Sets LFO WaveType
    pub fn set_wave_type(&mut self, waveType_: WaveType){
        self.waveType = waveType_;
        self.fill_waveform();
    }
    /// Refills the wavetable and resets the read pointer
    pub fn reset(&mut self){
        self.phase_offset = 0.0;
        self.fill_waveform();
    }
    /// Returns the value of a given parameter
    pub fn get_param(&mut self, param: LfoParams)->f32{
        match param{
            LfoParams::Frequency=> self.frequency,
            LfoParams::Amplitude=>self.amplitude,
        }
    }
    /// Fills internal wavetable buffer with current waveform
    /// 
    /// Will get called when a new wave_type is set or the LFO is reset
    fn fill_waveform(&mut self){
        let num_samples = self.buffer.capacity();

        for i in 0..num_samples {
            let phase = i as f32 / num_samples as f32;
            let val = match self.waveType {
                WaveType::Sine => (2.0 * PI * phase).sin(),
                WaveType::Tri => {
                    let x = phase % 1.0;
                    if x < 0.25 {
                        4.0 * x
                    } else if x < 0.75 {
                        2.0 - 4.0 * x
                    } else {
                        4.0 * x - 4.0
                    }
                }
                WaveType::Saw => 2.0 * phase - 1.0,
                WaveType::Square => {
                    if phase < 0.5 {
                        1.0
                    } else {
                        -1.0
                    }
                }
            };
            self.buffer.push(val);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;   
    #[test]
    fn test_lfo(){
        let mut lfo = LFO::new(32.0, WaveType::Sine,0.1, 1.0);
        for i in 0..64{
            
            let w = 2.0 * PI * 0.1 / 32.0;
            let sine = (i as f32 * w).sin();
            let popped = lfo.process();
            //println!("sine: {}, popped: {}, i: {}", sine, popped, i);
            assert_close!(sine, popped, 0.01);
        }
        lfo.reset();
        lfo.set_frequency(30.0);
        for i in 0..64{
            
            let w = 2.0 * PI * 30.0 / 32.0;
            let sine = (i as f32 * w).sin();
            let popped = lfo.process();
            //println!("sine: {}, popped: {}, i: {}", sine, popped, i);
            assert_close!(sine, popped, 0.01);
        }
    }
}

