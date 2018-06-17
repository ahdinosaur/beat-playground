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
use std::iter;
use std::sync::mpsc;

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

    // read samples into a ring_buffer: https://github.com/RustAudio/sample/blob/master/src/ring_buffer.rs
    // let mut buffer = ring_buffer::Fixed::from([Sample::equilibrium(); MAX_SAMPLES_PER_BEAT * CHANNELS as usize]);

    let pa_reader = try!(PortAudioReader::start());

    let signal = pa_reader.iter()
        .flat_map(|s| s.until_exhausted());

    for sample in signal {
        println!("Sample! {:?}", sample);
    }

    Ok(())
}

struct PortAudioReader {
    stream: pa::Stream<pa::NonBlocking, pa::Input<f32>>,
    receiver: mpsc::Receiver<Vec<f32>>
}

impl PortAudioReader {
    fn start() -> Result<Self, pa::Error> {
        let pa = try!(pa::PortAudio::new());

        println!("PortAudio");
        println!("version: {}", pa.version());
        println!("version text: {:?}", pa.version_text());
        println!("host count: {}", try!(pa.host_api_count()));

        let default_host = try!(pa.default_host_api());
        println!("default host: {:#?}", pa.host_api_info(default_host));

        let def_input = try!(pa.default_input_device());
        let info = try!(pa.device_info(def_input));
        println!("Default input device info: {:#?}", &info);

        // Construct the input stream parameters.
        let latency = info.default_low_input_latency;
        let params = pa::StreamParameters::<f32>::new(def_input, CHANNELS, INTERLEAVED, latency);

        // Check that the stream format is supported.
        try!(pa.is_input_format_supported(params, SAMPLE_RATE));

        // Construct the settings with which we'll open our input stream.
        let settings = pa::InputStreamSettings::new(params, SAMPLE_RATE, FRAMES);

        // We'll use this channel to send the samples back to the main thread.
        let (sender, receiver) = ::std::sync::mpsc::channel();

        // A callback to pass to the non-blocking input stream.
        let callback = move |pa::InputStreamCallbackArgs { buffer, frames, .. }| {
            assert!(frames == FRAMES as usize);

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
        let mut stream = try!(pa.open_non_blocking_stream(settings, callback));

        try!(stream.start());
        
        Ok(PortAudioReader {
            stream,
            receiver
        })
    }

    fn iter (&self) -> PortAudioReaderIterator {
        PortAudioReaderIterator {
            stream: &self.stream,
            receiver: &self.receiver
        }
    }
}


struct PortAudioReaderIterator<'a> {
    stream: &'a pa::Stream<pa::NonBlocking, pa::Input<f32>>,
    receiver: &'a mpsc::Receiver<Vec<f32>>
}

impl<'a> PortAudioReaderIterator<'a> {
    fn read_next_buffer (&self) -> Result<Vec<f32>, mpsc::RecvError> {
        self.receiver.recv()
    }
}

impl<'a> Iterator for PortAudioReaderIterator<'a> {
    type Item = Box<Signal<Frame=[f32; CHANNELS as usize]> + 'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.read_next_buffer() {
            Ok(buffer) => {
                let signal = signal::from_interleaved_samples_iter::<_, [f32; CHANNELS as usize]>(buffer.into_iter());
                Some(Box::new(signal))
            },
            Err(err) => panic!(err),
        }
    }
}
