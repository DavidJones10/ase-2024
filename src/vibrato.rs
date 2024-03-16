use core::num;
use std::sync::mpsc::channel;

use crate::ring_buffer::RingBuffer;

use crate::lfo::LFO;

/// Clamps values between min and max
fn fclamp(x: f32, min_val: f32, max_val: f32) -> f32 {
    if x < min_val {
        min_val
    } else if x > max_val {
        max_val
    } else {
        x
    }
}
/// Asserts that inputs are within an epsilon away from each other
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

/// I chose to have both a sample-by-sample processing scheme and a block-based processing scheme
/// because I think it is easier to use sample-by-sample in most cases.
/// I also chose to make the block processing only take in a mutable slice of slices because that is what it would be in a plugin.
/// For the setters, I chose to have them separate rather than just having one because
/// if you're using a modern editor, it is easier to type "myVibrato." to see the available methods than it is
/// to type "myVibrato.set_param(vibrato:: VibratoParams::Delay, value)" every time you want to change a parameter.  
/// The core of my design is to make it easy to use this object without ever having to look at its source code.
pub struct Vibrato{
    center_delay: f32,
    max_delay: f32,
    dry_wet: f32,
    buffers: Vec<RingBuffer::<f32>>,
    num_channels: usize,
    sample_rate: f32,
    lfo: LFO,
    lfo_width: f32,
    lfo_rate: f32,
}

///Delay, Rate, Width, DryWet
pub enum  VibratoParams{
    Delay,
    Rate,
    Width,
    DryWet
}

impl Vibrato{
    /// Creates a new Vibrato object
    pub fn new(num_channels_: usize, sample_rate_: f32, max_delay_ms: f32)->Self{
        let center_del = 0.005000  * sample_rate_; //  5 ms starting
        let max_del = (max_delay_ms/1000.0) * sample_rate_;
        let buffs_: Vec<RingBuffer<f32>> = (0..num_channels_)
            .map(|_| RingBuffer::<f32>::new(((max_delay_ms/1000.0)*sample_rate_+1.0) as usize))
            .collect();
        let lfo_ = LFO::new(sample_rate_, crate::lfo::WaveType::Sine
                                    ,8.0, center_del* sample_rate_*1.0);
         Vibrato{
            center_delay:  center_del, // initialized at 5 ms
            max_delay: max_del,
            dry_wet: 1.0, // initialized at 100%
            buffers: buffs_,
            num_channels: num_channels_,
            sample_rate: sample_rate_,
            lfo: lfo_,
            lfo_width: 1.0, 
            lfo_rate: 8.0,
        }
    }
    /// Processes vibrato in a blocking context.
    /// 
    /// Requrires that the read and write buffer be the same.
    pub fn processBlock(&mut self, buff: &mut [ &mut [f32]]){
        for (chunk_id,chunk)in buff.iter_mut().enumerate(){
            for (channel_id,sample) in chunk.iter_mut().enumerate(){
                let og_value = *sample;
                *sample = self.process(og_value, channel_id);
            }
        }
    }

