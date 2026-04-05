fn main() {
    let sdr = skywatch_runtime::sdr_factory("soapysdr");
    let device = sdr.create_device().unwrap();
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
