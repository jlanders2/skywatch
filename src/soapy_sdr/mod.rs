use num_complex::Complex;
use std::io;

use crate::sdr::*;
use soapysdr::{Device, Direction, RxStream};

pub struct SoapySdr;

impl Sdr for SoapySdr {
    fn create_device(&self) -> Result<Box<dyn SdrDevice>, SdrError> {
        let mut devices = soapysdr::enumerate("").expect("Devices were retrieved.");
        let device_count = devices.iter().len();
        if device_count <= 0 {
            panic!("No compatible devices were detected. Terminating...");
        }
        for (i, device) in devices.iter().enumerate() {
            println!("{}: {}", i, device);
        }
        println!(
            "Enter device # [0{}]",
            (if devices.len() > 1 {
                format!("-{}", devices.len() - 1)
            } else {
                "".to_string()
            })
        );
        let mut device_input = String::new();
        io::stdin()
            .read_line(&mut device_input)
            .expect("Successfully read device input");
        let selected_device_idx: usize = device_input
            .trim()
            .parse::<usize>()
            .expect("Input is a number");
        let selected_device_args = devices.remove(selected_device_idx);
        let soapysdr_device = SoapySdrDevice {
            device: soapysdr::Device::new(selected_device_args).unwrap(),
            direction: Direction::Rx,
            channel: 0,
        };

        return Ok(Box::new(soapysdr_device));
    }
}

pub struct SoapySdrDevice {
    device: Device,
    direction: Direction,
    channel: usize,
}

impl SdrDevice for SoapySdrDevice {
    fn set_direction(&mut self, direction: SdrDirection) -> Result<(), SdrError> {
        if direction == SdrDirection::Transmit {
            panic!("Transmission is not currently supported");
        }
        match direction {
            SdrDirection::Receive => self.direction = Direction::Rx,
            SdrDirection::Transmit => self.direction = Direction::Tx,
        }

        return Ok(());
    }

    fn set_channel(&mut self, channel: usize) -> Result<(), SdrError> {
        self.channel = channel;

        return Ok(());
    }

    fn set_sample_rate(&mut self, sample_rate: f64) -> Result<(), SdrError> {
        self.device
            .set_sample_rate(self.direction, self.channel, sample_rate)
            .expect("Successfully set sample rate for device");

        return Ok(());
    }

    fn set_frequency(&mut self, frequency: f64) -> Result<(), SdrError> {
        self.device
            .set_frequency(self.direction, self.channel, frequency, "")
            .expect("Successfully set frequency for device");

        return Ok(());
    }

    fn set_gain(&mut self, gain: f64) -> Result<(), SdrError> {
        self.device
            .set_gain(self.direction, self.channel, gain)
            .expect("Successfully set gain for device");

        return Ok(());
    }

    fn get_stream(&self) -> Result<Box<dyn SdrStream>, SdrError> {
        let mut rx_stream = self
            .device
            .rx_stream::<Complex<f32>>(&[self.channel])
            .expect("Successfully created Rx stream for device");
        rx_stream
            .activate(None)
            .expect("Successfully activated stream");

        let soapysdr_stream = SoapySdrStream {
            direction: self.direction,
            // TODO - Magic number buffer size 1MB
            buffer: vec![Complex::new(0.0f32, 0.0f32); 1_048_576],
            stream: rx_stream,
        };

        return Ok(Box::new(soapysdr_stream));
    }
}

pub struct SoapySdrStream {
    direction: Direction,
    buffer: Vec<Complex<f32>>,
    stream: RxStream<Complex<f32>>,
}

impl SdrStream for SoapySdrStream {
    fn read(&mut self) -> Result<Vec<f32>, SdrError> {
        if self.direction != Direction::Rx {
            panic!("Attempted to read in when direction set to transmission");
        }
        let samples_read = self
            .stream
            .read(&mut [self.buffer.as_mut_slice()], 1_000_000)
            .expect("Successfully read from Rx Stream");
        // Convert samples from raw I/Q data to scalar values; expensive, so do once here each loop
        // I believe norm_sqrt is supposed to be more efficient if ever needed
        let scalars: Vec<f32> = self.buffer[..samples_read]
            .iter()
            .map(|s| s.norm())
            .collect();

        return Ok(scalars);
    }
}
