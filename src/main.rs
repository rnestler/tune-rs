extern crate sdl2;
extern crate stft;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::pixels::Color;
use sdl2::keyboard::Keycode;
use sdl2::event::Event;

use stft::{STFT, WindowType};

type Float = f32;

pub struct FFTPrinter {
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

pub struct Printer {
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

pub struct MpscFftSender {
    stft: STFT<f32>,
    sender: mpsc::Sender<Vec<Float>>,
}

const STEP_SIZE: usize = 2048;
const WINDOW: usize = 2048;

impl MpscFftSender {
    pub fn new(sender: mpsc::Sender<Vec<Float>>) -> MpscFftSender {
        let stft = STFT::<Float>::new(WindowType::Hanning, WINDOW, STEP_SIZE);
        MpscFftSender {
            stft: stft,
            sender: sender,
        }
    }
}

impl AudioCallback for MpscFftSender {
    type Channel = Float;

    fn callback(&mut self, input: &mut [Float]) {
        self.stft.append_samples(input);
        let mut spectrogram_column = vec![0.0; self.stft.output_size()];
        while self.stft.contains_enough_to_compute() {
            self.stft.compute_column(&mut spectrogram_column);

            self.sender.send(spectrogram_column.clone()).unwrap();

            self.stft.move_to_next_column();
        }
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    const WIDTH: u32 = 1024;
    const HEIGHT: u32 = 1024;

    let window = video_subsystem
        .window("FFT", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));

    canvas.clear();
    canvas.present();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),  // mono
        samples: Some(WINDOW as u16),    // default sample size
    };

    let (sender, receiver) = mpsc::channel();

    let device = audio_subsystem.open_capture(None, &desired_spec, |_|{ 
        //Printer { }
        //FFTPrinter::new()
        MpscFftSender::new(sender)
    }).expect(&format!("{}:{} failed", file!(), line!()));

    device.resume();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ =>  {}
            }
        }
        match receiver.try_recv() {
            Ok(data) => {
                canvas.set_draw_color(Color::RGB(0, 0, 0));
                canvas.clear();

                canvas.set_draw_color(Color::RGB(100, 0, 0));

                let scale: f32 = 1.0;
                canvas.set_scale(scale, scale).unwrap();
                for (n, (y1, y2)) in data.iter().zip(data.iter().skip(1)).enumerate()  {
                    if y1 > &0.0 || y2 > &0.0 {
                        let y1 = HEIGHT as f32 / scale - (y1 * 50.0 / scale);
                        let y2 = HEIGHT as f32 / scale - (y2 * 50.0 / scale);

                        let x1 = (n as f32) * (WIDTH as f32 / scale) / (WINDOW as f32 / 2.0);
                        let x2 = ((n+1) as f32) * (WIDTH as f32 / scale) / (WINDOW as f32 / 2.0);
                        canvas.draw_line((x1 as i32, y1 as i32), (x2 as i32, y2 as i32)).unwrap();
                    }
                }

                canvas.present();
            }
            _ => {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}
