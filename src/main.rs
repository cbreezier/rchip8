extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::env;
use std::fs;
use std::time::Duration;

use crate::display::Display;
use crate::state::State;

mod op_code;
mod display;
mod state;

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
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60)); // 60fps
    }
}
