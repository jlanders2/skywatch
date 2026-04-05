use sdr_core::SdrDirection;

fn main() {
    let sdr = skywatch_runtime::sdr_factory("soapysdr");
    let mut device = sdr.create_device().unwrap();
    // Assume all these let _ are an anti-pattern
    let _ = device.set_channel(0);
    let _ = device.set_direction(SdrDirection::Receive);
    let _ = device.set_frequency(1_090_000_000.0);
    let _ = device.set_sample_rate(2_000_000.0);
    let _ = device.set_gain(40.0);

    let mut stream = device.get_stream().unwrap();
    loop {
        let samples = stream.read().unwrap();
        // process samples mode_s
        mode_s::proccess_samples(samples).expect("Samples read successfully");

    }

    // cleanup -> still not sure how to get to here.
    // Think it will be more relevant once there is 
    // a GUI implemented.
}
