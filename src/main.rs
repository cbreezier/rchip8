extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::env;
use std::fs;
use std::time::Duration;
use std::time::Instant;

use crate::display::Display;
use crate::state::State;
use crate::timing::{TimedSystem, Timing};

mod op_code;
mod display;
mod state;
mod timing;

const CPU_SYSTEM: &str = "cpu";
const TIMER_SYSTEM: &str = "timer";
const DISPLAY_SYSTEM: &str = "display";

macro_rules! debug {
    ($( $args:expr ),*) => {
        // println!( $( $args ),* );
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

    // Init our timing contoller
    let mut timing = Timing::new(
        Instant::now(),
        vec![
            TimedSystem::new(CPU_SYSTEM, 700),
            TimedSystem::new(TIMER_SYSTEM, 60),
            TimedSystem::new(DISPLAY_SYSTEM, 60),
        ],
    );

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
        let instructions = timing.get_instructions(Instant::now());
        for instruction in instructions {
            match instruction.name {
                CPU_SYSTEM => {
                    debug!("=== Running cpu for {} cycles", instruction.cycles);
                    for _ in 0..instruction.cycles {
                        let op_code = state.next_op();
                        state.execute_op(op_code);
                    }
                },
                TIMER_SYSTEM => {
                    debug!("=== Running timer for {} cycles", instruction.cycles);
                    for _ in 0..instruction.cycles {
                        state.decrement_timers();
                    }
                },
                DISPLAY_SYSTEM => {
                    debug!("=== Running display for {} cycles", instruction.cycles);
                    for _ in 0..instruction.cycles {
                        display.draw_canvas(state.get_frame_buffer());
                    }
                },
                unknown => panic!("Unexpected instruction {}", unknown),
            }
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60)); // 60fps
    }
}
