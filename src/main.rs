mod chip8;
use chip8::Chip8;
use sdl3::{event::Event, keyboard::Keycode, pixels::Color};
use std::time::Duration;

use crate::chip8::LoadFont;
mod constants;
fn main() {
    // opening sdl and creating a window
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(constants::WINDOW_NAME, constants::WIDTH, constants::HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas(); // where to write pixels
    let mut chip8 = Chip8::default();
    chip8.load_fonts(&constants::FONTS);
    canvas.set_draw_color(Color::RGB(255, 0, 0));
    canvas.clear();
    canvas.present(); // renders window, showing what changed.
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
    }
    canvas.present();
    ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
}
