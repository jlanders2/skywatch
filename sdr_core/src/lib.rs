use std::{error::Error, fmt};

#[derive(Debug)]
struct SdrError;

impl Error for SdrError {}

impl fmt::Display for SdrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SdrError - Bad error, not descriptive")
    }
}

enum SdrDirection {
    Transmit,
    Receive
}

trait Sdr {
    fn create_device() -> Result<SdrDevice, SdrError>;
}

trait SdrDevice {
    fn set_direction(direction: SdrDirection) -> Result<(), SdrError>;
    fn set_channel(channel: usize) -> Result<(), SdrError>;
    fn set_sample_rate(sample_rate: f64) -> Result<(), SdrError>;
    fn set_frequency(frequency: f64) -> Result<(), SdrError>;
    fn set_gain(gain: f64) -> Result<(), SdrError>;
    fn get_stream() -> Result<SdrStream, SdrError>;
}

trait SdrStream {
    fn read() -> Result<Vec<f32>, SdrError>;
}