use std::{error::Error, fmt};

#[derive(Debug)]
pub struct SdrError;

impl Error for SdrError {}

impl fmt::Display for SdrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SdrError - Bad error, not descriptive")
    }
}

#[derive(PartialEq)]
pub enum SdrDirection {
    Transmit,
    Receive
}

pub trait Sdr {
    fn create_device(&self) -> Result<Box<dyn SdrDevice>, SdrError>;
}

pub trait SdrDevice {
    fn set_direction(&mut self, direction: SdrDirection) -> Result<(), SdrError>;
    fn set_channel(&mut self, channel: usize) -> Result<(), SdrError>;
    fn set_sample_rate(&mut self, sample_rate: f64) -> Result<(), SdrError>;
    fn set_frequency(&mut self, frequency: f64) -> Result<(), SdrError>;
    fn set_gain(&mut self, gain: f64) -> Result<(), SdrError>;
    fn get_stream(&self) -> Result<Box<dyn SdrStream>, SdrError>;
}

pub trait SdrStream {
    fn read(&mut self) -> Result<Vec<f32>, SdrError>;
}