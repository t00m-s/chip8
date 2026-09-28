pub trait LoadFont {
    fn load_fonts(&mut self, fonts: &[u8; 80]);
}
pub trait LoadRom {
    fn load_rom(&mut self, rom: &[u8]);
}
pub struct Chip8 {
    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,
    pc: u16,
    stack: [u16; 16],
    sp: usize,
    delay_timer: u8,
    sound_timer: u8,
    display: [bool; 64 * 32],
    keypad: [bool; 16],
}
impl Default for Chip8 {
    fn default() -> Self {
        Chip8 {
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: [false; 64 * 32],
            keypad: [false; 16],
        }
    }
}
impl LoadFont for Chip8 {
    fn load_fonts(&mut self, fonts: &[u8; 80]) {
        // fonts start at address 0x50 as a convention.
        self.memory[0x50..0x50 + fonts.len()].copy_from_slice(fonts)
    }
}

impl LoadRom for Chip8 {
    fn load_rom(&mut self, rom: &[u8]) {
        // rom loading starts at address 0x200
        self.memory[0x200..0x200 + rom.len()].copy_from_slice(rom);
    }
}
