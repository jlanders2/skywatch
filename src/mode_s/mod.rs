use std::{error::Error, fmt};

mod constants;
mod enums;

use constants::*;
use enums::*;

// The following lengths are specific to 1090MHz ADS-B
// and should be placed appropriately at some point.
static ADS_B_LENGTH: usize = 112 * 2; // 112 bits, 124 samples @ 2_000_000 sample rate
static DF_LENGTH: usize = 5 * 2; // 5 bits, 10 samples @ 2_000_000 sample rate
static CA_LENGTH: usize = 3 * 2; // 3 bits, 6 samples @ 2_000_000 sample rate
static ICAO_LENGTH: usize = 24 * 2; // 24 bits, 48 samples @ 2_000_000 sample rate
static PREAMBLE_LENGTH: usize = 8 * 2; // 8 bits, 16 samples @ 2_000_000 sample rate
static MESSAGE_LENGTH: usize = 56 * 2; // 56 bits, 112 samples @ 2_000_000 sample rate
static TC_LENGTH: usize = 5 * 2; // 5 bits, 10 samples @ 2_000_000 sample rate
static PARITY_LENGTH: usize = 24 * 2; // 24 bits, 48 samples @ 2_000_000 sample rate
const PREAMBLE_PATTERN: [bool; PREAMBLE_LENGTH] = [
    true, false, true, false, false, false, false, true, false, true, false, false, false, false,
    false, false,
];

#[derive(Debug)]
pub struct ModeSError;

impl Error for ModeSError {}

impl fmt::Display for ModeSError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ModeSError - Bad error, not descriptive")
    }
}

pub fn proccess_samples(samples: Vec<f32>) -> Result<(), ModeSError> {
    let samples_read = samples.len();

    let mut i = 0;
    while i < samples_read {
        // Preamble
        if (i + ADS_B_LENGTH) >= samples_read {
            break;
        }
        let preamble: [f32; PREAMBLE_LENGTH] = samples[i..i + PREAMBLE_LENGTH]
            .try_into()
            .expect("slice length is always PREAMBLE_LENGTH");
        let preamble_detected = check_preamble(preamble);
        if preamble_detected {
            // TODO: Should read this into a struct
            i += PREAMBLE_LENGTH;
            // DF
            let df_buffer: [f32; DF_LENGTH] = samples[i..i + DF_LENGTH]
                .try_into()
                .expect("slice length is always DF_LENGTH");
            let df = extract_u8(&df_buffer, DF_LENGTH);
            i += DF_LENGTH;
            println!("DF: {}", df);
            if df == 17 {
                // ADS-B
                // CA
                let ca_buffer: [f32; CA_LENGTH] = samples[i..i + CA_LENGTH]
                    .try_into()
                    .expect("slice length is always CA_LENGTH");
                let ca = extract_u8(&ca_buffer, CA_LENGTH);
                i += CA_LENGTH;
                // ICAO
                let icao_buffer: [f32; ICAO_LENGTH] = samples[i..i + ICAO_LENGTH]
                    .try_into()
                    .expect("slice length is always ICAO_LENGTH");
                // TODO: Not sure u8 works here, but it"ll do to build :)
                let icao = extract_u8(&icao_buffer, ICAO_LENGTH);
                i += ICAO_LENGTH;
                // Message
                let message_buffer: [f32; MESSAGE_LENGTH] = samples[i..i + MESSAGE_LENGTH]
                    .try_into()
                    .expect("slice length is always MESSAGE_LENGTH");
                i += MESSAGE_LENGTH;
                // TC - contained in first 5 bits of message
                let tc_buffer: [f32; TC_LENGTH] = message_buffer[0..TC_LENGTH]
                    .try_into()
                    .expect("slice length is always TC_LENGTH");
                let tc = extract_u8(&tc_buffer, TC_LENGTH);
                // Parity
                let parity_buffer: [f32; PARITY_LENGTH] = samples[i..i + PARITY_LENGTH]
                    .try_into()
                    .expect("slice length is always PARITY_LENGTH");
                // TODO: Not sure u8 works here, but it"ll do to build :)
                let parity = extract_u8(&parity_buffer, PARITY_LENGTH);
                i += PARITY_LENGTH;

                println!("TC: {}", tc);
                if (tc >= 1 && tc <= 4) {
                    let callsign = decode_callsign(message_buffer);
                    println!("Detected Aircraft: {}", callsign);
                }
            }
        } else {
            i += 1;
        }
    }

    Ok(())
}

fn check_preamble(magnitude_buffer: [f32; PREAMBLE_LENGTH]) -> bool {
    let mut result = true;
    for i in 0..PREAMBLE_LENGTH {
        if (magnitude_buffer[i] > HIT_THRESHOLD).ne(&PREAMBLE_PATTERN[i]) {
            result = false;
            break;
        }
    }

    result
}

fn extract_u8(buffer: &[f32], buffer_len: usize) -> u8 {
    let mut result = 0u8;
    for bit in 0..buffer_len {
        let first = buffer[bit * 2];
        let second = buffer[(bit * 2) + 1];
        if first > second {
            result |= 1 << (4 - bit);
        }
    }

    result
}

// TODO: temp function
// this is horrible code, just trying to brute force
fn decode_callsign(me_buffer: [f32; 112]) -> String {
    let temp = me_buffer[(8 * 2)..(MESSAGE_LENGTH * 2)].to_vec();
    let mut result = String::with_capacity(8);
    let callsign_char_bit_length: usize = 6;

    for i in 0..7 {
        let start_index = i * callsign_char_bit_length;
        let end_index = start_index + callsign_char_bit_length;
        let char_as_u8 = extract_u8(&temp[start_index..end_index], callsign_char_bit_length);
        result.push_str(u8_to_callsign_char(char_as_u8));
    }

    result
}

// TODO: temp function
// TODO: see below note
// If you are familiar with the ASCII (American Standard Code for Information Interchange)
// code, it is easy to identify that a callsign character
// is encoded using the lower six bits of the same character in ASCII.
fn u8_to_callsign_char(encoded_value: u8) -> &'static str {
    match encoded_value {
        1 => "A",
        2 => "B",
        3 => "C",
        4 => "D",
        5 => "E",
        6 => "F",
        7 => "G",
        8 => "H",
        9 => "I",
        10 => "J",
        11 => "K",
        12 => "L",
        13 => "M",
        14 => "N",
        15 => "O",
        16 => "P",
        17 => "Q",
        18 => "R",
        19 => "S",
        20 => "T",
        21 => "U",
        22 => "V",
        23 => "W",
        24 => "X",
        25 => "Y",
        26 => "Z",
        48 => "0",
        49 => "1",
        50 => "2",
        51 => "3",
        52 => "4",
        53 => "5",
        54 => "6",
        55 => "7",
        56 => "8",
        57 => "9",
        _ => "",
    }
}
