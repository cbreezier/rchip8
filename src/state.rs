use rand::Rng;

use crate::op_code::OpCode;

macro_rules! debug {
    ($( $args:expr ),*) => {
        // debug!( $( $args ),* );
    }
}

#[derive(Debug)]
pub struct State {
    display: [[bool; 32]; 64],
    ram: [u8; 4096],
    pc: u16,
    i: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    v: [u8; 16],
    keypad: [bool; 16],
}

impl State {
    pub fn new() -> Self {
        let mut result = Self {
            display: [[false; 32]; 64],
            ram: [0; 4096],
            pc: 0x200,
            i: 0,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            v: [0; 16],
            keypad: [false; 16],
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

    pub fn load_rom(self: &mut State, rom: Vec<u8>) {
        for (i, byte) in rom.iter().enumerate() {
            self.ram[0x200 + i] = *byte;
        }
    }

    pub fn next_op(self: &mut State) -> OpCode {
        let byte1 = self.ram[usize::from(self.pc)];
        let byte2 = self.ram[usize::from(self.pc + 1)];

        let next_op = OpCode::from_bytes(byte1, byte2);

        self.pc += 2;

        return next_op;
    }

    pub fn get_frame_buffer(&self) -> &[[bool; 32]; 64] {
        return &self.display;
    }

    pub fn get_vx(self: &State, op_code: &OpCode) -> u8 {
        return self.v[usize::from(op_code.x)];
    }

    pub fn set_vx(self: &mut State, op_code: &OpCode, value: u8) {
        self.v[usize::from(op_code.x)] = value;
    }

    pub fn get_vy(self: &State, op_code: &OpCode) -> u8 {
        return self.v[usize::from(op_code.y)];
    }

    pub fn set_carry(self: &mut State, value: bool) {
        self.v[0xF] = if value { 1 } else { 0 };
    }

    pub fn decrement_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn key_down(&mut self, key: usize) {
        self.keypad[key] = true;
    }

    pub fn key_up(&mut self, key: usize) {
        self.keypad[key] = false;
    }

    pub fn execute_op(self: &mut State, op_code: OpCode) {
        let vx = self.get_vx(&op_code);
        let vy = self.get_vy(&op_code);
        match op_code.op {
            0u8 => {
                match op_code.n {
                    0u8 => { // Clear screen
                        debug!("00E0: Clear screen");
                        self.display = [[false; 32]; 64];
                    },
                    0xEu8 => { // Return
                        self.pc = self.stack.pop().expect("Nothing on the stack to pop!");
                        debug!("00EE: Return to {}", self.pc);
                    },
                    _ => panic!("Unimplemented op {:?}", op_code),
                }
            },
            0x1u8 => { // Jump
                debug!("00EE: Jump {}", op_code.nnn);
                self.pc = op_code.nnn;
            },
            0x2u8 => { // Call
                debug!("2NNN: Call {}", op_code.nnn);
                self.stack.push(self.pc);
                self.pc = op_code.nnn;
            },
            0x3u8 => { // Skip if vx == nn
                debug!("3XNN: Skip if V{}({}) == NN({})", op_code.x, vx, op_code.nn);
                if vx == op_code.nn {
                    self.pc += 2;
                }
            },
            0x4u8 => { // Skip if vx != nn
                debug!("4XNN: Skip if V{}({}) != NN({})", op_code.x, vx, op_code.nn);
                if vx != op_code.nn {
                    self.pc += 2;
                }
            },
            0x5u8 => { // Skip if vx == vy
                debug!("5XY0: Skip if V{}({}) == V{}({})", op_code.x, vx, op_code.y, vy);
                if vx == vy {
                    self.pc += 2;
                }
            },
            0x9u8 => { // Skip if vx != vy
                debug!("9XY0: Skip if V{}({}) != V{}({})", op_code.x, vx, op_code.y, vy);
                if vx != vy {
                    self.pc += 2;
                }
            },
            0x6u8 => { // Set nn
                debug!("6XNN: V{} = {}", op_code.x, op_code.nn);
                self.set_vx(&op_code, op_code.nn);
            },
            0x7u8 => { // Add nn
                debug!("7XNN: V{}({}) += {}", op_code.x, vx, op_code.nn);
                self.set_vx(&op_code, vx.wrapping_add(op_code.nn));
            },
            0x8u8 => {
                match op_code.n {
                    0x0u8 => { // Set vy
                        debug!("8XY0: V{}({}) = V{}({})", op_code.x, vx, op_code.y, vy);
                        self.set_vx(&op_code, vy);
                    },
                    0x1u8 => { // OR
                        debug!("8XY1: V{}({}) |= V{}({})", op_code.x, vx, op_code.y, vy);
                        self.set_vx(&op_code, vx | vy);
                    },
                    0x2u8 => { // AND
                        debug!("8XY2: V{}({}) &= V{}({})", op_code.x, vx, op_code.y, vy);
                        self.set_vx(&op_code, vx & vy);
                    },
                    0x3u8 => { // XOR
                        debug!("8XY3: V{}({}) ^= V{}({})", op_code.x, vx, op_code.y, vy);
                        self.set_vx(&op_code, vx ^ vy);
                    },
                    0x4u8 => { // Add vy
                        debug!("8XY4: V{}({}) += V{}({})", op_code.x, vx, op_code.y, vy);
                        self.set_vx(&op_code, vx.wrapping_add(vy));
                        self.set_carry(self.get_vx(&op_code) < vx); // Overflowed
                    },
                    0x5u8 => { // Subtract vy
                        debug!("8XY5: V{}({}) -= V{}({})", op_code.x, vx, op_code.y, vy);
                        self.set_vx(&op_code, vx.wrapping_sub(vy));
                        self.set_carry(vx > vy);
                    },
                    0x7u8 => { // Subtract from vy
                        debug!("8XY7: V{}({}) = V{}({}) - VX", op_code.x, vx, op_code.y, vy);
                        self.set_vx(&op_code, vy.wrapping_sub(vx));
                        self.set_carry(vy > vx);
                    },
                    0x6u8 => { // Shift right
                        // debug!("8XY6: V{}({}) = V{}({} >> 1)", op_code.x, vx, op_code.y, vy);
                        // self.set_vx(&op_code, vy >> 1);
                        // self.set_carry(vy & 0b00000001u8 != 0);

                        // We use the "modern" implementation of these instructions
                        debug!("8XY6: V{}({}) >>= 1", op_code.x, vx);
                        self.set_vx(&op_code, vx >> 1);
                        self.set_carry(vx & 0b00000001u8 != 0);
                    },
                    0xEu8 => { // Shift left
                        // debug!("8XYE: V{}({}) = V{}({} << 1)", op_code.x, vx, op_code.y, vy);
                        // self.set_vx(&op_code, vy << 1);
                        // self.set_carry(vy & 0b10000000u8 != 0);

                        // We use the "modern" implementation of these instructions
                        debug!("8XYE: V{}({}) <<= 1", op_code.x, vx);
                        self.set_vx(&op_code, vx << 1);
                        self.set_carry(vx & 0b10000000u8 != 0);
                    },
                    _ => panic!("Unimplemented op {:?}", op_code),
                }
            },
            0xAu8 => { // Set index
                debug!("ANNN: I = {}", op_code.nnn);
                self.i = op_code.nnn;
            },
            0xBu8 => { // Jump with offset
                debug!("BNNN: PC = NNN({}) + V0({})", op_code.nnn, self.v[0]);
                self.pc = op_code.nnn + u16::from(self.v[0]);
            },
            0xCu8 => { // Random
                debug!("CXNN: Random & NN({})", op_code.nn);
                let random_value: u8 = rand::thread_rng().gen();
                self.set_vx(&op_code, random_value & op_code.nn);
            },
            0xDu8 => { // Draw
                let vx = self.get_vx(&op_code);
                let vy = self.get_vy(&op_code);
                debug!("DXYN: {} height sprite at {}, {}", op_code.n, vx, vy);

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
                    i += 1;
                }
            },
            0xEu8 => { // Skip if key
                match op_code.nn {
                    0x9Eu8 => {
                        debug!("EX9E: Skip if key V{}({})", op_code.x, vx);
                        if vx < 16 && self.keypad[usize::from(vx)] {
                            self.pc += 2;
                        }
                    },
                    0xA1u8 => {
                        debug!("EX9E: Skip if not key V{}({})", op_code.x, vx);
                        if vx < 16 && !self.keypad[usize::from(vx)] {
                            self.pc += 2;
                        }
                    },
                    _ => panic!("Unimplemented op {:?}", op_code),
                }
            },
            0xFu8 => {
                match op_code.nn {
                    0x07u8 => {
                        debug!("FX07: V{} = delay timer({})", op_code.x, self.delay_timer);
                        self.set_vx(&op_code, self.delay_timer);
                    },
                    0x15u8 => {
                        debug!("FX15: delay timer = V{}({})", op_code.x, vx);
                        self.delay_timer = vx;
                    },
                    0x18u8 => {
                        debug!("FX18: sound timer = V{}({})", op_code.x, vx);
                        self.sound_timer = vx;
                    },
                    0x1Eu8 => {
                        debug!("FX1E: I += V{}({})", op_code.x, vx);
                        self.i += u16::from(vx);
                    },
                    0x0Au8 => {
                        debug!("FX0A: Get key");
                        // We stray a bit from original implementation
                        // Rather than waiting for a keyup or even waiting for a keydown
                        // we accept even currently-held keys :fingerscrossed:
                        // This does mean that we have implicit priority when multiple
                        // keys are held
                        let mut is_pressed = false;
                        for (i, key) in self.keypad.iter().enumerate() {
                            if *key {
                                self.set_vx(&op_code, i as u8);
                                is_pressed = true;
                                break;
                            }
                        }

                        if !is_pressed {
                            self.pc -= 2; // Effectively "pause" execution
                        }
                    },
                    0x29u8 => {
                        debug!("FX29: Font at V{}({})", op_code.x, vx);
                        let character = vx & 0xF;
                        self.i = 0x50u16 + (5u16 * u16::from(character));
                    },
                    0x33u8 => {
                        debug!("FX33: Decimal font of V{}({})", op_code.x, vx);
                        let digit3 = vx % 10;
                        let digit2 = (vx % 100) / 10;
                        let digit1 = vx / 100;

                        self.ram[usize::from(self.i)] = digit1;
                        self.ram[usize::from(self.i + 1)] = digit2;
                        self.ram[usize::from(self.i + 2)] = digit3;
                    },
                    0x55u8 => {
                        debug!("FX55: Store V0..V{} to I", op_code.x);
                        for i in 0..usize::from(op_code.x + 1) {
                            self.ram[usize::from(self.i) + i] = self.v[i];
                        }
                        for _i in 0..usize::from(op_code.x + 1) {
                            debug!("    Ram at {} is now {}", usize::from(self.i) + _i, self.ram[usize::from(self.i) + _i]);
                        }
                    },
                    0x65u8 => {
                        debug!("FX65: Load V0..V{} from I", op_code.x);
                        for i in 0..usize::from(op_code.x + 1) {
                            self.v[i] = self.ram[usize::from(self.i) + i];
                        }
                        for _i in 0..usize::from(op_code.x + 1) {
                            debug!("    Loaded {} into V{}", self.v[_i], _i);
                        }
                    },
                    _ => panic!("Unimplemented op {:?}", op_code),
                }
            },
            _ => panic!("Unimplemented op {:?}", op_code),
        }
    }
}
