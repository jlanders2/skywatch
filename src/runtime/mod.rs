use crate::mock_sdr::MockSdr;
use crate::sdr::Sdr;
use crate::soapy_sdr::SoapySdr;

pub fn sdr_factory(sdr_type: &str) -> Box<dyn Sdr> {
    match sdr_type {
        "soapysdr" => Box::new(SoapySdr),
        "mock" => Box::new(MockSdr),
        _ => panic!("Error: Sdr type not valid"),
    }
}
