use sdl3::keyboard::KeyboardState;

use crate::chip8::{Chip8, KeyboardHandler};

pub fn handle_pressed_keys(chip8: &mut Chip8, keyboard: KeyboardState<'_>) {
    /*
     * mappings:
     * CHIP-8       Keyboard
     * 1 2 3 C      1 2 3 4
     * 4 5 6 D      Q W E R
     * 7 8 9 E      A S D F
     * A 0 B F      Z X C V
     */
    chip8.set_key(
        0x4,
        keyboard.is_scancode_pressed(sdl3::keyboard::Scancode::Q),
    );
    chip8.set_key(
        0x5,
        keyboard.is_scancode_pressed(sdl3::keyboard::Scancode::W),
    );
    chip8.set_key(
        0x6,
        keyboard.is_scancode_pressed(sdl3::keyboard::Scancode::E),
    );
    chip8.set_key(
        0xD,
        keyboard.is_scancode_pressed(sdl3::keyboard::Scancode::R),
    );

    chip8.set_key(
        0x7,
        keyboard.is_scancode_pressed(sdl3::keyboard::Scancode::A),
    );
    chip8.set_key(
        0x8,
        keyboard.is_scancode_pressed(sdl3::keyboard::Scancode::S),
    );
    chip8.set_key(
        0x9,
        keyboard.is_scancode_pressed(sdl3::keyboard::Scancode::D),
    );
    chip8.set_key(
        0xE,
        keyboard.is_scancode_pressed(sdl3::keyboard::Scancode::F),
    );

    chip8.set_key(
        0xA,
        keyboard.is_scancode_pressed(sdl3::keyboard::Scancode::Z),
    );
    chip8.set_key(
        0x0,
        keyboard.is_scancode_pressed(sdl3::keyboard::Scancode::X),
    );
    chip8.set_key(
        0xB,
        keyboard.is_scancode_pressed(sdl3::keyboard::Scancode::C),
    );
    chip8.set_key(
        0xF,
        keyboard.is_scancode_pressed(sdl3::keyboard::Scancode::V),
    );
    chip8.set_key(
        0x1,
        keyboard.is_scancode_pressed(sdl3::keyboard::Scancode::_1),
    );
    chip8.set_key(
        0x2,
        keyboard.is_scancode_pressed(sdl3::keyboard::Scancode::_2),
    );
    chip8.set_key(
        0x3,
        keyboard.is_scancode_pressed(sdl3::keyboard::Scancode::_3),
    );
    chip8.set_key(
        0xC,
        keyboard.is_scancode_pressed(sdl3::keyboard::Scancode::_4),
    );
}
