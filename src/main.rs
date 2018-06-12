//! A demonstration of recording input
//!
//! Audio from the default input device is recorded into memory until
//! the user presses Enter. They are then played back to the default
//! output device.

extern crate portaudio;

use portaudio as pa;
use std::io;
use std::thread;
use std::time::Duration;

const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES: u32 = 256;
const CHANNELS: i32 = 2;
const INTERLEAVED: bool = true;


fn main() {
    match run() {
        Ok(_) => {},
        e => {
            eprintln!("Example failed with the following: {:?}", e);
        }
    }
}

fn run() -> Result<(), pa::Error> {

    let pa = try!(pa::PortAudio::new());

    println!("PortAudio:");
    println!("version: {}", pa.version());
    println!("version text: {:?}", pa.version_text());
    println!("host count: {}", try!(pa.host_api_count()));

    let default_host = try!(pa.default_host_api());
    println!("default host: {:#?}", pa.host_api_info(default_host));

    let def_input = try!(pa.default_input_device());
    let input_info = try!(pa.device_info(def_input));
    println!("Default input device info: {:#?}", &input_info);

    // Construct the input stream parameters.
    let latency = input_info.default_low_input_latency;
    let input_params = pa::StreamParameters::<f32>::new(def_input, CHANNELS, INTERLEAVED, latency);

    // Check that the stream format is supported.
    try!(pa.is_input_format_supported(input_params, SAMPLE_RATE));

    // Construct the settings with which we'll open our input stream.
    let input_settings = pa::InputStreamSettings::new(input_params, SAMPLE_RATE, FRAMES);

    // We'll use this channel to send the samples back to the main thread.
    let (sender, receiver) = ::std::sync::mpsc::channel();

    // A callback to pass to the non-blocking input stream.
    let input_callback = move |pa::InputStreamCallbackArgs { buffer, frames, flags, time }| {
        assert!(frames == FRAMES as usize);
        // get start time of callback
        //
        // get end time of callbac
        //
        // time of sample is time.now + time.buffer_adc

        println!("time: {:?}", time);
        println!("flags: {}", flags);

        // We'll construct a copy of the input buffer and send that
        // onto the channel. This doesn't block, even though nothing
        // is waiting on the receiver yet.
        let vec_buffer = Vec::from(buffer);
        // There are actually 512 samples here. 256 for the left, 256 for the right.
        assert!(vec_buffer.len() == FRAMES as usize * CHANNELS as usize);

        // If sending fails (the receiver has been dropped), stop recording
        match sender.send(vec_buffer) {
            Ok(_) => pa::Continue,
            Err(_) => pa::Complete
        }
    };

    // Construct a stream with input sample types of f32.
    let mut input_stream = try!(pa.open_non_blocking_stream(input_settings, input_callback));

    try!(input_stream.start());

    println!("Recording has started. Press Enter to stop.");
    
    // Wait for enter to be pressed
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).ok();

    try!(input_stream.stop());

    Ok(())
}
