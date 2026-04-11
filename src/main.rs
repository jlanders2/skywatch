use skywatch::mode_s;
use skywatch::mode_s::format::get_callsign;
use skywatch::mode_s::format::get_type_code;
use skywatch::runtime;
use skywatch::sdr::SdrDirection;

fn main() {
    let sdr = runtime::sdr_factory("soapysdr");
    // Builder pattern might be nice here
    let mut device = sdr.create_device().unwrap();
    device.set_channel(0);
    device.set_direction(SdrDirection::Receive);
    device.set_frequency(1_090_000_000.0);
    device.set_sample_rate(2_000_000.0);
    device.set_gain(40.0);

    let mut stream = device.get_stream().unwrap();
    loop {
        let samples = stream.read().unwrap();
        let hits = mode_s::proccess_samples(samples).expect("Samples read successfully");
        for hit in hits {
            if hit.downlink_format == 17 {
                let tc = get_type_code(&hit);
                if tc >= 1 && tc <= 4 {
                    let callsign = get_callsign(&hit);

                    if !callsign.contains("#") {
                        println!("Found: {}", callsign);
                    }
                }
            }
        }
    }

    // cleanup -> still not sure how to get to here.
    // Think it will be more relevant once there is
    // a GUI implemented.
}