    ///Takes input sample and channel id and 
    /// outputs processed vibrato sample
    pub fn process(&mut self, input_sample: f32, channel_id: usize)->f32{
        assert!(channel_id < self.num_channels);
        self.buffers[channel_id].push(input_sample);
        let del = self.lfo.process() + self.center_delay+1.0;
        self.buffers[channel_id].pop_frac(del)*self.dry_wet + input_sample *(1.0-self.dry_wet)


    }
    /// Sets vibrato width on a scale from 0-1 
    /// lower values will sound more natural 
    pub fn set_width(&mut self, mut width: f32){
        self.lfo_width = fclamp(width, 0.0, 1.0);
        let width_samps = self.lfo_width * self.center_delay;
        self.lfo.set_amplitude(width_samps);
    }
    /// Sets delay in ms (4-12)
    pub fn set_delay(&mut self, delay: f32){
        self.center_delay = fclamp(delay, 4.0, 12.0) /1000.0 * self.sample_rate;
    }
    /// Sets lfo rate in Hz (2-20)
    pub fn set_rate(&mut self, rate: f32){
        self.lfo_rate = fclamp(rate, 2.0, 20.0);
        self.lfo.set_frequency(self.lfo_rate);
    }
    /// Sets dry/wet (0-1)    
    pub fn set_dry_wet(&mut self, dryWet: f32){
        self.dry_wet = fclamp(dryWet, 0.0, 1.0);
    }
    ///Resets internal lfo and ring buffers
    pub fn reset(&mut self){
        for i in 0..self.num_channels{
            self.buffers[i].reset();
        }
        self.lfo.reset();
    }
    /// Returns value of a given parameter
    pub fn get_param(&mut self, param:  VibratoParams)->f32{
        match param {
             VibratoParams::Delay=>self.center_delay*1000.0 / self.sample_rate,
             VibratoParams::Rate=> self.lfo.get_param(crate::lfo::LfoParams::Frequency),
             VibratoParams::Width=>self.lfo_width,
             VibratoParams::DryWet=>self.dry_wet
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;   
    #[test]
    fn test_0_mod_amp(){
        let mut vibe = Vibrato::new(1,6000.0,10.0);
        vibe.set_delay(10.0); // .010 * 6000 = 60
        vibe.set_width(0.0);
        vibe.set_dry_wet(1.0);
        vibe.set_rate(5.0);
        for i in 0..200{
            let processed = vibe.process(i as f32, 0);
            //println!("processed: {}, i: {}", processed, i);
            if i >=60{
                assert_eq!(processed,i as f32 - 60.0);
            }
        }
        println!("Test passed! Output = delayed input!");
    }
    #[test]
    fn test_dc(){
        let mut vibe = Vibrato::new(1,6000.0,30.0);
        vibe.set_delay(10.0); // .010 * 6000 = 60
        vibe.set_width(0.5);
        vibe.set_dry_wet(1.0);
        vibe.set_rate(2.0);
        let arr: Vec<f32> = vec![5.0;200];
        for i in 0..200{
            let processed = vibe.process(arr[i], 0);
            //println!("processed: {}, i: {}", processed, i);
            if i > 65{ // transition sample will likely be between 0 and 5 so just check after it
                assert_eq!(processed, 5.0);
            }
        }
        println!("Test passed! DC input = delayed DC output!");
    }
    #[test]
    // Test for varying block sizes
    fn test_process_blocking() {
        // Set up parameters for Vibrato
        let num_channels = 2;
        let sample_rate = 44100.0;
        let max_delay_ms = 20.0;
        let input_samples: Vec<f32> = (0..sample_rate as usize *num_channels).map(|i| i as f32).collect(); 

        let mut vibrato = Vibrato::new(num_channels, sample_rate, max_delay_ms);
        vibrato.set_delay(10.0);
        vibrato.set_width(0.0);
        vibrato.set_dry_wet(1.0);
 
        let length: usize = sample_rate as usize;

        let block_sizes = [2, 4, 8, 512, 1024];

        for &block_size in &block_sizes {
            vibrato.reset();

            let mut samples = input_samples.clone();
            let mut og_samples = samples.clone();

            let mut buffer: Vec<_> = samples.chunks_mut(num_channels).collect();

            for chunk in buffer.chunks_mut(block_size / num_channels) {
                vibrato.processBlock(&mut chunk[..]);
            }

            let mut count = 0;
            for chunk in buffer{
                for sample in chunk{
                    if count < 882 {
                        assert_eq!(*sample,0.0); 
                    }else{
                        assert_eq!(*sample,input_samples[count-882]);
                    }
                    count += 1;
                }
            }
        }
        println!("Test passed! Output = Delayed input for each block size");
    }
    #[test]
    // test with more than 2 channels
    fn channels_test(){
        let channels = 5;
        let sr = 6000.0;
        let delay_in_ms = 10.0;
        let mut vibe = Vibrato::new(channels,sr,30.0);
        vibe.set_delay(10.0); // .010 * 6000 = 60
        vibe.set_width(0.0); // non-modulating delay
        vibe.set_dry_wet(1.0);
        vibe.set_rate(10.0);
        for i in 0..900{
            let processed = vibe.process(i as f32, i %channels);
            if i > (channels as f32 * sr * delay_in_ms / 1000.0) as usize{
                assert_eq!(processed,i as f32 - 300.0);
            }
        }
        println!("Test passed! Output = delayed input!");
    }
    #[test]
    fn zero_test(){
        let channels = 5;
        let sr = 6000.0;
        let delay_in_ms = 10.0;
        let mut vibe = Vibrato::new(channels,sr,30.0);
        vibe.set_delay(delay_in_ms); // .010 * 6000 = 60
        vibe.set_width(0.0); // non-modulating delay
        vibe.set_dry_wet(1.0);
        vibe.set_rate(10.0);
        let vect = vec![0.0; 900];
        for i in 0..900{
            let processed = vibe.process(vect[i], i %channels);
            assert_eq!(processed, 0.0);
        }
        println!("Test passed! Output = Input = 0!");
    }
    #[test]
    fn test_param_values(){
        
        let channels = 1;
        let sr = 6000.0;
        let delay_in_ms = 0.0;
        let mut vibe = Vibrato::new(channels,sr,30.0);
        vibe.set_delay(delay_in_ms); // below 0 will default to 4 ms so delay of 24 samples plus mod will be 34 samples
        vibe.set_width(1.0); 
        vibe.set_dry_wet(1.0);
        vibe.set_rate(10.0);
        for i in 0..900{
            let processed = vibe.process(i as f32, 0);
            if i > 33{
               assert!(processed !=0.0);
            }
        }
        vibe.reset();
        vibe.set_dry_wet(0.5);
        vibe.set_rate(0.0);
        vibe.set_width(0.0);
        let ones = vec![1.0;900];
        for i in 0..900{
            let processed = vibe.process(ones[i], 0);
            if i <= 25{
               assert_close!(0.5, processed, 0.0001);
            }else{
                assert_close!(1.0, processed, 0.00001)
            }
        }
        assert_eq!(vibe.get_param(VibratoParams::DryWet),0.5);
        assert_eq!(vibe.get_param(VibratoParams::Width),0.0);
        assert_eq!(vibe.get_param(VibratoParams::Rate),2.0); // 2.0 is the minimum value
        assert_close!(vibe.get_param(VibratoParams::Delay),4.0, 0.0001);// Will be the same that was set above

        println!("Test passed! Output = Input = 0!");
        
    }
}