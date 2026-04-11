// Not sure this belongs here
// definitely named wrong

// Not using icao or parity
// Notably Vec will be slow, could use 2xu32 or 1xu64; don't care for now though

pub struct AdsBData {
    pub downlink_format: u8,
    pub transponder_capability: u8,
    pub message: [u8; 56],
}

pub fn get_type_code(data: &AdsBData) -> u8 {
    let tc_vec = &data.message[0..5];
    let mut result: u8 = 0;
    for i in 0..tc_vec.iter().len() {
        if tc_vec[i] == 1 {
            // Guaranteed 5 bit length, so last index 5 - 1 = 4
            result |= 1 << (4 - i);
        }
    }

    result
}

static CHARSET: &[u8] = b"#ABCDEFGHIJKLMNOPQRSTUVWXYZ##### ###############0123456789######";

pub fn get_callsign(data: &AdsBData) -> String {
    let mut result = String::with_capacity(8);
    let callsign_buffer = &data.message[8..];

    for i in 0..8 {
        let mut idx = 0u8;
        for j in 0..6 {
            if callsign_buffer[(i * 6) + j] == 1 {
                // Guaranteed 6 bit length, so last index 6 - 1 = 5
                idx |= 1 << (5 - j);
            }
        }
        result.push(CHARSET[idx as usize] as char);
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::mode_s::format::{AdsBData, get_callsign, get_type_code};

    #[test]
    fn test_get_type_code() {
        let mut data = AdsBData {
            downlink_format: 0,
            transponder_capability: 0,
            message: [0; 56],
        };
        assert_eq!(get_type_code(&data), 0);
        data.message[0] = 1;
        assert_eq!(get_type_code(&data), 16);
        data.message[0] = 0;
        data.message[1] = 1;
        assert_eq!(get_type_code(&data), 8);
        data.message[0] = 0;
        data.message[1] = 0;
        data.message[2] = 1;
        assert_eq!(get_type_code(&data), 4);
        data.message[0] = 0;
        data.message[1] = 0;
        data.message[2] = 0;
        data.message[3] = 1;
        assert_eq!(get_type_code(&data), 2);
        data.message[0] = 0;
        data.message[1] = 0;
        data.message[2] = 0;
        data.message[3] = 0;
        data.message[4] = 1;
        assert_eq!(get_type_code(&data), 1);
        data.message[0] = 1;
        data.message[1] = 0;
        data.message[2] = 1;
        data.message[3] = 0;
        data.message[4] = 1;
        assert_eq!(get_type_code(&data), 21);
    }

    #[test]
    fn test_get_callsign_klm1023() {
        // AI Generated test case
        // ME field from 8D4840D6202CC371C32CE0576098
        // HEX: 202CC371C32CE0
        // BIN: 00100000001011001100001101110001110000110010110011100000
        let bits: [u8; 56] = [
            0, 0, 1, 0, 0, // TC = 4
            0, 0, 0, // CA = 0
            // C1-C8 (callsign chars)
            0, 0, 1, 0, 1, 1, // 11 = K
            0, 0, 1, 1, 0, 0, // 12 = L
            0, 0, 1, 1, 0, 1, // 13 = M
            1, 1, 0, 0, 0, 1, // 49 = 1
            1, 1, 0, 0, 0, 0, // 48 = 0
            1, 1, 0, 0, 1, 0, // 50 = 2
            1, 1, 0, 0, 1, 1, // 51 = 3
            1, 0, 0, 0, 0, 0, // 32 = space
        ];

        let data = AdsBData {
            downlink_format: 17,
            transponder_capability: 0,
            message: bits,
        };

        assert_eq!(get_callsign(&data), "KLM1023 ");
    }
}
