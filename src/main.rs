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
            [0x1, nibb1, nibb2, nibb3] => {
                let mut addr = (nibb1 as u16) << 8;
                addr |= (nibb2 as u16) << 4;
                addr |= nibb3 as u16;
                self.pc = addr;
            }
            [0x6, x, nibb1, nibb2] => {
                let idx = x as usize;
                let val = ((nibb1 as u8) << 4) | nibb2 as u8;
                self.registers[idx] = val;
            }
            [0x7, x, nibb1, nibb2] => {
                let idx = x as usize;
                let val = ((nibb1 as u8) << 4) | nibb2 as u8;
                self.registers[idx] += val;
            }
            [0xA, nibb1, nibb2, nibb3] => {
                let mut addr = (nibb1 as u16) << 8;
                addr |= (nibb2 as u16) << 4;
                addr |= nibb3 as u16;
                self.reg_i = addr;
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
