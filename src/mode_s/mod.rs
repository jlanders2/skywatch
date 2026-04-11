use std::{error::Error, fmt};

mod constants;
pub mod format;

use constants::*;
use format::*;

// The following lengths are specific to 1090MHz ADS-B
// and should be placed appropriately at some point.
// They are also specifically tied to the sample rate
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

// In reality i don't need to loop or anything, i know the exact
// slices needed for each location. So when refactoring consider this.
pub fn proccess_samples(samples: Vec<f32>) -> Result<Vec<AdsBData>, ModeSError> {
    let mut ads_b_hits: Vec<AdsBData> = Vec::new();
    let samples_read = samples.len();

    let mut i = 0;
    while i < samples_read {
        // Preamble
        if (i + ADS_B_LENGTH) > samples_read {
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
            let mut temp = [0u8; MESSAGE_LENGTH / 2];
            for j in 0..MESSAGE_LENGTH / 2 {
                // j * 2 gives me the correct location in the doubled samples buffer
                let bit_slice = &message_buffer[j * 2..(j * 2) + 2];
                let bit = extract_u8(bit_slice, bit_slice.len());
                temp[j] = bit;

                i += 2; // move forward, 1 bit 2 samples @ 2_000_000 sample rate
            }
            ads_b_hit.message = temp;
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

// TODO: These tests aren't great, they mostly test happy path
// and could easily be re-written to be more robust
// Need quit a few helper methods though.
#[cfg(test)]
mod tests {
    use crate::mode_s::{
        ADS_B_LENGTH, CA_LENGTH, DF_LENGTH, ICAO_LENGTH, MESSAGE_LENGTH, PREAMBLE_LENGTH,
        PREAMBLE_PATTERN,
        format::{AdsBData, get_callsign},
        proccess_samples,
    };

    #[test]
    fn df_extracts_correctly() {
        let mut samples: Vec<f32> = Vec::new();
        // Populate pre-amble
        for sample in PREAMBLE_PATTERN {
            if sample {
                samples.push(1.0);
            } else {
                samples.push(0.0);
            }
        }
        // Populate DF
        samples.push(1.0); // 1
        samples.push(0.0);
        samples.push(0.0); // 0
        samples.push(0.0);
        samples.push(0.0); // 0
        samples.push(0.0);
        samples.push(0.0); // 0
        samples.push(0.0);
        samples.push(1.0); // 1
        samples.push(0.0);
        samples.push(0.0); // 0
        samples.push(0.0);
        samples.push(0.0); // 0
        samples.push(0.0);
        samples.push(1.0); // 1
        samples.push(0.0);
        // fill everything else with 0s
        for i in 0..ADS_B_LENGTH - samples.iter().len() {
            samples.push(0.0);
        }
        assert_eq!(samples.iter().len(), ADS_B_LENGTH);
        let mut ads_b_results: Vec<AdsBData> = Vec::new();
        ads_b_results = proccess_samples(samples).unwrap();

        assert_eq!(ads_b_results[0].downlink_format, 17);
    }

    #[test]
    fn ca_extracts_correctly() {
        let mut samples: Vec<f32> = Vec::new();
        // Populate pre-amble
        for sample in PREAMBLE_PATTERN {
            if sample {
                samples.push(1.0);
            } else {
                samples.push(0.0);
            }
        }
        // fill everything else with 0s
        for i in 0..ADS_B_LENGTH - samples.iter().len() {
            samples.push(0.0);
        }
        let ca_start = PREAMBLE_LENGTH + DF_LENGTH;
        let ca_end = ca_start + CA_LENGTH;
        for i in ca_start..ca_end {
            if i % 2 == 0 {
                samples[i] = 1.0;
            } else {
                samples[i] = 0.0;
            }
        }
        assert_eq!(samples.iter().len(), ADS_B_LENGTH);
        let mut ads_b_results: Vec<AdsBData> = Vec::new();
        ads_b_results = proccess_samples(samples).unwrap();

        assert_eq!(ads_b_results[0].transponder_capability, 7);
    }

    #[test]
    fn message_extracts_correctly() {
        let mut samples: Vec<f32> = Vec::new();
        // Populate pre-amble
        for sample in PREAMBLE_PATTERN {
            if sample {
                samples.push(1.0);
            } else {
                samples.push(0.0);
            }
        }
        // fill everything else with 0s
        for i in 0..ADS_B_LENGTH - samples.iter().len() {
            samples.push(0.0);
        }
        let message_start = PREAMBLE_LENGTH + DF_LENGTH + CA_LENGTH + ICAO_LENGTH;
        let message_end = message_start + MESSAGE_LENGTH;
        let mut dbg_counter = 0;
        for i in message_start..message_end {
            if i % 2 == 0 {
                dbg_counter += 1;
                samples[i] = 1.0;
            } else {
                samples[i] = 0.0;
            }
        }
        assert_eq!(message_end - message_start, MESSAGE_LENGTH);
        assert_eq!(samples.iter().len(), ADS_B_LENGTH);
        let mut ads_b_results: Vec<AdsBData> = Vec::new();
        ads_b_results = proccess_samples(samples).unwrap();
        for ads_b_hit in ads_b_results {
            for (i, message_bit) in ads_b_hit.message.iter().enumerate() {
                assert_eq!(
                    *message_bit,
                    1,
                    "Failed at index {}, bit {}",
                    i,
                    message_start + i
                );
            }
        }
    }

    #[test]
    fn test_process_samples_known_frame() {
        // AI Generated Test Case
        // 8D4840D6202CC371C32CE0576098
        // DF=17 (10001), CA=5 (101), ICAO=4840D6, ME=202CC371C32CE0, PI=576098
        let df_bits = [1, 0, 0, 0, 1];
        let ca_bits = [1, 0, 1];
        let icao_bits = [
            0, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 0, 1, 1, 0,
        ];
        let me_bits = [
            0, 0, 1, 0, 0, // TC=4
            0, 0, 0, // CA=0
            0, 0, 1, 0, 1, 1, // K
            0, 0, 1, 1, 0, 0, // L
            0, 0, 1, 1, 0, 1, // M
            1, 1, 0, 0, 0, 1, // 1
            1, 1, 0, 0, 0, 0, // 0
            1, 1, 0, 0, 1, 0, // 2
            1, 1, 0, 0, 1, 1, // 3
            1, 0, 0, 0, 0, 0, // space
        ];
        let pi_bits = [
            0, 1, 0, 1, 0, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 0,
        ];

        // Helper: encode bits as PPM samples (1 = [1.0, 0.0], 0 = [0.0, 0.0])
        fn to_samples(bits: &[u8]) -> Vec<f32> {
            bits.iter()
                .flat_map(|&b| {
                    if b == 1 {
                        vec![1.0, 0.0]
                    } else {
                        vec![0.0, 0.0]
                    }
                })
                .collect()
        }

        // Preamble pattern (hardcoded from PREAMBLE_PATTERN)
        let preamble: Vec<f32> = vec![
            1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ];

        let mut samples: Vec<f32> = Vec::new();
        samples.extend_from_slice(&preamble);
        samples.extend(to_samples(&df_bits));
        samples.extend(to_samples(&ca_bits));
        samples.extend(to_samples(&icao_bits));
        samples.extend(to_samples(&me_bits));
        samples.extend(to_samples(&pi_bits));

        let results = proccess_samples(samples).unwrap();
        assert_eq!(results.len(), 1, "should detect exactly one frame");

        let frame = &results[0];
        assert_eq!(frame.downlink_format, 17);
        assert_eq!(frame.transponder_capability, 5);

        // Verify ME bits directly
        let expected_me: [u8; 56] = [
            0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 0, 1, 1, 1, 0,
            0, 0, 1, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0,
        ];
        assert_eq!(frame.message, expected_me);
    }

    #[test]
    fn test_process_samples_skw3780() {
        // AI Generated Test Case
        // Raw frame: 8da32fd5234cb5f3df8c20f5d4af
        // ME: 234cb5f3df8c20
        // Expected callsign: SKW3780

        let df_bits = [1, 0, 0, 0, 1];
        let ca_bits = [1, 0, 1];
        let icao_bits = [
            1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 1,
        ];
        let me_bits: [u8; 56] = [
            0, 0, 1, 0, 0, // TC=4
            0, 1, 1, // CA=3
            0, 1, 0, 0, 1, 1, // 19 = S
            0, 0, 1, 0, 1, 1, // 11 = K
            0, 1, 0, 1, 1, 1, // 23 = W
            1, 1, 0, 0, 1, 1, // 51 = 3
            1, 1, 0, 1, 1, 1, // 55 = 7
            1, 1, 1, 0, 0, 0, // 56 = 8
            1, 1, 0, 0, 0, 0, // 48 = 0
            1, 0, 0, 0, 0, 0, // 32 = space
        ];
        let pi_bits = [
            1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1,
        ];

        fn to_samples(bits: &[u8]) -> Vec<f32> {
            bits.iter()
                .flat_map(|&b| {
                    if b == 1 {
                        vec![1.0, 0.0]
                    } else {
                        vec![0.0, 0.0]
                    }
                })
                .collect()
        }

        let preamble: Vec<f32> = vec![
            1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ];

        let mut samples: Vec<f32> = Vec::new();
        samples.extend_from_slice(&preamble);
        samples.extend(to_samples(&df_bits));
        samples.extend(to_samples(&ca_bits));
        samples.extend(to_samples(&icao_bits));
        samples.extend(to_samples(&me_bits));
        samples.extend(to_samples(&pi_bits));

        let results = proccess_samples(samples).unwrap();
        assert_eq!(results.len(), 1);

        let frame = &results[0];
        assert_eq!(frame.downlink_format, 17);
        assert_eq!(frame.transponder_capability, 5);
        assert_eq!(frame.message, me_bits);

        let callsign = get_callsign(frame);
        assert_eq!(callsign, "SKW3780 ");
    }
}
