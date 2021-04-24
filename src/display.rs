extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::Sdl;
use sdl2::video::Window;

pub struct Display {
    canvas: Canvas<Window>,
    scale: u32,
    background_color: Color,
    foreground_color: Color,
}

impl Display {
    pub fn new(sdl_context: &Sdl) -> Self {
        let video_subsystem = sdl_context.video().unwrap();

        let scale = 10;

        let window = video_subsystem.window("rust-sdl2 demo", 64 * scale, 32 * scale)
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();

        Self {
            canvas,
            scale: 10,
            background_color: Color::RGB(0, 0, 0),
            foreground_color: Color::RGB(255, 255, 255),
        }
    }

    pub fn draw_canvas(self: &mut Display, frame_buffer: &[[bool; 32]; 64]) {
        self.canvas.set_draw_color(self.background_color);
        self.canvas.clear();

        self.canvas.set_draw_color(self.foreground_color);
        for (x, col) in frame_buffer.iter().enumerate() {
            for (y, pixel) in col.iter().enumerate() {
                if *pixel {
                    let x = ((x as u32) * self.scale) as i32;
                    let y = ((y as u32) * self.scale) as i32;
                    let width = self.scale;
                    let height = self.scale;
                    self.canvas.draw_rect(Rect::new(
                        x,
                        y,
                        width,
                        height,
                    )).expect("Failed to draw pixel");
                }
            }
        }

        self.canvas.present();
    }
}
