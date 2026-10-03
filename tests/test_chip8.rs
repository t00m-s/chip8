use chip8_emulator::chip8::{Chip8, LoadFont, LoadRom, MachineCycle};
use chip8_emulator::constants::FONTS;

fn mock_chip8(opcodes: &[u16]) -> Chip8 {
    let program: Vec<u8> = opcodes
        .iter()
        .flat_map(|opcode| opcode.to_be_bytes())
        .collect();

    let mut chip8 = Chip8::default();
    chip8.load_program(&program);
    chip8
}

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

#[test]
fn clear_screen_clears_every_pixel() {
    let mut chip8 = mock_chip8(&[0x00E0]);
    chip8.display_mut().fill(true);

    chip8.machine_cycle();

    assert!(chip8.display().iter().all(|pixel| !pixel));
    assert_eq!(chip8.program_counter(), 0x202);
}

#[test]
fn jump_sets_program_counter() {
    let mut chip8 = mock_chip8(&[0x134A]);

    chip8.machine_cycle();

    assert_eq!(chip8.program_counter(), 0x34A);
}

#[test]
fn call_and_return_restore_the_return_address() {
    let mut chip8 = mock_chip8(&[0x2204, 0x0000, 0x00EE]);

    chip8.machine_cycle();
    assert_eq!(chip8.program_counter(), 0x204);
    assert_eq!(chip8.stack_depth(), 1);

    chip8.machine_cycle();
    assert_eq!(chip8.program_counter(), 0x202);
    assert_eq!(chip8.stack_depth(), 0);
}

#[test]
fn nested_calls_return_in_last_in_first_out_order() {
    let mut chip8 = mock_chip8(&[0x2206, 0x0000, 0x0000, 0x220A, 0x00EE, 0x00EE]);

    chip8.machine_cycle();
    chip8.machine_cycle();
    chip8.machine_cycle();
    assert_eq!(chip8.program_counter(), 0x208);

    chip8.machine_cycle();
    assert_eq!(chip8.program_counter(), 0x202);
    assert_eq!(chip8.stack_depth(), 0);
}

#[test]
#[should_panic(expected = "Attempting to return from an empty stack")]
fn return_panics_when_stack_is_empty() {
    let mut chip8 = mock_chip8(&[0x00EE]);
    chip8.machine_cycle();
}

#[test]
#[should_panic(expected = "stack overflow")]
fn call_panics_when_stack_is_full() {
    let mut chip8 = mock_chip8(&[0x2200]);

    for _ in 0..17 {
        chip8.machine_cycle();
    }
}

#[test]
fn skip_if_register_equals_constant_handles_both_cases() {
    let mut equal = mock_chip8(&[0x61AB, 0x31AB]);
    equal.machine_cycle();
    equal.machine_cycle();
    assert_eq!(equal.program_counter(), 0x206);

    let mut unequal = mock_chip8(&[0x61AA, 0x31AB]);
    unequal.machine_cycle();
    unequal.machine_cycle();
    assert_eq!(unequal.program_counter(), 0x204);
}

#[test]
fn skip_if_register_differs_from_constant_handles_both_cases() {
    let mut unequal = mock_chip8(&[0x61AA, 0x41AB]);
    unequal.machine_cycle();
    unequal.machine_cycle();
    assert_eq!(unequal.program_counter(), 0x206);

    let mut equal = mock_chip8(&[0x61AB, 0x41AB]);
    equal.machine_cycle();
    equal.machine_cycle();
    assert_eq!(equal.program_counter(), 0x204);
}

#[test]
fn skip_if_registers_are_equal_handles_both_cases() {
    let mut equal = mock_chip8(&[0x610A, 0x620A, 0x5120]);
    equal.machine_cycle();
    equal.machine_cycle();
    equal.machine_cycle();
    assert_eq!(equal.program_counter(), 0x208);

    let mut unequal = mock_chip8(&[0x610A, 0x620B, 0x5120]);
    unequal.machine_cycle();
    unequal.machine_cycle();
    unequal.machine_cycle();
    assert_eq!(unequal.program_counter(), 0x206);
}

#[test]
fn set_register_assigns_constant() {
    let mut chip8 = mock_chip8(&[0x6ACD]);

    chip8.machine_cycle();

    assert_eq!(chip8.registers()[0xA], 0xCD);
}

#[test]
fn add_constant_updates_register() {
    let mut chip8 = mock_chip8(&[0x610A, 0x7105]);

    chip8.machine_cycle();
    chip8.machine_cycle();

    assert_eq!(chip8.registers()[1], 0x0F);
}

#[test]
fn add_constant_wraps_on_overflow() {
    let mut chip8 = mock_chip8(&[0x61FA, 0x710A]);

    chip8.machine_cycle();
    chip8.machine_cycle();

    assert_eq!(chip8.registers()[1], 0x04);
}

#[test]
fn skip_if_registers_differ_handles_both_cases() {
    let mut unequal = mock_chip8(&[0x610A, 0x620B, 0x9120]);
    unequal.machine_cycle();
    unequal.machine_cycle();
    unequal.machine_cycle();
    assert_eq!(unequal.program_counter(), 0x208);

    let mut equal = mock_chip8(&[0x610A, 0x620A, 0x9120]);
    equal.machine_cycle();
    equal.machine_cycle();
    equal.machine_cycle();
    assert_eq!(equal.program_counter(), 0x206);
}

#[test]
fn set_index_register_assigns_address() {
    let mut chip8 = mock_chip8(&[0xAABC]);

    chip8.machine_cycle();

    assert_eq!(chip8.index_register(), 0xABC);
}

#[test]
fn jump_with_offset_adds_v0() {
    let mut chip8 = mock_chip8(&[0x6005, 0xB300]);

    chip8.machine_cycle();
    chip8.machine_cycle();

    assert_eq!(chip8.program_counter(), 0x305);
}

#[test]
#[should_panic(expected = "Invalid 0x5 instruction")]
fn invalid_5xyn_opcode_panics() {
    let mut chip8 = mock_chip8(&[0x5121]);
    chip8.machine_cycle();
}

#[test]
#[should_panic(expected = "Invalid 0x9 instruction")]
fn invalid_9xyn_opcode_panics() {
    let mut chip8 = mock_chip8(&[0x9121]);
    chip8.machine_cycle();
}
