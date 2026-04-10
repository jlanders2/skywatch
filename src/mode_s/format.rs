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
    // are slices from a vec O(1)? Unlikely
    // may want message to be an array, but for now
    // don't care
    let tc_vec = &data.message[0..5];
    let mut result: u8 = 0;
    for i in 0..tc_vec.iter().len() {
        if tc_vec[i] == 1 {
            result |= 1 << (4 - i);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::mode_s::format::{AdsBData, get_type_code};

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
}
