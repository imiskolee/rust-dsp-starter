
use std::f32::consts::PI;
use std::ops::Add;

pub struct Signal {
    pub data : Vec<f32>,
    pub sample_rate: usize,
}


impl Add for Signal {
    type Output = Signal;
    fn add(self, rhs: Self) -> Self::Output {
        let mut new_data:Vec<f32> = Vec::with_capacity(self.data.len());
        for (i,x) in self.data.iter().enumerate() {
            new_data.push(x + rhs.data[i]);
        }
        return Signal {
            data: new_data,
            sample_rate: self.sample_rate,
        }
    }
}

/**
生成正弦信号
**/
pub fn sine(length: usize, freq: f32, sample_rate: usize,amplitude:f32) -> Signal {
    let w = 2.0 * PI * freq / (sample_rate as f32);
    let data = (0..length).map(|i| f32::sin((i as f32) * w) * amplitude ).collect();
    Signal { data, sample_rate }
}





