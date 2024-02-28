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

pub struct Vibrato{
    center_delay: f32,
    max_delay: f32,
    dry_wet: f32,
    buffers: Vec<RingBuffer::<f32>>,
    num_channels: usize,
    sample_rate: f32,
    lfo: LFO,
}
pub enum Param{
    Delay,
    Rate,
    Depth,
    DryWet,
}

impl Vibrato{
    pub fn new(num_channels_: usize, sample_rate_: f32, max_delay_ms: f32)->Self{
        let center_del = 0.005  * sample_rate_; //  5 ms starting
        let max_del = (max_delay_ms/1000.0) * sample_rate_;
        let buffs_ = (0..num_channels_)
            .map(|_| RingBuffer::<f32>::new(((max_delay_ms/1000.0)*sample_rate_ + 1.0) as usize))
            .collect();
        let lfo_ = LFO::new(sample_rate_, crate::lfo::WaveType::Sine, 
            2.0,8.0, center_del* sample_rate_*0.5);
         Vibrato{
            center_delay:  center_del, // initialized at 5 ms
            max_delay: max_del,
            dry_wet: 0.5, // initialized at 50%
            buffers: buffs_,
            num_channels: num_channels_,
            sample_rate: sample_rate_,
            lfo: lfo_,
        }
    }
    pub fn process(&mut self, input_sample: f32, channel_id: usize)->f32{
        assert!(channel_id < self.num_channels);
        self.buffers[channel_id].push(input_sample);
        let del = self.lfo.pop() + self.center_delay;
        self.buffers[channel_id].pop_frac(del)*self.dry_wet + input_sample *(1.0-self.dry_wet)
    }

    // Sets vibrato width on a scale from 0-1
    pub fn set_width(&mut self, mut width: f32){
        let width_samps = fclamp(width,0.0,1.0) * self.center_delay;
        self.lfo.set_amplitude(width_samps);
    }
    // Sets delay in ms
    pub fn set_delay(&mut self, delay: f32){
        self.center_delay = fclamp(delay, 4.0, 12.0) * 0.001 * self.sample_rate;
    }
    // Sets lfo rate in Hz
    pub fn set_rate(&mut self, rate: f32){
        let rate_ = fclamp(rate, 2.0, 20.0);
        self.lfo.set_frequency(rate_);
    }
    pub fn set_dry_wet(&mut self, dryWet: f32){
        self.dry_wet = fclamp(dryWet, 0.0, 1.0);
    }
}