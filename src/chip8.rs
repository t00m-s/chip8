use std::collections::VecDeque;

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
    fn execute(&mut self, instruction: InstructionType);
}

pub trait MachineCycle {
    fn machine_cycle(&mut self);
}
pub struct Chip8 {
    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,
    pc: u16,
    stack: VecDeque<u16>,
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
            stack: VecDeque::with_capacity(16),
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

    pub fn load_program(&mut self, program: &[u8]) {
        let start = 0x200;
        let end = start + program.len();
        assert!(end <= self.memory.len(), "program is too large for memory");
        self.memory[start..end].copy_from_slice(program);
    }

    pub fn registers(&self) -> &[u8; 16] {
        &self.v
    }

    pub fn index_register(&self) -> u16 {
        self.i
    }

    pub fn program_counter(&self) -> u16 {
        self.pc
    }

    pub fn stack_depth(&self) -> usize {
        self.sp
    }

    pub fn display(&self) -> &[bool; 64 * 32] {
        &self.display
    }

    pub fn display_mut(&mut self) -> &mut [bool; 64 * 32] {
        &mut self.display
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
        self.load_program(&rom);
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
        let nnn = opcode & 0x0FFF;
        let nn = (opcode & 0x00FF) as u8;
        let n = (opcode & 0x000F) as u8;
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => InstructionType::Inst00E0,
                0x00EE => InstructionType::Inst00EE,
                _ => panic!("Unsupported 0x0 instruction: {opcode:#06X}"),
            },

            0x1000 => InstructionType::Inst1NNN { nnn: nnn },
            0x2000 => InstructionType::Inst2NNN { nnn: nnn },
            0x3000 => InstructionType::Inst3XNN { x: x, nn: nn },
            0x4000 => InstructionType::Inst4XNN { x: x, nn: nn },

            0x5000 => match opcode & 0x000F {
                0x0 => InstructionType::Inst5XY0 { x: x, y: y },
                _ => panic!("Invalid 0x5 instruction: {opcode:#06X}"),
            },

            0x6000 => InstructionType::Inst6XNN { x: x, nn: nn },
            0x7000 => InstructionType::Inst7XNN { x: x, nn: nn },

            0x8000 => match opcode & 0x000F {
                0x0 => InstructionType::Inst8XY0 { x: x, y: y },
                0x1 => InstructionType::Inst8XY1 { x: x, y: y },
                0x2 => InstructionType::Inst8XY2 { x: x, y: y },
                0x3 => InstructionType::Inst8XY3 { x: x, y: y },
                0x4 => InstructionType::Inst8XY4 { x: x, y: y },
                0x5 => InstructionType::Inst8XY5 { x: x, y: y },
                0x6 => InstructionType::Inst8XY6 { x: x, y: y },
                0x7 => InstructionType::Inst8XY7 { x: x, y: y },
                0xE => InstructionType::Inst8XYE { x: x, y: y },
                _ => panic!("Invalid 0x8 instruction: {opcode:#06X}"),
            },

            0x9000 => match opcode & 0x000F {
                0x0 => InstructionType::Inst9XY0 { x: x, y: y },
                _ => panic!("Invalid 0x9 instruction: {opcode:#06X}"),
            },

            0xA000 => InstructionType::InstANNN { nnn: nnn },
            0xB000 => InstructionType::InstBNNN { nnn: nnn },
            0xD000 => InstructionType::InstDXYN { x: x, y: y, n: n },

            _ => panic!("Unsupported instruction: {opcode:#06X}"),
        }
    }
}

impl Execute for Chip8 {
    fn execute(&mut self, instruction: InstructionType) {
        match instruction {
            InstructionType::Inst00E0 => self.display = [false; 64 * 32],
            InstructionType::Inst00EE => {
                if self.sp != 0 {
                    self.pc = self.stack.pop_front().unwrap();
                    self.sp -= 1;
                } else {
                    panic!("Attempting to return from an empty stack.");
                }
            }
            InstructionType::Inst1NNN { nnn } => {
                self.pc = nnn;
            }
            InstructionType::Inst2NNN { nnn } => {
                if self.stack.len() >= 16 {
                    panic!("stack overflow.");
                }
                self.sp += 1;
                self.stack.push_front(self.pc);
                self.pc = nnn;
            }
            InstructionType::Inst3XNN { x, nn } => {
                if self.v[x] == nn {
                    self.pc += 2;
                }
            }
            InstructionType::Inst4XNN { x, nn } => {
                if self.v[x] != nn {
                    self.pc += 2;
                }
            }
            InstructionType::Inst5XY0 { x, y } => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            InstructionType::Inst6XNN { x, nn } => {
                self.v[x] = nn;
            }
            InstructionType::Inst7XNN { x, nn } => {
                self.v[x] = self.v[x].wrapping_add(nn);
            }
            InstructionType::Inst9XY0 { x, y } => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            InstructionType::InstANNN { nnn } => {
                self.i = nnn;
            }
            InstructionType::InstBNNN { nnn } => {
                self.pc = nnn + self.v[0] as u16;
            }
            InstructionType::Inst8XY0 { x, y } => {
                self.v[x] = self.v[y];
            }
            InstructionType::Inst8XY1 { x, y } => {
                self.v[x] |= self.v[y];
            }
            InstructionType::Inst8XY2 { x, y } => {
                self.v[x] &= self.v[y];
            }
            InstructionType::Inst8XY3 { x, y } => {
                self.v[x] ^= self.v[y];
            }
            InstructionType::Inst8XY4 { x, y } => {
                let (res, overflow) = self.v[x].overflowing_add(self.v[y]);
                self.v[x] = res;
                self.v[0xf] = overflow as u8;
            }
            InstructionType::Inst8XY5 { x, y } => {
                let (res, overflow) = self.v[x].overflowing_sub(self.v[y]);
                self.v[x] = res;
                self.v[0xf] = u8::from(!overflow);
            }
            InstructionType::Inst8XY6 { x, y } => {
                self.v[0xf] = (self.v[x] & 0b1 == 1) as u8;
                self.v[x] = self.v[x].wrapping_shr(1);
            }
            InstructionType::Inst8XY7 { x, y } => {
                let (res, underflow) = self.v[y].overflowing_sub(self.v[x]);
                self.v[x] = res;
                self.v[0xf] = u8::from(!underflow);
            }
            InstructionType::Inst8XYE { x, y } => {
                self.v[0xf] = (self.v[x] & 0b10000000 != 0) as u8;
                self.v[x] = self.v[x].wrapping_shl(1);
            }
            InstructionType::InstDXYN { x, y, n } => todo!(),
        }
    }
}

impl MachineCycle for Chip8 {
    fn machine_cycle(&mut self) {
        let opcode = self.fetch();
        let instruction = self.decode(opcode);
        self.execute(instruction);
    }
}
