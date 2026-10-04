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

#[test]
fn copy_register_copies_vy_into_vx() {
    let mut chip8 = mock_chip8(&[0x61AB, 0x8210]);

    chip8.machine_cycle();
    chip8.machine_cycle();

    assert_eq!(chip8.registers()[2], 0xAB);
}

#[test]
fn bitwise_or_updates_vx() {
    let mut chip8 = mock_chip8(&[0x610F, 0x62F0, 0x8121]);

    chip8.machine_cycle();
    chip8.machine_cycle();
    chip8.machine_cycle();

    assert_eq!(chip8.registers()[1], 0xFF);
}

#[test]
fn bitwise_and_updates_vx() {
    let mut chip8 = mock_chip8(&[0x61AA, 0x620F, 0x8122]);

    chip8.machine_cycle();
    chip8.machine_cycle();
    chip8.machine_cycle();

    assert_eq!(chip8.registers()[1], 0x0A);
}

#[test]
fn bitwise_xor_updates_vx() {
    let mut chip8 = mock_chip8(&[0x61AA, 0x620F, 0x8123]);

    chip8.machine_cycle();
    chip8.machine_cycle();
    chip8.machine_cycle();

    assert_eq!(chip8.registers()[1], 0xA5);
}

#[test]
fn add_registers_sets_carry_flag() {
    let mut without_carry = mock_chip8(&[0x6F01, 0x61A0, 0x620F, 0x8124]);
    for _ in 0..4 {
        without_carry.machine_cycle();
    }
    assert_eq!(without_carry.registers()[1], 0xAF);
    assert_eq!(without_carry.registers()[0xF], 0);

    let mut with_carry = mock_chip8(&[0x6F00, 0x61FA, 0x620A, 0x8124]);
    for _ in 0..4 {
        with_carry.machine_cycle();
    }
    assert_eq!(with_carry.registers()[1], 0x04);
    assert_eq!(with_carry.registers()[0xF], 1);
}

#[test]
fn subtract_vy_from_vx_sets_not_borrow_flag() {
    let mut no_borrow = mock_chip8(&[0x610A, 0x6203, 0x8125]);
    for _ in 0..3 {
        no_borrow.machine_cycle();
    }
    assert_eq!(no_borrow.registers()[1], 0x07);
    assert_eq!(no_borrow.registers()[0xF], 1);

    let mut equal = mock_chip8(&[0x610A, 0x620A, 0x8125]);
    for _ in 0..3 {
        equal.machine_cycle();
    }
    assert_eq!(equal.registers()[1], 0);
    assert_eq!(equal.registers()[0xF], 1);

    let mut borrow = mock_chip8(&[0x6103, 0x620A, 0x8125]);
    for _ in 0..3 {
        borrow.machine_cycle();
    }
    assert_eq!(borrow.registers()[1], 0xF9);
    assert_eq!(borrow.registers()[0xF], 0);
}

#[test]
fn shift_right_stores_lost_bit_in_vf() {
    let mut even = mock_chip8(&[0x6104, 0x8126]);
    even.machine_cycle();
    even.machine_cycle();
    assert_eq!(even.registers()[1], 0x02);
    assert_eq!(even.registers()[0xF], 0);

    let mut odd = mock_chip8(&[0x6105, 0x8126]);
    odd.machine_cycle();
    odd.machine_cycle();
    assert_eq!(odd.registers()[1], 0x02);
    assert_eq!(odd.registers()[0xF], 1);
}

#[test]
fn subtract_vx_from_vy_sets_not_borrow_flag() {
    let mut no_borrow = mock_chip8(&[0x6103, 0x620A, 0x8127]);
    for _ in 0..3 {
        no_borrow.machine_cycle();
    }
    assert_eq!(no_borrow.registers()[1], 0x07);
    assert_eq!(no_borrow.registers()[0xF], 1);

    let mut equal = mock_chip8(&[0x610A, 0x620A, 0x8127]);
    for _ in 0..3 {
        equal.machine_cycle();
    }
    assert_eq!(equal.registers()[1], 0);
    assert_eq!(equal.registers()[0xF], 1);

    let mut borrow = mock_chip8(&[0x610A, 0x6203, 0x8127]);
    for _ in 0..3 {
        borrow.machine_cycle();
    }
    assert_eq!(borrow.registers()[1], 0xF9);
    assert_eq!(borrow.registers()[0xF], 0);
}

#[test]
fn shift_left_stores_lost_bit_in_vf() {
    let mut without_overflow = mock_chip8(&[0x6140, 0x812E]);
    without_overflow.machine_cycle();
    without_overflow.machine_cycle();
    assert_eq!(without_overflow.registers()[1], 0x80);
    assert_eq!(without_overflow.registers()[0xF], 0);

    let mut with_overflow = mock_chip8(&[0x6181, 0x812E]);
    with_overflow.machine_cycle();
    with_overflow.machine_cycle();
    assert_eq!(with_overflow.registers()[1], 0x02);
    assert_eq!(with_overflow.registers()[0xF], 1);
}

#[test]
fn draw_sprite_places_each_bit_at_the_correct_screen_position() {
    let mut chip8 = mock_chip8(&[0x6102, 0x6203, 0xA208, 0xD121, 0xA500]);

    for _ in 0..4 {
        chip8.machine_cycle();
    }

    let row_start = 3 * 64;
    let expected_pixels = [2, 4, 7, 9];

    for column in 0..64 {
        assert_eq!(
            chip8.display()[row_start + column],
            expected_pixels.contains(&column),
            "unexpected pixel state at ({column}, 3)"
        );
    }
    assert_eq!(chip8.registers()[0xF], 0);
}

#[test]
fn drawing_the_same_sprite_twice_erases_it_and_sets_collision_flag() {
    let mut chip8 = mock_chip8(&[0x6102, 0x6203, 0xA20A, 0xD121, 0xD121, 0xA500]);

    for _ in 0..4 {
        chip8.machine_cycle();
    }
    assert_eq!(chip8.registers()[0xF], 0);

    chip8.machine_cycle();

    assert!(chip8.display().iter().all(|pixel| !pixel));
    assert_eq!(chip8.registers()[0xF], 1);
}
