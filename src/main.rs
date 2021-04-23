use std::io;
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
        Self {
            display: [[false; 32]; 64],
            ram: [0; 4096],
            pc: 0x200,
            i: 0,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            v: [0; 16],
        }
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

    fn set_vy(self: &mut State, op_code: &OpCode, value: u8) {
        self.v[usize::from(op_code.y)] = value;
    }

    fn execute_op(self: &mut State, display: &mut Display, op_code: OpCode) {
        match op_code.op {
            0u8 => {
                match op_code.n {
                    0u8 => { // Clear screen
                        println!("00E0");
                        self.display = [[false; 32]; 64];
                        self.draw_canvas(display);
                    },
                    0xEu8 => { // Return
                        println!("00EE");
                        self.pc = self.stack.pop().expect("Nothing on the stack to pop!");
                    },
                    _ => panic!("Unimplemented op {:?}", op_code),
                }
            },
            0x1u8 => { // Jump
                println!("00EE");
                self.pc = op_code.nnn;
            },
            0x2u8 => { // Call
                println!("2NNN");
                self.stack.push(self.pc);
                self.pc = op_code.nnn;
            },
            0x3u8 => { // Skip if vx == nn
                println!("3XNN");
                if self.get_vx(&op_code) == op_code.nn {
                    self.pc += 2;
                }
            },
            0x4u8 => { // Skip if vx != nn
                println!("4XNN");
                if self.get_vx(&op_code) != op_code.nn {
                    self.pc += 2;
                }
            },
            0x5u8 => { // Skip if vx == vy
                println!("5XY0");
                if self.get_vx(&op_code) == self.get_vy(&op_code) {
                    self.pc += 2;
                }
            },
            0x9u8 => { // Skip if vx != vy
                println!("9XY0");
                if self.get_vx(&op_code) != self.get_vy(&op_code) {
                    self.pc += 2;
                }
            },
            0x6u8 => { // Set
                println!("6XNN: V{} = {}", op_code.x, op_code.nn);
                self.set_vx(&op_code, op_code.nn);
            },
            0x7u8 => { // Add
                println!("7XNN: V{} += {}", op_code.x, op_code.nn);
                self.set_vx(&op_code, self.get_vx(&op_code) + op_code.nn);
            },
            0xAu8 => { // Set index
                println!("ANNN: I = {}", op_code.nnn);
                self.i = op_code.nnn;
            },
            0xDu8 => { // Draw
                let vx = self.get_vx(&op_code);
                let vy = self.get_vy(&op_code);
                println!("DXYN: {} height sprite at {}, {}", op_code.n, vx, vy);

                // Drawing a sprite should wrap
                let start_x = usize::from(vx) % 64;
                let start_y = usize::from(vy) % 32;

                // Draw a sprite n pixels high
                let mut i = 0;
                for row in 0..usize::from(op_code.n) {
                    let y = start_y + row;
                    if y >= 32 {
                        break;
                    }

                    let sprite = self.ram[usize::from(self.i + i)];
                    println!("Got sprite {} from memory {}", sprite, self.i + i);
                    for col in 0..8 {
                        let x = start_x + col;
                        if x >= 64 {
                            break;
                        }

                        let new_pixel = (sprite & (1 << (7 - col))) != 0;
                        println!("Drawing new pixel {} at {}, {}", new_pixel, x, y);
                        self.display[x][y] ^= new_pixel;
                    }

                    // Move onto the next sprite
                    // TODO should we modify self.i?
                    i += 1;
                }

                self.draw_canvas(display);
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
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 2)); // 2fps

        // let mut input = String::new();
        // io::stdin()
        //     .read_line(&mut input)
        //     .unwrap();
    }
}
