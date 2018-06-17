//! A demonstration of constructing and using a blocking stream.
//!
//! Audio from the default input device is passed directly to the default output device in a duplex
//! stream, so beware of feedback!

extern crate portaudio;
extern crate sample;

use portaudio as pa;
use sample::{envelope, Sample, Signal, signal};
use sample::frame;
use sample::ring_buffer;
use std::collections::VecDeque;

const SAMPLE_RATE: f64 = 44_100.0;
const CHANNELS: i32 = 2;
const FRAMES: u32 = 256;
const INTERLEAVED: bool = true;
const MAX_SAMPLES_PER_BEAT: usize = 1024;

fn main() {
    match run() {
        Ok(()) => {},
        e => {
            eprintln!("Example failed with the following: {:?}", e);
        }
    }
}

fn run() -> Result<(), pa::Error> {

    // read samples into a ring_buffer: https://github.com/RustAudio/sample/blob/master/src/ring_buffer.rs
    // let mut buffer = ring_buffer::Fixed::from([Sample::equilibrium(); MAX_SAMPLES_PER_BEAT * CHANNELS as usize]);

    let pa_reader = try!(PortAudioReader::new());

    for signal in pa_reader.iter() {
        if signal.is_none() { continue; }
        println!("Signal!");

        for sample in signal.unwrap().until_exhausted() {
            println!("Sample!");
        }
    }

    Ok(())
}

// TODO streaming iterator?
// or store buffer in iterator

struct PortAudioReader {
    stream: pa::Stream<pa::Blocking<pa::stream::Buffer>, pa::Input<f32>>,
}

impl PortAudioReader {
    fn new() -> Result<Self, pa::Error> {
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

        // Construct the settings with which we'll open our input stream.
        let settings = pa::InputStreamSettings::new(input_params, SAMPLE_RATE, FRAMES);

        let mut stream = try!(pa.open_blocking_stream(settings));
        
        Ok(PortAudioReader {
            stream
        })
    }

    fn start (&mut self) -> Result<(), pa::Error> {
        self.stream.start()
    }

    fn iter (&self) -> PortAudioReaderIterator {
        PortAudioReaderIterator {
            stream: &self.stream
        }
    }
}


struct PortAudioReaderIterator<'a> {
    stream: &'a pa::Stream<pa::Blocking<pa::stream::Buffer>, pa::Input<f32>>,
}

impl<'a> PortAudioReaderIterator<'a> {
    fn read_next_buffer (&self) -> Result<Option<&'a [f32]>, pa::Error> {
        // how many samples are available on the input stream?
        let num_input_samples = wait_for_stream(|| self.stream.read_available(), "Read");
        // println!("Available samples: {:?}", num_input_samples);
        
        if num_input_samples == 0 { return Ok(None); }

        // if there are samples available, let's take them and add them to the buffer
        let buffer = try!(self.stream.read(num_input_samples));

        // println!("Read samples: {:?}", num_input_samples);
        // println!("Time: {}", stream.time());
        
        Ok(Some(buffer))
    }
}

impl<'a> Iterator for PortAudioReaderIterator<'a> {
    type Item = Option<Box<Signal<Frame=[f32; CHANNELS as usize]> + 'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        'waiting_for_next: loop {
            match self.read_next_buffer() {
                Ok(Some(buffer)) => {
                    let interleaved_samples_iter = buffer.iter().cloned();
                    let signal = signal::from_interleaved_samples_iter::<_, [f32; CHANNELS as usize]>(interleaved_samples_iter);
                    return Some(Some(Box::new(signal)))
                },
                Ok(None) => {
                    println!("No signal");
                    return Some(None);
                },
                Err(err) => panic!(err),
            }
        }
    }
}


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
}
