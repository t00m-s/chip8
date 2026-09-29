use super::opcode::InstructionType;
use super::rom::read_rom_from_path;
pub trait LoadFont {
    fn load_fonts(&mut self, fonts: &[u8; 80]);
}
pub trait LoadRom {
    fn load_rom(&mut self, rom_path: &str);
}
// maybe pub is not it?
trait Fetch {
    fn fetch(&mut self) -> u16;
}
trait Decode {
    fn decode(&self, opcode: u16) -> InstructionType;
}
trait Execute {
    fn execute(&mut self, instruction: InstructionType, opcode: u16);
}

pub trait MachineCycle {
    fn machine_cycle(&mut self);
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
            pc: 0x200,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: [false; 64 * 32],
            keypad: [false; 16],
        }
    }
}

impl Chip8 {
    pub fn memory(&self) -> &[u8; 4096] {
        &self.memory
    }
}

impl LoadFont for Chip8 {
    fn load_fonts(&mut self, fonts: &[u8; 80]) {
        // fonts start at address 0x50 as a convention.
        // no need to add the if clause, fonts are fixed size and memory is 4kb
        self.memory[0x50..0x50 + fonts.len()].copy_from_slice(fonts)
    }
}

impl LoadRom for Chip8 {
    fn load_rom(&mut self, rom_path: &str) {
        // Safety check: ROM must fit in memory
        // rom loading starts at address 0x200
        let rom = read_rom_from_path(rom_path);
        if rom.len() + 0x200 > self.memory.len() {
            panic!("ROM too large for memory: {} bytes", rom.len());
        }
        self.memory[0x200..0x200 + rom.len()].copy_from_slice(&rom);
    }
}

impl Fetch for Chip8 {
    fn fetch(&mut self) -> u16 {
        let first = self.memory[self.pc as usize];
        let second = self.memory[(self.pc + 1) as usize];
        let opcode = (first as u16) << 8 | second as u16;
        self.pc += 2;
        return opcode;
    }
}

impl Decode for Chip8 {
    fn decode(&self, opcode: u16) -> InstructionType {
        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => InstructionType::Inst00E0,
                0x00EE => InstructionType::Inst00EE,
                _ => panic!("Unsupported 0x0 instruction: {opcode:#06X}"),
            },

            0x1000 => InstructionType::Inst1NNN,
            0x2000 => InstructionType::Inst2NNN,
            0x3000 => InstructionType::Inst3XNN,
            0x4000 => InstructionType::Inst4XNN,

            0x5000 => match opcode & 0x000F {
                0x0 => InstructionType::Inst5XY0,
                _ => panic!("Invalid 0x5 instruction: {opcode:#06X}"),
            },

            0x6000 => InstructionType::Inst6XNN,
            0x7000 => InstructionType::Inst7XNN,

            0x8000 => match opcode & 0x000F {
                0x0 => InstructionType::Inst8XY0,
                0x1 => InstructionType::Inst8XY1,
                0x2 => InstructionType::Inst8XY2,
                0x3 => InstructionType::Inst8XY3,
                0x4 => InstructionType::Inst8XY4,
                0x5 => InstructionType::Inst8XY5,
                0x6 => InstructionType::Inst8XY6,
                0x7 => InstructionType::Inst8XY7,
                0xE => InstructionType::Inst8XYE,
                _ => panic!("Invalid 0x8 instruction: {opcode:#06X}"),
            },

            0x9000 => match opcode & 0x000F {
                0x0 => InstructionType::Inst9XY0,
                _ => panic!("Invalid 0x9 instruction: {opcode:#06X}"),
            },

            0xA000 => InstructionType::InstANNN,
            0xB000 => InstructionType::InstBNNN,

            _ => panic!("Unsupported instruction: {opcode:#06X}"),
        }
    }
}

impl Execute for Chip8 {
    fn execute(&mut self, instruction: InstructionType, opcode: u16) {
        let nnn = opcode & 0x0FFF;
        let nn = (opcode & 0x00FF) as u8;
        let n = (opcode & 0x000F) as u8;
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        match instruction {
            InstructionType::Inst00E0 => todo!(),
            InstructionType::Inst00EE => todo!(),
            InstructionType::Inst1NNN => todo!(),
            InstructionType::Inst2NNN => todo!(),
            InstructionType::Inst3XNN => todo!(),
            InstructionType::Inst4XNN => todo!(),
            InstructionType::Inst5XY0 => todo!(),
            InstructionType::Inst6XNN => todo!(),
            InstructionType::Inst7XNN => todo!(),
            InstructionType::Inst9XY0 => todo!(),
            InstructionType::InstANNN => todo!(),
            InstructionType::InstBNNN => todo!(),
            InstructionType::Inst8XY0 => todo!(),
            InstructionType::Inst8XY1 => todo!(),
            InstructionType::Inst8XY2 => todo!(),
            InstructionType::Inst8XY3 => todo!(),
            InstructionType::Inst8XY4 => todo!(),
            InstructionType::Inst8XY5 => todo!(),
            InstructionType::Inst8XY6 => todo!(),
            InstructionType::Inst8XY7 => todo!(),
            InstructionType::Inst8XYE => todo!(),
        }
    }
}

impl MachineCycle for Chip8 {
    fn machine_cycle(&mut self) {
        let opcode = self.fetch();
        let instruction = self.decode(opcode);
        self.execute(instruction, opcode);
    }
}
