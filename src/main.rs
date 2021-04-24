use std::env;
use std::fs;

extern crate sdl2;

use sdl2::Sdl;
use sdl2::video::Window;
use sdl2::render::Canvas;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

#[derive(Debug)]
struct OpCode {
    op: u8, // Technically a u4 but u8 makes for easier comparisons
    x: u8, // Technically a u4 but u8 makes for easier comparisons
    y: u8, // Technically a u4 but u8 makes for easier comparisons
    n: u8, // Technically a u4 but u8 makes for easier comparisons
    nn: u8,
    nnn: u16, // Technically a u12 but u16 makes for easier comparisons
}

impl OpCode {
    fn from_bytes(byte1: u8, byte2: u8) -> Self {
        Self {
            op: (byte1 >> 4) & 0xF,
            x: byte1 & 0xF,
            y: (byte2 >> 4) & 0xF,
            n: byte2 & 0xF,
            nn: byte2,
            nnn: (u16::from(byte1 & 0xF) << 8) | u16::from(byte2),
        }
    }
}

struct Display {
    canvas: Canvas<Window>,
    scale: u32,
    background_color: Color,
    foreground_color: Color,
}

impl Display {
    fn new(sdl_context: &Sdl) -> Self {
        let video_subsystem = sdl_context.video().unwrap();

        let scale = 10;

        let window = video_subsystem.window("rust-sdl2 demo", 64 * scale, 32 * scale)
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();

        Self {
            canvas: canvas,
            scale: 10,
            background_color: Color::RGB(0, 0, 0),
            foreground_color: Color::RGB(255, 255, 255),
        }
    }
}

#[derive(Debug)]
struct State {
    display: [[bool; 32]; 64],
    ram: [u8; 4096],
    pc: u16,
    i: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    v: [u8; 16],
}

impl State {
    fn new() -> Self {
        let mut result = Self {
            display: [[false; 32]; 64],
            ram: [0; 4096],
            pc: 0x200,
            i: 0,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            v: [0; 16],
        };

        let fonts = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80  // F
        ];
        for (i, font) in fonts.iter().enumerate() {
            result.ram[0x50 + i] = *font;
        }

