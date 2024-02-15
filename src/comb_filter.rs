
mod ring_buffer;
use ring_buffer::RingBuffer;

pub struct CombFilter {
    // TODO: your code here
    buffers : Vec<RingBuffer::<f32>>,
    gain : f32,
    delay : f32,
    filterType : FilterType,
    numChannels : usize,
    last_sample : f32,
    max_delay: f32,
    sample_rate : f32
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
        let max_delay_ = (max_delay_secs * sample_rate_hz) as usize;
        let buffers_ = (0..num_channels)
            .map(|_| RingBuffer::<f32>::new(max_delay_ + 1))
            .collect();
        CombFilter{filterType: filter_type, 
                    buffers: buffers_,
                    gain: 0.0,
                    delay: 0.0,
                    numChannels : num_channels,
                    last_sample : 0.0,
                    max_delay : max_delay_ as f32,
                    sample_rate : sample_rate_hz
                    }
        
    }

    pub fn reset(&mut self) {
        //todo!("implement")
        let index = self.find_read_index((self.delay*self.sample_rate) as i32);
        for buffer in self.buffers.iter_mut(){
            buffer.reset();
            buffer.set_read_index(index);
        }
    }

    pub fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]) {
        //todo!("implement");
        for  (out_chunk,in_chunk) in output.iter_mut().zip(input)
        {   
            for (channel_id,(outVal, inVal)) in out_chunk.iter_mut().zip(in_chunk.iter()).enumerate()
            {
                self.calculate_filter(inVal, outVal, channel_id);
            }
        }
    }

    pub fn set_param(&mut self, param: FilterParam, value: f32) -> Result<(), Error> {
        //todo!("implement")
        use FilterParam::*;
        match param
        {
            Delay => {
                match self.filterType {
                    FilterType::FIR => {
                        if value > self.max_delay || value < 0.0{
                            println!("Delay must be greater than 0 and less than or equal to max delay size!");
                            return Err(Error::InvalidValue { param, value });
                        }
                    },
                    FilterType::IIR => {
                        if value <= 0.0 {
                            println!("Delay cannot be zero or less than zero in an IIR filter!");
                            return Err(Error::InvalidValue { param, value });
                        }
                        if value > self.max_delay{
                            println!("Delay must be less than or equal to max delay size!");
                            return Err(Error::InvalidValue { param, value });
                        }
                    }
                }
                self.delay = value;
                let index = self.find_read_index((value*self.sample_rate) as i32);
                for buffer in self.buffers.iter_mut(){
                    buffer.set_read_index(index);
                }
            },
            Gain => {
                if value < 0.0 || value > 1.0{
                    println!("Gain must be between 0.0 and 1.0!");
                    return Err(Error::InvalidValue { param, value });
                }
                self.gain = value
            },
            _ => {
                println!("Invalid Parameter!");
                return Err(Error::InvalidValue { param, value });
            }
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
    fn find_read_index(&mut self, del_in_samps: i32) -> usize {
        let write_index = self.buffers[0].get_write_index() as i32;
        let mut calc_index = if write_index >= del_in_samps {
            write_index - del_in_samps
        } else {
            write_index - del_in_samps + self.buffers[0].capacity() as i32
        };
        if calc_index >= self.max_delay as i32 {
            calc_index -= self.buffers[0].capacity() as i32;
        }
        calc_index as usize
    }
    fn calculate_filter(&mut self, inVal: &f32, outVal: &mut f32, channel: usize)
    {
        use FilterType::*;
        match self.filterType
                {
                    FIR => {
                        self.buffers[channel].push(*inVal);
                        *outVal = *inVal + (self.gain * self.buffers[channel].pop());
                    }
                    IIR => {
                        self.last_sample = self.buffers[channel].pop();
                        *outVal = *inVal + self.last_sample * self.gain;
                        self.buffers[channel].push(*outVal);
                    }
                }
    }

    // TODO: feel free to define other functions for your own use
}




