use std::{error::Error, fmt};

mod constants;
mod enums;
mod format;

use constants::*;
use enums::*;
use format::*;

// The following lengths are specific to 1090MHz ADS-B
// and should be placed appropriately at some point.
static ADS_B_LENGTH: usize = 112 * 2; // 112 bits, 224 samples @ 2_000_000 sample rate
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

pub fn proccess_samples(samples: Vec<f32>) -> Result<Vec<AdsBData>, ModeSError> {
    let mut ads_b_hits: Vec<AdsBData> = Vec::new();
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
            i += PREAMBLE_LENGTH; // move forward
            let mut ads_b_hit = AdsBData {
                downlink_format: 0,
                transponder_capability: 0,
                message: [0; 56],
            };
            // DF
            let df_buffer: [f32; DF_LENGTH] = samples[i..i + DF_LENGTH]
                .try_into()
                .expect("slice length is always DF_LENGTH");
            ads_b_hit.downlink_format = extract_u8(&df_buffer, DF_LENGTH);
            i += DF_LENGTH; // move forward
            // CA
            let ca_buffer: [f32; CA_LENGTH] = samples[i..i + CA_LENGTH]
                .try_into()
                .expect("slice length is always CA_LENGTH");
            ads_b_hit.transponder_capability = extract_u8(&ca_buffer, CA_LENGTH);
            i += CA_LENGTH; // move forward
            // ICAO
            i += ICAO_LENGTH; // move forward, skip icao
            // Message
            let message_buffer: [f32; MESSAGE_LENGTH] = samples[i..i + MESSAGE_LENGTH]
                .try_into()
                .expect("slice length is always MESSAGE_LENGTH");
            // Loop over to extract bits
            let mut temp = [0u8; 56];
            for i in 0..MESSAGE_LENGTH / 2 {
                let bit_slice = &message_buffer[i..i + 2];
                let bit = extract_u8(bit_slice, bit_slice.len());
                temp[i] = bit;
            }
            ads_b_hit.message = temp;
            i += MESSAGE_LENGTH; // move forward
            // Parity
            i += PARITY_LENGTH; // move forward, skip parity
            ads_b_hits.push(ads_b_hit);
        } else {
            i += 1;
        }
    }

    Ok(ads_b_hits)
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

// Note to self, this is HIGHLY dependent on 2_000_000 sample rate
fn extract_u8(buffer: &[f32], buffer_len: usize) -> u8 {
    let mut result = 0u8;
    for bit in 0..(buffer_len / 2) {
        let first = buffer[bit * 2];
        let second = buffer[(bit * 2) + 1];
        if first > second {
            // mode-s is Big endian
            result |= 1 << (((buffer_len / 2) - 1) - bit);
        }
    }

    result
}
