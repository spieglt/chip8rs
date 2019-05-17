
extern crate rand;
extern crate sdl2;
use sdl2::keyboard::Keycode;
use sdl2::event::Event;

use std::fs::File;
use std::io::Read;
use std::time::{Duration, Instant};

mod chip8;
mod graphics;
mod input;
mod audio;

const SOUND_DELAY_HZ:	u32 = 60;
const CPU_HZ: 			u32 = 500;
const STAT_FREQUENCY:	u32 = 5;	// every x seconds

const DEBUG: bool = false;
const STEP: bool = false;

fn main() -> Result<(), String> {

	let stdin = std::io::stdin();
	let mut buf = String::new();

	println!("EMULATING");
	let mut chip = chip8::Chip8::new();

	// load cartridge
	let argv: Vec<String> = std::env::args().collect();
	assert!(argv.len() > 1, "\nMust use filename of rom as argument!\nEXAMPLE:\n$ cargo run roms/pong.ch8\nor\n> chip8.exe roms\\pong.ch8\n");
	let mut rom = File::open(&argv[1]).map_err(|e| e.to_string())?;
	let mut rom_data: Vec<u8> = Vec::new();
	rom.read_to_end(&mut rom_data).map_err(|e| e.to_string())?;
	assert!(rom_data.len() < 4096 - chip8::CARTRIDGE_LOCATION as usize, "ROM file too big for CHIP-8 memory!");
	for i in 0..rom_data.len() {
		chip.memory[chip8::CARTRIDGE_LOCATION as usize + i] = rom_data[i];
	}

	// set up SDL2
	let sdl_context = sdl2::init()?;
	let mut event_pump = sdl_context.event_pump()?;
	let mut graphics_driver = graphics::GraphicsDriver::new(&sdl_context)?;
	let audio_device = audio::initialize(&sdl_context)?;

	// configure timers
	let mut loop_time = Instant::now();		// point of reference for each loop to judge whether it's time to progress
	let mut sound_delay_time = loop_time;	// deadline used for sound and delay timers in CPU
	let mut cpu_time = loop_time;			// deadline used for CPU clock
	let mut playing = false;				// beep flag

	let mut fps_time = Instant::now();		// deadline used for calculating clock speeds
	let (mut sound_delay_cycles, mut cpu_cycles) = (0u32, 0u32); // counters used for stats

	// main loop
	'running: loop {
		// break loop if program is exited or ESC key is hit
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
					break 'running
				},
				_ => {}
			}
		}

		loop_time = Instant::now();
		// if now is after cpu_time, run a cycle and extend the deadline.
		if loop_time > cpu_time {
			if STEP {
				stdin.read_line(&mut buf);
			}
			chip.cycle();
			extend(&mut cpu_time, CPU_HZ);
			cpu_cycles += 1;
		}
		// if now is after sound_delay_time, decrement their counters and extend the deadline.
		if loop_time > sound_delay_time {
			if chip.sound_timer > 0 { chip.sound_timer -= 1; }
			if chip.delay_timer > 0 { chip.delay_timer -= 1; }
			extend(&mut sound_delay_time, SOUND_DELAY_HZ);
			sound_delay_cycles += 1;
		}

		// graphics
		if chip.draw_flag {
			graphics_driver.draw(chip.gfx)?;
			chip.draw_flag = false;
		}

		// input
		input::get_keys(&mut chip.key, &event_pump);
		if DEBUG {
			if chip.key != [0;16] {
				println!("chip's key state: {:?}", chip.key);
			}
		}

		// audio
		if chip.sound_timer > 0 && !playing {
			audio_device.resume();
			playing = true;
		}
		if chip.sound_timer == 0 && playing {
			audio_device.pause();
			playing = false;
		}

		// print stats, reset counters, extend deadline
		if loop_time > fps_time {
			println!("sound/delay: {}Hz, cpu: {}Hz", sound_delay_cycles/STAT_FREQUENCY, cpu_cycles/STAT_FREQUENCY);
			sound_delay_cycles = 0;
			cpu_cycles = 0;
			fps_time += Duration::new(STAT_FREQUENCY as u64, 0);
		}
	}
	Ok(())
}

fn extend(timestamp: &mut Instant, fraction_of_second: u32) {
	*timestamp += Duration::new(0, 1_000_000_000u32 / fraction_of_second);
}
