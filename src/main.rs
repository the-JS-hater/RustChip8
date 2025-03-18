#![allow(dead_code)]
#![allow(unused_variables)]

use raylib::ffi::TraceLogLevel::LOG_NONE;
use raylib::prelude::*;
use std::io::{Read, stdin};

const SCREEN_WIDTH: i32 = 64;
const SCREEN_HEIGHT: i32 = 32;
const SQUARE_SIZE: i32 = 16;

struct Chip8 {
    mem: [u8; 4096],
    pc: u16,
    reg_i: u16,
    stack: Vec<u16>,
    registers: [u8; 16],
    display: [[bool; SCREEN_WIDTH as usize]; SCREEN_HEIGHT as usize],
    delay_timer: u8,
    sound_timer: u8,
}

impl Chip8 {
    fn new() -> Self {
        Chip8 {
            mem: [0; 4096],
            pc: 0x200,
            reg_i: 0,
            stack: Vec::new(),
            registers: [0; 16],
            display: [[false; SCREEN_WIDTH as usize]; SCREEN_HEIGHT as usize],
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    fn fetch(&mut self) -> u16 {
        let byte1 = self.mem[self.pc as usize];
        let byte2 = self.mem[(self.pc + 1) as usize];
        self.pc += 2;

        return (byte1 as u16) << 8 | (byte2 as u16);
    }

    fn execute(&mut self, instruction: u16) {
        let instruction = split_nibbles(instruction);

        match instruction {
            [0x0, 0x0, 0xE, 0x0] => self.clear_screen(),
            [0x0, 0x0, 0xE, 0xE] => {
                self.pc = self.stack.pop().expect("Tried to pop at empty stack!");
            }
            [0x1, nibb1, nibb2, nibb3] => {
                let addr = conc_nibbles(&[nibb1, nibb2, nibb3]);
                self.pc = addr;
            }
            [0x2, nibb1, nibb2, nibb3] => {
                self.stack.push(self.pc);
                let addr = conc_nibbles(&[nibb1, nibb2, nibb3]);
                self.pc = addr;
            }
            [0x3, x, nibb1, nibb2] => {
                let val = nibb1 << 4 | nibb2;
                if self.registers[x as usize] == val {
                    self.pc += 2
                };
            }
            [0x4, x, nibb1, nibb2] => {
                let val = nibb1 << 4 | nibb2;
                if self.registers[x as usize] != val {
                    self.pc += 2
                };
            }
            [0x5, x, y, 0x0] => {
                if self.registers[x as usize] == self.registers[y as usize] {
                    self.pc += 2;
                }
            }
            [0x6, x, nibb1, nibb2] => {
                let idx = x as usize;
                let val = nibb1 << 4 | nibb2;
                self.registers[idx] = val;
            }
            [0x7, x, nibb1, nibb2] => {
                let idx = x as usize;
                let val = nibb1 << 4 | nibb2;
                self.registers[idx] += val;
            }
            [0x8, x, y, 0x0] => {
                self.registers[x as usize] = self.registers[y as usize];
            }
            [0x8, x, y, 0x1] => {
                self.registers[x as usize] |= self.registers[y as usize];
            }
            [0x8, x, y, 0x2] => {
                self.registers[x as usize] &= self.registers[y as usize];
            }
            [0x8, x, y, 0x3] => {
                self.registers[x as usize] ^= self.registers[y as usize];
            }
            [0x8, x, y, 0x4] => {
                let x_val = self.registers[x as usize] as u16;
                let y_val = self.registers[y as usize] as u16;
                let sum = x_val + y_val;
                if sum > 255 {
                    self.registers[0xF] = 1
                };
                self.registers[x as usize] += self.registers[y as usize];
            }
            [0x8, x, y, 0x5] => {
                self.registers[x as usize] -= self.registers[y as usize];
            }
            [0x8, x, y, 0x6] => {
                //NOTE: ambigious!
                todo!();
            }
            [0x8, x, y, 0x7] => {
                let res = self.registers[x as usize] - self.registers[y as usize];
                self.registers[x as usize] = res;
            }
            [0x8, x, y, 0xE] => {
                //NOTE: ambigious!
                todo!();
            }
            [0x9, x, y, 0x0] => {
                if self.registers[x as usize] != self.registers[y as usize] {
                    self.pc += 2;
                }
            }
            [0xA, nibb1, nibb2, nibb3] => {
                let addr = conc_nibbles(&[nibb1, nibb2, nibb3]);
                self.reg_i = addr;
            }
            [0xB, nibb1, nibb2, nibb3] => {
                //NOTE: ambigious!
                todo!()
            }
            [0xC, x, nibb1, nibb2] => {
                //NOTE: ambigious!
                todo!();
            }
            [0xD, x, y, n] => {
                let x_idx = x as usize;
                let y_idx = y as usize;
                let x_pos = self.registers[x_idx] as usize % 64;
                let y_pos = self.registers[y_idx] as usize % 32;
                self.registers[0xF] = 0;

                for i in 0..n {
                    let addr = self.reg_i as usize + i as usize;
                    let sprite_data = self.mem[addr];

                    for j in 0..8 {
                        let pixel = (sprite_data >> (7 - j)) & 1 != 0;
                        let screen_x = (x_pos + j) % 64;
                        let screen_y = (y_pos + i as usize) % 32;

                        if pixel {
                            if self.display[screen_y][screen_x] {
                                self.registers[0xF] = 1;
                            }
                            self.display[screen_y][screen_x] ^= true;
                        }
                    }
                }
            }
            [0xE, x, 0x9, 0xE] => {
                todo!();
                //Skip if key_pressed() == VX
            }
            [0xE, x, 0xA, 0x1] => {
                todo!();
                //Skip if key_pressed() != VX
            }
            [0xF, x, 0x0, 0x7] => {
                self.registers[x as usize] = self.delay_timer;
            }
            [0xF, x, 0x1, 0x5] => {
                self.delay_timer = self.registers[x as usize];
            }
            [0xF, x, 0x1, 0x8] => {
                self.sound_timer = self.registers[x as usize];
            }
            [0xF, x, 0x1, 0xE] => {
                //TODO: make overflow, VF = 1, configurable
                //Overflow would be above 0x1000 (normal addr space)
                self.reg_i += self.registers[x as usize] as u16;
            }
            [0xF, x, 0x0, 0xA] => {
                loop {
                    // if key_pressed() -> store value of keypress in VX, then break
                    todo!();
                }
            }
            [0xF, x, 0x2, 0x9] => {
                // need to figure out fonts first
                todo!();

                // The index register I is set to the address of the hexadecimal character in VX. You
                // probably stored that font somewhere in the first 512 bytes of memory, so now you
                // just need to point I to the right character.
            }
            [0xF, x, 0x3, 0x3] => {
                let val = self.registers[x as usize];
                let digit1 = val / 100_u8;
                let digit2 = (val % 100 - val % 10) / 10_u8;
                let digit3 = val % 10;
                self.mem[self.reg_i as usize] = digit1;
                self.mem[self.reg_i as usize + 1] = digit2;
                self.mem[self.reg_i as usize + 2] = digit3;
            }
            [0xF, x, 0x5, 0x5] => {
                //NOTE: ambigious instruction
                todo!()
            }
            [0xF, x, 0x6, 0x5] => {
                //NOTE: ambigious instruction
                todo!()
            }
            _ => {
                println!("ERROR: UNKNOWN INSTRUCTION {instruction:#?}");
                panic!();
            }
        }
    }

    fn push_stack(&mut self, addr: u16) {
        self.stack.push(addr);
    }

    fn pop_stack(&mut self) -> u16 {
        return self
            .stack
            .pop()
            .expect("Program tried to pop an empty stack");
    }

    fn decrement_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn load_rom(&mut self, rom_data: &[u8]) {
        self.mem[0x200..0x200 + rom_data.len()].copy_from_slice(rom_data);
    }

    fn clear_screen(&mut self) {
        self.display = [[false; SCREEN_WIDTH as usize]; SCREEN_HEIGHT as usize];
    }

    fn draw_display(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::BLACK);
        for i in 0..(SCREEN_HEIGHT * SCREEN_WIDTH) {
            let x = i % SCREEN_WIDTH;
            let y = i / SCREEN_WIDTH;

            if !self.display[y as usize][x as usize] {
                continue;
            }

            d.draw_rectangle(
                x * SQUARE_SIZE,
                y * SQUARE_SIZE,
                SQUARE_SIZE,
                SQUARE_SIZE,
                Color::GREEN,
            );
        }
    }
}

fn split_nibbles(word: u16) -> [u8; 4] {
    [
        ((word >> 12) & 0xF) as u8,
        ((word >> 8) & 0xF) as u8,
        ((word >> 4) & 0xF) as u8,
        (word & 0xF) as u8,
    ]
}

fn conc_nibbles(nibbs: &[u8]) -> u16 {
    let mut addr: u16 = 0;
    for nibb in nibbs {
        addr <<= 4;
        addr |= *nibb as u16;
    }

    return addr;
}

fn main() {
    let mut chip8 = Chip8::new();

    let mut buffer = Vec::new();
    let lines = stdin()
        .read_to_end(&mut buffer)
        .expect("Failed to read ROM file");

    chip8.load_rom(&buffer);

    set_trace_log(LOG_NONE);

    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH * SQUARE_SIZE, SCREEN_HEIGHT * SQUARE_SIZE)
        .title("CHIP-8 Emulator")
        .build();

    while !rl.window_should_close() {
        let instruction = chip8.fetch();
        chip8.execute(instruction);
        chip8.draw_display(&mut rl, &thread);
    }
}

//TODO: put into 050–09F

//0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
//0x20, 0x60, 0x20, 0x20, 0x70, // 1
//0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
//0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
//0x90, 0x90, 0xF0, 0x10, 0x10, // 4
//0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
//0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
//0xF0, 0x10, 0x20, 0x40, 0x40, // 7
//0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
//0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
//0xF0, 0x90, 0xF0, 0x90, 0x90, // A
//0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
//0xF0, 0x80, 0x80, 0x80, 0xF0, // C
//0xE0, 0x90, 0x90, 0x90, 0xE0, // D
//0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
//0xF0, 0x80, 0xF0, 0x80, 0x80  // F
