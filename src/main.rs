extern crate sdl2;

use std::thread::sleep;
use std::time::Duration;

use sdl2::audio::{AudioCallback, AudioSpecDesired};

struct Printer {
}

impl AudioCallback  for Printer {
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
        Printer { }
    }).expect(&format!("{}:{} failed", file!(), line!()));

    device.resume();

    sleep(Duration::from_secs(2));
}
