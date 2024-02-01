use core::num;
use std::io::prelude;

pub struct CombFilter {
    // TODO: your code here
    buffer : Vec<f32>,
    gain : f32,
    delay : f32,
    filterType : FilterType,
    numChannels : usize,
    last_sample : f32
}

#[derive(Debug, Clone, Copy)]
pub enum FilterType {
    FIR,
    IIR,
}

#[derive(Debug, Clone, Copy)]
pub enum FilterParam {
    Gain,
    Delay,
}

#[derive(Debug, Clone)]
pub enum Error {
    InvalidValue { param: FilterParam, value: f32 }
}

impl CombFilter {
    pub fn new(filter_type: FilterType, max_delay_secs: f32, sample_rate_hz: f32, num_channels: usize) -> Self {
        //todo!("implement");
        let delaySize = (max_delay_secs * sample_rate_hz) as usize;
        CombFilter{filterType: filter_type, 
                    buffer: vec![0.0;(max_delay_secs*sample_rate_hz)as usize],
                    gain: 1.0,
                    delay: 0.0,
                    numChannels : num_channels,
                    last_sample : 0.0
                    }
        
    }

    pub fn reset(&mut self) {
        //todo!("implement")
        self.buffer.fill(0.0);
    }

    pub fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]) {
        //todo!("implement");
        use FilterType::*;
        for (out_slice,in_slice) in output.iter_mut().zip(input)
        {
            for (outVal, inVal) in out_slice.iter_mut().zip(in_slice.iter())
            {
                match self.filterType
                {
                    FIR => {
                        self.buffer.push(*inVal);
                        *outVal = *inVal + (self.gain * self.buffer.get((self.delay-1.0) as usize).unwrap());
                        self.buffer.pop();
                    }
                    IIR => {
                        *outVal = *inVal + self.last_sample * self.gain;
                        self.buffer.push(*outVal);
                        self.last_sample = *self.buffer.get((self.delay-1.0) as usize).unwrap();
                        self.buffer.pop();
                    }
                }
            }
        }
    }

    pub fn set_param(&mut self, param: FilterParam, value: f32) -> Result<(), Error> {
        //todo!("implement")
        use FilterParam::*;
        match param
        {
            Delay => self.delay = value,
            Gain => self.gain = value,
            _ => println!("Invalid Parameter!")
        }
        Ok(())
    }

    pub fn get_param(&self, param: FilterParam) -> f32 {
        //todo!("implement");
        use FilterParam::*;
        match param
        {
            Delay => return self.delay,
            Gain => return self.gain
        }
    }

    // TODO: feel free to define other functions for your own use
}

// TODO: feel free to define other types (here or in other modules) for your own use
