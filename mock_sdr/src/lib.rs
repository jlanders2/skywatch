use sdr_core::*;

pub struct MockSdr;

impl Sdr for MockSdr {
    fn create_device(&self) -> Result<Box<dyn SdrDevice>, SdrError> {
        todo!()
    }
}

pub struct MockSdrDevice { }

impl SdrDevice for MockSdrDevice {
    fn set_direction(&mut self, _direction: SdrDirection) -> Result<(), SdrError> {
        todo!()
    }

    fn set_channel(&mut self, _channel: usize) -> Result<(), SdrError> {
        todo!()
    }

    fn set_sample_rate(&mut self, _sample_rate: f64) -> Result<(), SdrError> {
        todo!()
    }

    fn set_frequency(&mut self, _frequency: f64) -> Result<(), SdrError> {
        todo!()
    }

    fn set_gain(&mut self, _gain: f64) -> Result<(), SdrError> {
        todo!()
    }

    fn get_stream(&self) -> Result<Box<dyn SdrStream>, SdrError> {
        todo!()
    }
}

pub struct MockSdrStream;

impl SdrStream for MockSdrStream {
    fn read(&mut self) -> Result<Vec<f32>, SdrError> {
        todo!()
    }
}
