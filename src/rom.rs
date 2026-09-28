use std::fs::File;
use std::io::Read;
use std::path::Path;
pub fn read_rom_from_path(rom_path: &str) -> Vec<u8> {
    if !Path::new(rom_path).exists() {
        panic!("Nothing in {rom_path}");
    }

    let mut rom = File::open(rom_path).expect("Failed to open file");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer)
        .expect("Failed to read file contents");

    return buffer;
}
