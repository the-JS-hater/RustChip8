use raylib::ffi::TraceLogLevel::LOG_NONE;
use raylib::prelude::*;

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

    fn push_stack(&mut self, addr: u16) {
        self.stack.push(addr);
    }

    fn pop_stack(&mut self) -> u16 {
        return self
            .stack
            .pop()
            .expect("Program tried to pop an empty stack");
    }
}

fn main() {
    let mut chip8 = Chip8::new();

    set_trace_log(LOG_NONE);

    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH * SQUARE_SIZE, SCREEN_HEIGHT * SQUARE_SIZE)
        .title("CHIP-8 Emulator")
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        // Render CHIP-8 display
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                if chip8.display[y as usize][x as usize] {
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
