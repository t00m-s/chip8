use chip8_emulator::chip8::{Chip8, LoadFont, LoadRom, MachineCycle};
use chip8_emulator::constants;
use chip8_emulator::keypad::handle_pressed_keys;
use sdl3::pixels::Color;
use sdl3::render::FRect;
use std::ffi::OsString;
use std::time::Duration;

fn main() {
    //
    let args: Vec<OsString> = std::env::args_os().collect();
    // opening sdl and creating a window
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    // unwrap because yes, if i don't have a window, how am i going to play
    let window = video_subsystem
        .window(constants::WINDOW_NAME, constants::WIDTH, constants::HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas(); // where to write pixels
    let mut chip8 = Chip8::default();
    // fine if panics, needed for input.
    let mut event_pump = sdl_context.event_pump().unwrap();
    chip8.load_fonts(&constants::FONTS);
    chip8.load_rom("roms/ibm-logo.ch8");
    'game_loop: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl3::event::Event::Quit { timestamp: _ } => {
                    drop(canvas);
                    break 'game_loop;
                }
                _ => {
                    break;
                }
            }
        }
        let keyboard = event_pump.keyboard_state();
        handle_pressed_keys(&mut chip8, keyboard);
        chip8.machine_cycle();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        for (i, &pixel) in chip8.display().into_iter().enumerate() {
            if pixel {
                let x = i % 64;
                let y = i / 64;
                // IF I CHANGE WIDTH AND HEIGHT I NEED TO CHANGE THE SCALE TOO.
                let x_window = (x * 10) as f32;
                let y_window = (y * 10) as f32;
                canvas
                    .fill_rect(FRect::new(x_window, y_window, 10.0, 10.0))
                    .expect("Error while drawing.");
            }
        }
        canvas.present();

        // keeping 60fps
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
