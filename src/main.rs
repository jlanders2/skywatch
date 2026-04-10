use skywatch::mode_s;
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
                println!(
                    "DF-{}, TC-{}",
                    hit.downlink_format, hit.transponder_capability
                );
            }
        }
    }

    // cleanup -> still not sure how to get to here.
    // Think it will be more relevant once there is
    // a GUI implemented.
}
