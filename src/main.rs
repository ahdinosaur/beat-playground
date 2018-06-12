//! A demonstration of constructing and using a blocking stream.
//!
//! Audio from the default input device is passed directly to the default output device in a duplex
//! stream, so beware of feedback!

extern crate portaudio;
extern crate sample;

use portaudio as pa;
use sample::{Sample, Signal, signal};
use sample::ring_buffer;
use std::collections::VecDeque;

const SAMPLE_RATE: f64 = 44_100.0;
const CHANNELS: i32 = 2;
const FRAMES: u32 = 256;
const INTERLEAVED: bool = true;
const MAX_SAMPLES_PER_BEAT: usize = 1024;

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

    println!("PortAudio");
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

    // Construct the settings with which we'll open our duplex stream.
    let settings = pa::InputStreamSettings::new(input_params, SAMPLE_RATE, FRAMES);

    let mut stream = try!(pa.open_blocking_stream(settings));

    // read samples into a ring_buffer: https://github.com/RustAudio/sample/blob/master/src/ring_buffer.rs
    let mut buffer = ring_buffer::Fixed::from([Sample::equilibrium(); MAX_SAMPLES_PER_BEAT * CHANNELS as usize]);

    try!(stream.start());

    // We'll use this function to wait for read/write availability.
    fn wait_for_stream<F>(f: F, name: &str) -> u32
        where F: Fn() -> Result<pa::StreamAvailable, pa::error::Error>
    {
        'waiting_for_stream: loop {
            match f() {
                Ok(available) => match available {
                    pa::StreamAvailable::Frames(frames) => return frames as u32,
                    pa::StreamAvailable::InputOverflowed => println!("Input stream has overflowed"),
                    pa::StreamAvailable::OutputUnderflowed => println!("Output stream has underflowed"),
                },
                Err(err) => panic!("An error occurred while waiting for the {} stream: {}", name, err),
            }
        }
    };

    let mut i = 0;

    // Now start the main read/write loop! In this example, we pass the input buffer directly to
    // the output buffer, so watch out for feedback.
    'stream: loop {

        // how many samples are available on the input stream?
        let num_input_samples = wait_for_stream(|| stream.read_available(), "Read");

        // if there are samples available, let's take them and add them to the buffer
        if num_input_samples > 0 {
            let samples = try!(stream.read(num_input_samples));
            for sample in samples {
                println!("Sample {:?}", sample);
                buffer.push(*sample);
            }
            println!("Read {:?} samples from the input stream.", num_input_samples);
            println!("Time: {}", stream.time());
        }

        let signal = signal::from_interleaved_samples_iter::<_, [f32; CHANNELS as usize]>(
            buffer.iter().map(|item|*item)
        );
    }
}
