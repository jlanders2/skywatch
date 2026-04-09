use std::{error::Error, fmt};

mod constants;

use constants::*;

static DF_LENGTH: usize = 10; // 5 bits, 10 samples @ 2_000_000 sample rate
static PREAMBLE_LENGTH: usize = 16;
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
        if (i + PREAMBLE_LENGTH) >= samples_read {
            break;
        }
        let preamble: [f32; PREAMBLE_LENGTH] = samples[i..i + PREAMBLE_LENGTH]
            .try_into()
            .expect("slice length is always PREAMBLE_LENGTH");
        let preamble_detected = check_preamble(preamble);
        if preamble_detected {
            i += PREAMBLE_LENGTH;
            // DF
            if (i + DF_LENGTH) >= samples_read {
                break;
            }
            let df_buffer: [f32; DF_LENGTH] = samples[i..i + DF_LENGTH]
                .try_into()
                .expect("slice length is always DF_LENGTH");
            let df = extract_df(df_buffer);
            println!("Preamble Hit: DF-{}", df);
            i += DF_LENGTH;
            // TC
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

fn extract_df(df_buffer: [f32; DF_LENGTH]) -> u8 {
    let mut df = 0u8;
    for bit in 0..5 {
        let first = df_buffer[bit * 2];
        let second = df_buffer[(bit * 2) + 1];
        if first > second {
            df |= 1 << (4 - bit);
        }
    }

    df
}
