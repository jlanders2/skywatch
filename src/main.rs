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
    device.set_sample_rate(direction, channel, 2_000_000.0).expect("Device could not successfully set sample rate");
    device.set_frequency(direction, channel, 1_090_000_000.0, "").expect("Device could not successfully set frequency");
    device.set_gain(direction, channel,40.0f64).expect("Device could not successfully set gain");
    let formats = device.stream_formats(direction, channel).unwrap();
    println!("Supported formats: {:?}", formats);
    let mut rx_stream = device.rx_stream::<Complex<f32>>(&[channel]).expect("Device could not successfully create Rx Stream");
    rx_stream.activate(None).expect("Rx Stream could not successfully activate");
    // receive samples
    let mut buffer = vec![Complex::new(0.0f32, 0.0f32); 1_024_000];
    let threshold = 0.01f32;
    let preamble_pattern = [
        true, 
        false,
        true, 
        false,
        false, 
        false,
        false, 
        true,
        false, 
        true,
        false, 
        false,
        false, 
        false,
        false, 
        false,
    ];
    let df_length: usize = 5; // bits
    loop {
        let samples_read = rx_stream.read(&mut [buffer.as_mut_slice()], 1_000_000).expect("Rx Stream could not successfully read");
        let magnitudes: Vec<f32> = buffer[..samples_read].iter().map(|s|s.norm()).collect();
        // println!("{} - {}", samples_read, magnitudes.len());
        // loop over buffer and search for preamble
        let mut i = 0;
        let mut overflow = false;
        while i < (samples_read - preamble_pattern.len()) {
            let mut preamble_detected = false;
            if magnitudes[i] > threshold {
                for j in 0..preamble_pattern.len() {
                    i = i + j;
                    if i >= samples_read {
                        overflow = true;
                        preamble_detected = false;
                        break;
                    }
                    if ((magnitudes[i] > threshold).ne(&preamble_pattern[j])) {
                        preamble_detected = false;
                        break;
                    } else {
                        preamble_detected = true;
                    }
                }
                if preamble_detected {
                    if i + 10 > samples_read { // 10 samples = 5microseconds = 5bits
                        break;
                    } else {
                        let mut df = 0u8;
                        let df_buffer = &magnitudes[i..i+10];
                        for bit in 0..5 {
                            let first = df_buffer[bit * 2];
                            let second= df_buffer[(bit * 2) + 1];
                            if first > second {
                                df |= 1 << (4 - bit);
                            }
                        }
                        if df == 17 {
                            if i + (27 * 2) >= samples_read { // 27 bits is what's between DF and Message
                                overflow = true;
                            }
                            i = i + (27 * 2);
                            if i + (56 * 2) >= samples_read {
                                overflow = true;
                            }
                            if !overflow {
                                let msg_buffer = &magnitudes[i..i+(56*2)];
                                for bit in 0..(56 * 2) {
                                    let first = msg_buffer[bit * 2];
                                    let second= msg_buffer[(bit * 2) + 1];
                                    if first > second {
                                        df |= 1 << (4 - bit);
                                    }
                                }
                            }
                        }
                    }
                }
                if overflow {
                    println!("Overflowed");
                    break;
                }
            } else {
                i = i + 1;
            }
        }
    }
    
    rx_stream.deactivate(None).expect("Rx Stream could not successfully deactivate");
}
