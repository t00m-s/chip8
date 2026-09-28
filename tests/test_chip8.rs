use chip8_emulator::chip8::{Chip8, LoadFont, LoadRom};
use chip8_emulator::constants::FONTS;

#[test]
fn rom_is_loaded_at_0x200() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/roms/ibm-logo.ch8");
    let expected = std::fs::read(path).unwrap();

    let mut chip8 = Chip8::default();
    chip8.load_rom(path);

    assert_eq!(
        &chip8.memory()[0x200..0x200 + expected.len()],
        expected.as_slice()
    );
}

#[test]
fn fonts_are_loaded_at_0x50() {
    let mut chip8 = Chip8::default();
    chip8.load_fonts(&FONTS);

    assert_eq!(&chip8.memory()[0x50..0x50 + FONTS.len()], FONTS.as_slice());
}
