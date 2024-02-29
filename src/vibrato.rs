use core::num;

use crate::ring_buffer::RingBuffer;

use crate::lfo::LFO;

fn fclamp(x: f32, min_val: f32, max_val: f32) -> f32 {
    if x < min_val {
        min_val
    } else if x > max_val {
        max_val
    } else {
        x
    }
}
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

/// I chose a sample-by-sample processing scheme rather than an input block scheme
/// because I think it is easier to use and much less complicated than a slice of a slice.
/// It also would make it easier to use this object in a chain of effects.
/// For the setters, I chose to have them separate rather than just having one because
/// if you're using a modern editor, it is easier to type "myVibrato." to see the available methods than it is
/// to type "myVibrato.set_param(vibrato::Params::Delay, value)" every time. 
/// The core of my design is to make it easy to use this object without ever having to look at its source.
pub struct Vibrato{
    center_delay: f32,
    max_delay: f32,
    dry_wet: f32,
    buffers: Vec<RingBuffer::<f32>>,
    num_channels: usize,
    sample_rate: f32,
    lfo: LFO,
    lfo_width: f32,
}

pub enum Params{
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
        let buffs_ = (0..num_channels_)
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
        }
    }
    
    ///Takes input sample and channel id and 
    /// outputs processed vibrato sample
    pub fn process(&mut self, input_sample: f32, channel_id: usize)->f32{
        assert!(channel_id < self.num_channels);
        self.buffers[channel_id].push(input_sample);
        let del = self.lfo.pop() + self.center_delay+1.0;
        self.buffers[channel_id].pop_frac(del)*self.dry_wet + input_sample *(1.0-self.dry_wet)

    }
    /// Sets vibrato width on a scale from 0-1 
    /// lower values will sound more natural 
    pub fn set_width(&mut self, mut width: f32){
        let width_samps = fclamp(width,0.0,1.0) * self.center_delay;
        self.lfo.set_amplitude(width_samps);
    }
    /// Sets delay in ms (4-12)
    pub fn set_delay(&mut self, delay: f32){
        self.center_delay = fclamp(delay, 4.0, 12.0) /1000.0 * self.sample_rate;
    }
    /// Sets lfo rate in Hz (2-20)
    pub fn set_rate(&mut self, rate: f32){
        let rate_ = fclamp(rate, 2.0, 20.0);
        self.lfo.set_frequency(rate_);
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
    pub fn get_param(&mut self, param: Params)->f32{
        match param {
            Params::Delay=>self.center_delay*1000.0 / self.sample_rate,
            Params::Rate=> self.lfo.get_param(crate::lfo::LfoParams::Frequency),
            Params::Width=>self.lfo_width,
            Params::DryWet=>self.dry_wet
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
    // Since my vibrato works on a sample-by-sample basis, it works with any block size,
    // so here's a test with several channels instead
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
            //println!("processed: {}, i: {}", processed, i);
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
        vibe.set_delay(delay_in_ms); // will default to 4 ms so delay of 24 samples plus mod will be 34 samples
        vibe.set_width(1.0); 
        vibe.set_dry_wet(1.0);
        vibe.set_rate(10.0);
        for i in 0..900{
            let processed = vibe.process(i as f32, 0);
            //println!("processed: {}, i: {}", processed, i);
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
            //println!("processed: {}, i: {}", processed, i);
            if i <= 25{
               assert_close!(0.5, processed, 0.0001);
            }else{
                assert_close!(1.0, processed, 0.00001)
            }
        }

        println!("Test passed! Output = Input = 0!");
        
    }
}