use std::{io::{self}, process::exit};

use num_complex::Complex;
use soapysdr::RxStream;

static DF_LENGTH: usize = 10; // 5 bits, 10 samples @ 2_000_000 sample rate
static PREAMBLE_LENGTH: usize = 16;
const PREAMBLE_PATTERN: [bool;PREAMBLE_LENGTH]= [
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

struct SkywatchState {
    samples_buffer: Vec<Complex<f32>>,
    hit_threshold: f32,
}

fn init_skywatch() -> Result<SkywatchState, ()> {
    let state= SkywatchState {
        samples_buffer: vec![Complex::new(0.0f32, 0.0f32); 1_024_000],
        hit_threshold: 0.01f32
    };
    return Ok(state);
}

fn init_soapysdr() -> Result<RxStream<Complex<f32>>, soapysdr::Error> {
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
    
    return Ok(rx_stream);
}

fn process_samples(state: &SkywatchState)
{
    let samples_read = state.samples_buffer.len();
    
    // Convert samples from raw I/Q data to scalar values; expensive, so do once here each loop
    // I believe norm_sqrt is supposed to be more efficient if ever needed
    let magnitudes: Vec<f32> = state.samples_buffer[..samples_read].iter().map(|s|s.norm()).collect();
    
    let mut i = 0;
    while i < samples_read {
        // Preamble
        if (i + PREAMBLE_LENGTH) >= samples_read {
            break;
        }
        let preamble: [f32; PREAMBLE_LENGTH] = magnitudes[i..i + PREAMBLE_LENGTH]
            .try_into()
            .expect("slice length is always PREAMBLE_LENGTH");
        let preamble_detected = check_preamble(state, preamble);
        if preamble_detected {
            i = i + PREAMBLE_LENGTH;
            // DF
            if (i + DF_LENGTH) >= samples_read {
                break;
            }
            let df_buffer: [f32; DF_LENGTH] = magnitudes[i..i + DF_LENGTH]
                .try_into()
                .expect("slice length is always DF_LENGTH");
            let df = extract_df(df_buffer);
            println!("Preamble Hit: DF-{}", df);
            i = i + DF_LENGTH;
            // TC
        } else {
            i = i + 1;
        }
    }
}

fn check_preamble(state: &SkywatchState, magnitude_buffer: [f32; PREAMBLE_LENGTH]) -> bool {
    let mut result = true;
    for i in 0..PREAMBLE_LENGTH {
        if (magnitude_buffer[i] > state.hit_threshold).ne(&PREAMBLE_PATTERN[i]) {
            result = false;
            break;
        }
    }

    return result;
}

fn extract_df(df_buffer: [f32; DF_LENGTH]) -> u8 {
    let mut df = 0u8;
    for bit in 0..5 {
        let first = df_buffer[bit * 2];
        let second= df_buffer[(bit * 2) + 1];
        if first > second {
            df |= 1 << (4 - bit);
        }
    }

    return df;
}

fn main() {
    let mut state = init_skywatch().unwrap();
    let mut rx_stream = init_soapysdr().expect("Could not initialize SoapySDR");
    loop {
        let _ = rx_stream.read(&mut [state.samples_buffer.as_mut_slice()], 1_000_000).expect("Rx Stream could not successfully read");
        process_samples(&state);
    }
    
    rx_stream.deactivate(None).expect("Rx Stream could not successfully deactivate");
}
