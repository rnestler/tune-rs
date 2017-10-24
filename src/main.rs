extern crate sdl2;
extern crate stft;

use std::thread::sleep;
use std::time::Duration;

use sdl2::audio::{AudioCallback, AudioSpecDesired};

use stft::{STFT, WindowType};

type Float = f32;

struct FFTPrinter {
    stft: STFT<f32>,
}

impl FFTPrinter {
    pub fn new() -> FFTPrinter {
        let stft = STFT::<Float>::new(WindowType::Hanning, 2048, 1024);
        FFTPrinter {
            stft: stft,
        }
    }
}

impl AudioCallback for FFTPrinter {
    type Channel = Float;

    fn callback(&mut self, input: &mut [Float]) {
        self.stft.append_samples(input);
        let mut spectrogram_column = vec![0.0; self.stft.output_size()];
        while self.stft.contains_enough_to_compute() {
            self.stft.compute_column(&mut spectrogram_column);

            for x in &spectrogram_column {
                print!("{} ", x);
            }
            println!("");

            self.stft.move_to_next_column();
        }
    }
}

struct Printer {
}

impl AudioCallback for Printer {
    type Channel = u16;

    fn callback(&mut self, out: &mut [u16]) {
        for x in out {
            print!("{} ", x);
        }
        println!("");
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),  // mono
        samples: None       // default sample size
    };

    let device = audio_subsystem.open_capture(None, &desired_spec, |_|{ 
        //Printer { }
        FFTPrinter::new()
    }).expect(&format!("{}:{} failed", file!(), line!()));

    device.resume();

    sleep(Duration::from_secs(2));
}
