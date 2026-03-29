use std::{i32::MAX, io::{self, Read}, process::exit};

use num_complex::Complex;

fn main() {
    let mut devices = soapysdr::enumerate("").expect("Devices could not be retrieved.");
    let device_count = devices.iter().len();
    if device_count <= 0 {
        println!("No compatible devices were detected. Terminating...");
        exit(1);
    }
    for (i, device) in devices.iter().enumerate() {
        println!("{}: {}", i, device);
    }
    println!("Enter device # [0{}]", (if devices.len() > 1 { format!("-{}", devices.len() - 1) } else { "".to_string() }));
    let mut device_input = String::new();
    io::stdin()
    .read_line(&mut device_input)
    .expect("Failed to read device input");
    let selected_device_idx: usize = device_input.trim().parse::<usize>().expect("Input not a number");
    let selected_device_args = devices.remove(selected_device_idx);
    let device = soapysdr::Device::new(selected_device_args).unwrap();
    let direction = soapysdr::Direction::Rx;
    let channel = 0;
    device.set_sample_rate(direction, channel, 2_560_000.0).expect("Device could not successfully set sample rate");
    device.set_frequency(direction, channel, 97_900_000.0, "").expect("Device could not successfully set frequency");
    device.set_gain(direction, channel,20.0f64).expect("Device could not successfully set gain");
    let formats = device.stream_formats(direction, channel).unwrap();
    println!("Supported formats: {:?}", formats);
    let mut rx_stream = device.rx_stream::<Complex<f32>>(&[channel]).expect("Device could not successfully create Rx Stream");
    rx_stream.activate(None).expect("Rx Stream could not successfully activate");
    // receive samples
    let mut buffer = vec![Complex::new(0.0f32, 0.0f32); 8192];
    let mut amp = 0.0f32;
    let amp_threshold = 0.01f32;
    loop {
        let samples_read = rx_stream.read(&mut [buffer.as_mut_slice()], 1_000_000).expect("Rx Stream could not successfully read");
        // loop over buffer and search for preamble
        for i in 0..samples_read {
            amp = buffer[i].norm();
            if  amp > amp_threshold {
                println!("Possible start of preamble: {}", amp);
            }
        }
    }
    
    // rx_stream.deactivate(None).expect("Rx Stream could not successfully deactivate");
}
