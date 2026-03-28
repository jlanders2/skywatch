use std::io::{self, Read};

use num_complex::Complex;

fn main() {
    let devices = soapysdr::enumerate("");
    let devices_list = devices.as_ref().unwrap();
    for (i, device) in devices_list.iter().enumerate() {
        println!("{}: {}", i, device);
    }
    println!("Enter device # [0{}]", (if devices_list.len() > 1 { format!("-{}", devices_list.len() - 1) } else { "".to_string() }));
    let mut device_input = String::new();
    io::stdin()
    .read_line(&mut device_input)
    .expect("Failed to read device input");
    let selected_device_idx: usize = device_input.trim().parse::<usize>().expect("Input not a number");
    let selected_device_args = devices.unwrap().remove(selected_device_idx);
    let device = soapysdr::Device::new(selected_device_args).unwrap();
    let direction = soapysdr::Direction::Rx;
    let channel = 0;
    device.set_sample_rate(direction, channel, 2_400_000.0).expect("Device could not successfully set sample rate");
    device.set_frequency(direction, channel, 1_090_000_000.0, "").expect("Device could not successfully set frequency");
    let mut rx_stream = device.rx_stream::<Complex<f32>>(&[channel]).expect("Device could not successfully create Rx Stream");
    rx_stream.activate(None).expect("Rx Stream could not successfully activate");
    // receive samples
    
    rx_stream.deactivate(None).expect("Rx Stream could not successfully deactivate");
}
