use chip8_emulator::audio::Beeper;
use chip8_emulator::chip8::{Chip8, LoadFont, LoadRom, MachineCycle, TimerHandler};
use chip8_emulator::constants;
use chip8_emulator::keypad::handle_pressed_keys;
use sdl3::pixels::Color;
use sdl3::render::FRect;
use std::ffi::OsString;
use std::time::{Duration, Instant};

fn main() {
    //
    let _args: Vec<OsString> = std::env::args_os().collect();
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
    let audio_subsystem = sdl_context.audio().unwrap();
    let mut beeper = Beeper::new(&audio_subsystem);
    chip8.load_fonts(&constants::FONTS);
    chip8.load_rom("roms/space-invaders.ch8");
    // 700 instructions/s
    let cpu_speed = Duration::from_secs_f64(1.0 / 700.0);
    // 60Hz
    let timers_delay = Duration::from_secs_f64(1.0 / 60.0);

    let starting_timer = Instant::now();
    let mut next_cpu_tick = starting_timer + cpu_speed;
    let mut next_60hz_tick = starting_timer + timers_delay;
    'game_loop: loop {
        for event in event_pump.poll_iter() {
            if let sdl3::event::Event::Quit { timestamp: _ } = event {
                drop(canvas);
                break 'game_loop;
            }
        }
        let keyboard = event_pump.keyboard_state();
        handle_pressed_keys(&mut chip8, keyboard);
        let now = Instant::now();
        while now >= next_cpu_tick {
            chip8.machine_cycle();
            next_cpu_tick += cpu_speed;
        }
        // This part should run at 60Hz
        while now >= next_60hz_tick {
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            canvas.set_draw_color(Color::RGB(255, 255, 255));
            for (i, &pixel) in chip8.display().iter().enumerate() {
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
            beeper.set_active(chip8.is_audio_active());
            chip8.handle_timers();
            next_60hz_tick += timers_delay;
        }
        let next_tick = next_cpu_tick.min(next_60hz_tick);
        std::thread::sleep(next_tick.saturating_duration_since(Instant::now()));
    }
}