        return result;
    }

    fn load_rom(self: &mut State, rom: Vec<u8>) {
        for (i, byte) in rom.iter().enumerate() {
            self.ram[0x200 + i] = *byte;
        }
    }

    fn next_op(self: &mut State) -> OpCode {
        let byte1 = self.ram[usize::from(self.pc)];
        let byte2 = self.ram[usize::from(self.pc + 1)];

        let next_op = OpCode::from_bytes(byte1, byte2);

        self.pc += 2;

        return next_op;
    }

    fn draw_canvas(self: &State, display: &mut Display) {
        display.canvas.set_draw_color(display.background_color);
        display.canvas.clear();

        display.canvas.set_draw_color(display.foreground_color);
        for (x, col) in self.display.iter().enumerate() {
            for (y, pixel) in col.iter().enumerate() {
                if *pixel {
                    let x = ((x as u32) * display.scale) as i32;
                    let y = ((y as u32) * display.scale) as i32;
                    let width = display.scale;
                    let height = display.scale;
                    display.canvas.draw_rect(Rect::new(
                        x,
                        y,
                        width,
                        height,
                    )).expect("Failed to draw pixel");
                }
            }
        }
        
        display.canvas.present();
    }

    fn get_vx(self: &State, op_code: &OpCode) -> u8 {
        return self.v[usize::from(op_code.x)];
    }

    fn set_vx(self: &mut State, op_code: &OpCode, value: u8) {
        self.v[usize::from(op_code.x)] = value;
    }

    fn get_vy(self: &State, op_code: &OpCode) -> u8 {
        return self.v[usize::from(op_code.y)];
    }

    fn set_carry(self: &mut State, value: bool) {
        self.v[0xF] = if value { 1 } else { 0 };
    }

    fn execute_op(self: &mut State, display: &mut Display, op_code: OpCode) {
        let vx = self.get_vx(&op_code);
        let vy = self.get_vy(&op_code);
        match op_code.op {
            0u8 => {
                match op_code.n {
                    0u8 => { // Clear screen
                        println!("00E0: Clear screen");
                        self.display = [[false; 32]; 64];
                        self.draw_canvas(display);
                    },
                    0xEu8 => { // Return
                        self.pc = self.stack.pop().expect("Nothing on the stack to pop!");
                        println!("00EE: Return to {}", self.pc);
                    },
                    _ => panic!("Unimplemented op {:?}", op_code),
                }
            },
            0x1u8 => { // Jump
                println!("00EE: Jump {}", op_code.nnn);
                self.pc = op_code.nnn;
            },
            0x2u8 => { // Call
                println!("2NNN: Call {}", op_code.nnn);
                self.stack.push(self.pc);
                self.pc = op_code.nnn;
            },
            0x3u8 => { // Skip if vx == nn
                println!("3XNN: Skip if V{}({}) == NN({})", op_code.x, vx, op_code.nn);
                if vx == op_code.nn {
                    self.pc += 2;
                }
            },
            0x4u8 => { // Skip if vx != nn
                println!("4XNN: Skip if V{}({}) != NN({})", op_code.x, vx, op_code.nn);
                if vx != op_code.nn {
                    self.pc += 2;
                }
            },
            0x5u8 => { // Skip if vx == vy
                println!("5XY0: Skip if V{}({}) == V{}({})", op_code.x, vx, op_code.y, vy);
                if vx == vy {
                    self.pc += 2;
                }
            },
            0x9u8 => { // Skip if vx != vy
                println!("9XY0: Skip if V{}({}) != V{}({})", op_code.x, vx, op_code.y, vy);
                if vx != vy {
                    self.pc += 2;
                }
            },
            0x6u8 => { // Set nn
                println!("6XNN: V{} = {}", op_code.x, op_code.nn);
                self.set_vx(&op_code, op_code.nn);
            },
            0x7u8 => { // Add nn
                println!("7XNN: V{}({}) += {}", op_code.x, vx, op_code.nn);
                self.set_vx(&op_code, vx.wrapping_add(op_code.nn));
            },
            0x8u8 => {
                match op_code.n {
                    0x0u8 => { // Set vy
                        println!("8XY0: V{}({}) = V{}({})", op_code.x, vx, op_code.y, vy);
                        self.set_vx(&op_code, vy);
                    },
                    0x1u8 => { // OR
                        println!("8XY1: V{}({}) |= V{}({})", op_code.x, vx, op_code.y, vy);
                        self.set_vx(&op_code, vx | vy);
                    },
                    0x2u8 => { // AND
                        println!("8XY2: V{}({}) &= V{}({})", op_code.x, vx, op_code.y, vy);
                        self.set_vx(&op_code, vx & vy);
                    },
                    0x3u8 => { // XOR
                        println!("8XY3: V{}({}) ^= V{}({})", op_code.x, vx, op_code.y, vy);
                        self.set_vx(&op_code, vx ^ vy);
                    },
                    0x4u8 => { // Add vy
                        println!("8XY4: V{}({}) += V{}({})", op_code.x, vx, op_code.y, vy);
                        self.set_vx(&op_code, vx.wrapping_add(vy));
                        self.set_carry(self.get_vx(&op_code) < vx); // Overflowed
                    },
                    0x5u8 => { // Subtract vy
                        println!("8XY5: V{}({}) -= V{}({})", op_code.x, vx, op_code.y, vy);
                        self.set_vx(&op_code, vx.wrapping_sub(vy));
                        self.set_carry(vx > vy);
                    },
                    0x7u8 => { // Subtract from vy
                        println!("8XY7: V{}({}) = V{}({}) - VX", op_code.x, vx, op_code.y, vy);
                        self.set_vx(&op_code, vy.wrapping_sub(vx));
                        self.set_carry(vy > vx);
                    },
                    0x6u8 => { // Shift right
                        // println!("8XY6: V{}({}) = V{}({} >> 1)", op_code.x, vx, op_code.y, vy);
                        // self.set_vx(&op_code, vy >> 1);
                        // self.set_carry(vy & 0b00000001u8 != 0);
                        println!("8XY6: V{}({}) >>= 1", op_code.x, vx);
                        self.set_vx(&op_code, vx >> 1);
                        self.set_carry(vx & 0b00000001u8 != 0);
                    },
                    0xEu8 => { // Shift left
                        // println!("8XYE: V{}({}) = V{}({} << 1)", op_code.x, vx, op_code.y, vy);
                        // self.set_vx(&op_code, vy << 1);
                        // self.set_carry(vy & 0b10000000u8 != 0);
                        println!("8XYE: V{}({}) <<= 1", op_code.x, vx);
                        self.set_vx(&op_code, vx << 1);
                        self.set_carry(vx & 0b10000000u8 != 0);
                    },
                    _ => panic!("Unimplemented op {:?}", op_code),
                }
            },
            0xAu8 => { // Set index
                println!("ANNN: I = {}", op_code.nnn);
                self.i = op_code.nnn;
            },
            0xBu8 => { // Jump with offset
                println!("BNNN: PC = NNN({}) + V0({})", op_code.nnn, self.v[0]);
                self.pc = op_code.nnn + u16::from(self.v[0]);
            },
            0xCu8 => { // Random
                println!("CXNN: Random & NN({})", op_code.nn);
                self.set_vx(&op_code, 4u8 & op_code.nn); // TODO use better random number than 4
            },
            0xDu8 => { // Draw
                let vx = self.get_vx(&op_code);
                let vy = self.get_vy(&op_code);
                println!("DXYN: {} height sprite at {}, {}", op_code.n, vx, vy);

                // Drawing a sprite should wrap
                let start_x = usize::from(vx) % 64;
                let start_y = usize::from(vy) % 32;

                self.set_carry(false);

                // Draw a sprite n pixels high
                let mut i = 0;
                for row in 0..usize::from(op_code.n) {
                    let y = start_y + row;
                    if y >= 32 {
                        break;
                    }

                    let sprite = self.ram[usize::from(self.i + i)];
                    for col in 0..8 {
                        let x = start_x + col;
                        if x >= 64 {
                            break;
                        }

                        let old_pixel = self.display[x][y];
                        let new_pixel = (sprite & (1 << (7 - col))) != 0;
                        self.display[x][y] = old_pixel ^ new_pixel;
                        self.set_carry(old_pixel && new_pixel);
                    }

                    // Move onto the next sprite
                    // TODO should we modify self.i?
                    i += 1;
                }

                self.draw_canvas(display);
            },
            0xEu8 => { // Skip if key
                match op_code.nn {
                    0x9Eu8 => {
                        println!("EX9E: Skip if key V{}({})", op_code.x, vx);
                        // TODO
                    },
                    0xA1u8 => {
                        println!("EX9E: Skip if not key V{}({})", op_code.x, vx);
                        // TODO
                    },
                    _ => panic!("Unimplemented op {:?}", op_code),
                }
            },
            0xFu8 => {
                match op_code.nn {
                    0x07u8 => {
                        println!("FX07: V{} = delay timer({})", op_code.x, self.delay_timer);
                        self.set_vx(&op_code, self.delay_timer);
                    },
                    0x15u8 => {
                        println!("FX15: delay timer = V{}({})", op_code.x, vx);
                        self.delay_timer = vx;
                    },
                    0x18u8 => {
                        println!("FX18: sound timer = V{}({})", op_code.x, vx);
                        self.sound_timer = vx;
                    },
                    0x1Eu8 => {
                        println!("FX1E: I += V{}({})", op_code.x, vx);
                        self.i += u16::from(vx);
                    },
                    0x0Au8 => {
                        println!("FX0A: Get key");
                        // TODO
                    },
                    0x29u8 => {
                        println!("FX29: Font at V{}({})", op_code.x, vx);
                        let character = vx & 0xF;
                        self.i = 0x50u16 + (5u16 * u16::from(character));
                    },
                    0x33u8 => {
                        println!("FX33: Decimal font of V{}({})", op_code.x, vx);
                        let digit3 = vx % 10;
                        let digit2 = (vx % 100) / 10;
                        let digit1 = vx / 100;
                        
                        self.ram[usize::from(self.i)] = digit1;
                        self.ram[usize::from(self.i + 1)] = digit2;
                        self.ram[usize::from(self.i + 2)] = digit3;
                    },
                    0x55u8 => {
                        println!("FX55: Store V0..V{} to I", op_code.x);
                        for i in 0..usize::from(op_code.x + 1) {
                            self.ram[usize::from(self.i) + i] = self.v[i];
                        }
                        for i in 0..usize::from(op_code.x + 1) {
                            println!("    Ram at {} is now {}", usize::from(self.i) + i, self.ram[usize::from(self.i) + i]);
                        }
                    },
                    0x65u8 => {
                        println!("FX65: Load V0..V{} from I", op_code.x);
                        for i in 0..usize::from(op_code.x + 1) {
                            self.v[i] = self.ram[usize::from(self.i) + i];
                        }
                        for i in 0..usize::from(op_code.x + 1) {
                            println!("    Loaded {} into V{}", self.v[i], i);
                        }
                    },
                    _ => panic!("Unimplemented op {:?}", op_code),
                }
            },
            _ => panic!("Unimplemented op {:?}", op_code),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom_file = &args[1];

    let rom_file = fs::read(rom_file)
        .expect("Failed to read rom data");

    let sdl_context = sdl2::init().unwrap();

    let mut display = Display::new(&sdl_context);
    let mut state = State::new();
    state.load_rom(rom_file);

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

        // The rest of the game loop goes here...
        let op_code = state.next_op();
        state.execute_op(&mut display, op_code);
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 10)); // 10fps

        // let mut input = String::new();
        // io::stdin()
        //     .read_line(&mut input)
        //     .unwrap();
    }
}
