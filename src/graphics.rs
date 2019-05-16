extern crate sdl2;

use sdl2::Sdl;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::chip8;

const SCALE_FACTOR: u16 = 15;
const COLOR: (u8, u8, u8) = (50, 90, 255);

pub struct GraphicsDriver {
	canvas: Canvas<Window>
}

impl GraphicsDriver {
	pub fn new(context: &Sdl) -> Result<Self, String> {
		let video_subsystem = context.video()?;
		let window = video_subsystem.window("CHIP-8", (chip8::PIXEL_W * SCALE_FACTOR) as u32, (chip8::PIXEL_H * SCALE_FACTOR) as u32)
			.position_centered()
			.opengl()
			.build()
			.map_err(|e| e.to_string())?;
		let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
		canvas.set_draw_color(Color::RGB(0, 0, 0));
		canvas.clear();
		canvas.present();
		Ok(GraphicsDriver{canvas})
	}

	pub fn draw(&mut self, graphics_buffer: [u8; (chip8::PIXEL_W * chip8::PIXEL_H) as usize]) -> Result<(), String> {
		// clear screen to black
		self.canvas.set_draw_color(Color::RGB(0, 0, 0));
		self.canvas.clear();

		// switch to contrast color
		let (r,g,b) = COLOR;
		self.canvas.set_draw_color(Color::RGB(r,g,b));

		for (i, p) in graphics_buffer.iter().enumerate() {
			if *p != 0 {
				// row = floor division
				let row: u16 = i as u16 / chip8::PIXEL_W;
				// column = remainder
				let col: u16 = i as u16 % chip8::PIXEL_W;
				self.canvas.fill_rect(Rect::new((col*SCALE_FACTOR) as i32, (row*SCALE_FACTOR) as i32, 
					SCALE_FACTOR as u32, SCALE_FACTOR as u32))?;
			}
		}
		
		self.canvas.present();
		Ok(()) 
	}
}
