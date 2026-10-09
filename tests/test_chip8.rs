use chip8_emulator::chip8::{
    Chip8, KeyboardHandler, LoadFont, LoadRom, MachineCycle, TimerHandler,
};
use chip8_emulator::constants::FONTS;

fn mock_chip8(opcodes: &[u16]) -> Chip8 {
    let program: Vec<u8> = opcodes
        .iter()
        .flat_map(|opcode| opcode.to_be_bytes())
        .collect();

    let mut chip8 = Chip8::default();
    chip8.load_fonts(&FONTS);
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

#[test]
fn pressed_key_causes_ex9e_to_skip_the_next_instruction() {
    let mut chip8 = mock_chip8(&[0x610A, 0xE19E]);
    chip8.set_key(0xA, true);

    chip8.machine_cycle();
    chip8.machine_cycle();

    assert_eq!(chip8.program_counter(), 0x206);
}

#[test]
fn unpressed_key_causes_exa1_to_skip_the_next_instruction() {
    let mut chip8 = mock_chip8(&[0x610A, 0xE1A1]);

    chip8.machine_cycle();
    chip8.machine_cycle();

    assert_eq!(chip8.program_counter(), 0x206);
}

#[test]
fn fx0a_halts_until_a_new_key_press_and_stores_the_key() {
    let mut chip8 = mock_chip8(&[0xF30A, 0x6401]);

    chip8.machine_cycle();
    assert_eq!(chip8.program_counter(), 0x202);

    chip8.machine_cycle();
    assert_eq!(chip8.program_counter(), 0x202);
    assert_eq!(chip8.registers()[4], 0);

    chip8.set_key(0xC, true);
    assert_eq!(chip8.registers()[3], 0xC);

    chip8.machine_cycle();
    assert_eq!(chip8.program_counter(), 0x204);
    assert_eq!(chip8.registers()[4], 1);
}

#[test]
fn fx0a_does_not_accept_a_key_that_was_already_held() {
    let mut chip8 = mock_chip8(&[0xF20A, 0x6301]);
    chip8.set_key(0x5, true);

    chip8.machine_cycle();
    chip8.set_key(0x5, true);
    chip8.machine_cycle();

    assert_eq!(chip8.program_counter(), 0x202);
    assert_eq!(chip8.registers()[2], 0);

    chip8.set_key(0x5, false);
    chip8.set_key(0x5, true);
    assert_eq!(chip8.registers()[2], 0x5);
}

#[test]
fn fx15_sets_delay_timer_and_fx07_reads_without_consuming_it() {
    for value in [0_u16, 1, 0x80, 0xFF] {
        let mut chip8 = mock_chip8(&[0x6300 | value, 0xF315, 0x6300, 0xF707, 0xF807]);

        for _ in 0..5 {
            chip8.machine_cycle();
        }

        let mut expected_registers = [0; 16];
        expected_registers[7] = value as u8;
        expected_registers[8] = value as u8;
        assert_eq!(
            *chip8.registers(),
            expected_registers,
            "reading delay timer value {value}"
        );
        assert_eq!(chip8.program_counter(), 0x20A);
        assert!(!chip8.is_audio_active());
    }
}

#[test]
fn delay_timer_decrements_once_per_tick_and_stops_at_zero() {
    let mut chip8 = mock_chip8(&[0x6103, 0xF115, 0xF207, 0x1204]);
    chip8.machine_cycle();
    chip8.machine_cycle();

    for expected in [3, 2, 1, 0, 0] {
        chip8.machine_cycle();
        assert_eq!(chip8.registers()[2], expected);

        chip8.machine_cycle(); // Jump back to the delay-timer read.
        chip8.handle_timers();
    }
}

#[test]
fn fx18_activates_sound_for_the_requested_number_of_ticks() {
    for value in [0_u16, 1, 3, 0xFF] {
        let mut chip8 = mock_chip8(&[0x6900 | value, 0xF918, 0x6900]);
        assert!(!chip8.is_audio_active());

        chip8.machine_cycle();
        chip8.machine_cycle();
        assert_eq!(chip8.is_audio_active(), value > 0);

        chip8.machine_cycle(); // Changing V9 must not change the loaded timer.
        assert_eq!(chip8.is_audio_active(), value > 0);

        for elapsed in 1..=value {
            chip8.handle_timers();
            assert_eq!(
                chip8.is_audio_active(),
                elapsed < value,
                "sound timer {value} after {elapsed} ticks"
            );
        }

        for _ in 0..3 {
            chip8.handle_timers();
            assert!(!chip8.is_audio_active(), "zero sound timer must not wrap");
        }
    }
}

#[test]
fn fx15_replaces_the_remaining_delay_timer() {
    for value in [0_u16, 1, 5] {
        let mut chip8 = mock_chip8(&[0x6403, 0xF415, 0x6400 | value, 0xF415, 0xF507]);
        chip8.machine_cycle();
        chip8.machine_cycle();
        chip8.handle_timers(); // Two ticks remain before the timer is replaced.

        for _ in 0..3 {
            chip8.machine_cycle();
        }

        assert_eq!(chip8.registers()[5], value as u8);
        assert_eq!(chip8.registers()[4], value as u8);
    }
}

#[test]
fn fx18_replaces_the_remaining_sound_timer() {
    for value in [0_u16, 1, 5] {
        let mut chip8 = mock_chip8(&[0x6403, 0xF418, 0x6400 | value, 0xF418]);
        chip8.machine_cycle();
        chip8.machine_cycle();
        chip8.handle_timers();
        assert!(chip8.is_audio_active());

        chip8.machine_cycle();
        chip8.machine_cycle();
        assert_eq!(chip8.is_audio_active(), value > 0);

        for elapsed in 1..=value {
            chip8.handle_timers();
            assert_eq!(
                chip8.is_audio_active(),
                elapsed < value,
                "replacement sound timer {value} after {elapsed} ticks"
            );
        }
    }
}

#[test]
fn cpu_cycles_do_not_decrement_timers() {
    let mut chip8 = mock_chip8(&[0x6102, 0xF115, 0xF118, 0xF207, 0x1206]);
    for _ in 0..3 {
        chip8.machine_cycle();
    }

    for _ in 0..100 {
        chip8.machine_cycle();
    }

    assert_eq!(chip8.registers()[2], 2);
    assert!(chip8.is_audio_active());

    chip8.handle_timers();
    assert!(chip8.is_audio_active());
    chip8.handle_timers();
    assert!(!chip8.is_audio_active());

    chip8.machine_cycle();
    assert_eq!(chip8.registers()[2], 0);
}

#[test]
fn timer_ticks_do_not_execute_cpu_instructions() {
    let mut chip8 = mock_chip8(&[0x6103, 0xF115, 0xF118, 0x6201, 0xF307]);
    for _ in 0..3 {
        chip8.machine_cycle();
    }
    let registers_before_ticks = *chip8.registers();

    chip8.handle_timers();
    chip8.handle_timers();

    assert_eq!(chip8.program_counter(), 0x206);
    assert_eq!(*chip8.registers(), registers_before_ticks);
    assert!(chip8.is_audio_active());

    chip8.machine_cycle();
    chip8.machine_cycle();
    assert_eq!(chip8.registers()[2], 1);
    assert_eq!(chip8.registers()[3], 1);

    chip8.handle_timers();
    assert!(!chip8.is_audio_active());
}

#[test]
fn delay_and_sound_timers_count_down_independently() {
    for (delay, sound) in [(1_u16, 3_u16), (3, 1)] {
        let mut chip8 = mock_chip8(&[
            0x6100 | delay,
            0xF115,
            0x6200 | sound,
            0xF218,
            0xF307,
            0x1208,
        ]);
        for _ in 0..4 {
            chip8.machine_cycle();
        }

        for elapsed in 0..=4 {
            chip8.machine_cycle();
            assert_eq!(
                chip8.registers()[3],
                delay.saturating_sub(elapsed) as u8,
                "delay timer {delay} after {elapsed} ticks"
            );
            assert_eq!(
                chip8.is_audio_active(),
                elapsed < sound,
                "sound timer {sound} after {elapsed} ticks"
            );

            chip8.machine_cycle(); // Jump back to the delay-timer read.
            chip8.handle_timers();
        }
    }
}

#[test]
fn timers_continue_while_fx0a_waits_for_a_key() {
    let mut chip8 = mock_chip8(&[0x6104, 0xF115, 0x6202, 0xF218, 0xF40A, 0xF507]);
    for _ in 0..5 {
        chip8.machine_cycle();
    }
    let registers_while_waiting = *chip8.registers();

    for sound_active in [true, false, false] {
        chip8.machine_cycle();
        chip8.handle_timers();

        assert_eq!(chip8.program_counter(), 0x20A);
        assert_eq!(*chip8.registers(), registers_while_waiting);
        assert_eq!(chip8.is_audio_active(), sound_active);
    }

    chip8.set_key(0xA, true);
    chip8.machine_cycle();

    assert_eq!(chip8.registers()[4], 0xA);
    assert_eq!(chip8.registers()[5], 1);
    assert_eq!(chip8.program_counter(), 0x20C);
}

#[test]
fn random_byte_is_masked_by_nn() {
    for _ in 0..64 {
        let mut chip8 = mock_chip8(&[0xC3A5]);

        chip8.machine_cycle();

        assert_eq!(chip8.registers()[3] & !0xA5, 0);
    }

    let mut zero_mask = mock_chip8(&[0xC700]);
    zero_mask.machine_cycle();
    assert_eq!(zero_mask.registers()[7], 0);
}

#[test]
fn add_register_to_index_register() {
    let mut chip8 = mock_chip8(&[0xA345, 0x61AB, 0xF11E]);

    for _ in 0..3 {
        chip8.machine_cycle();
    }

    assert_eq!(chip8.index_register(), 0x3F0);
}

#[test]
fn font_character_sets_index_to_its_sprite() {
    for digit in 0_u16..=0xF {
        let mut chip8 = mock_chip8(&[0x6300 | digit, 0xF329]);

        chip8.machine_cycle();
        chip8.machine_cycle();

        let expected_address = 0x50 + digit * 5;
        assert_eq!(chip8.index_register(), expected_address);
        assert_eq!(
            &chip8.memory()[expected_address as usize..expected_address as usize + 5],
            &FONTS[digit as usize * 5..digit as usize * 5 + 5]
        );
    }
}

#[test]
fn binary_coded_decimal_stores_all_three_digits() {
    for (value, expected) in [
        (0_u16, [0, 0, 0]),
        (7, [0, 0, 7]),
        (42, [0, 4, 2]),
        (255, [2, 5, 5]),
    ] {
        let mut chip8 = mock_chip8(&[0xA300, 0x6100 | value, 0xF133]);

        for _ in 0..3 {
            chip8.machine_cycle();
        }

        assert_eq!(&chip8.memory()[0x300..0x303], &expected);
        assert_eq!(chip8.index_register(), 0x300);
    }
}

#[test]
fn store_registers_writes_v0_through_vx_inclusively() {
    let mut chip8 = mock_chip8(&[0x600A, 0x611B, 0x622C, 0xA300, 0xF255]);

    for _ in 0..5 {
        chip8.machine_cycle();
    }

    assert_eq!(&chip8.memory()[0x300..0x303], &[0x0A, 0x1B, 0x2C]);
    assert_eq!(chip8.index_register(), 0x300);
}

#[test]
fn load_registers_reads_v0_through_vx_inclusively() {
    let mut chip8 = mock_chip8(&[
        0x600A, 0x611B, 0x622C, 0xA300, 0xF255, 0x6000, 0x6100, 0x6200, 0xF265,
    ]);

    for _ in 0..9 {
        chip8.machine_cycle();
    }

    assert_eq!(&chip8.registers()[0..3], &[0x0A, 0x1B, 0x2C]);
    assert_eq!(chip8.index_register(), 0x300);
}
