
use rand::{Rng, thread_rng};

pub const PIXEL_W: u16 = 64;				// width of CHIP-8 screen
pub const PIXEL_H: u16 = 32;				// height of CHIP-8 screen
pub const FONT_LOCATION: u16 = 0x80;		// location of font set in system RAM
pub const CARTRIDGE_LOCATION: u16 = 0x200;	// location in system RAM where game data should be loaded on boot

pub struct Chip8 {
	pub memory: [u8; 4096],							// RAM
	pub reg: [u8; 16],								// registers
	pub gfx: [u8; (PIXEL_W * PIXEL_H) as usize],	// pixels
	stack: [u16; 16],								// subroutine stack
	pub key: [u8; 16],								// keypad

	idx: u16,										// index register
	pc: u16,										// program counter
	sp: u16,										// stack pointer
	pub delay_timer: u8,
	pub sound_timer: u8,
	pub draw_flag: bool,							// set when clear screen or draw opcodes are called
}

impl Chip8 {
	pub fn new() -> Chip8 {
		let mut chip = Chip8 {
			memory: [0;4096],
			reg: [0;16],
			gfx: [0; (PIXEL_W * PIXEL_H) as usize],
			stack: [0; 16],
			key: [0; 16],
			idx: 0,
			pc: CARTRIDGE_LOCATION,
			sp: 0,
			delay_timer: 0,
			sound_timer: 0,
			draw_flag: false,
		};
		// load font set
		for (i, v) in FONT_SET.iter().enumerate() {
			chip.memory[FONT_LOCATION as usize + i] = *v;
		}
		chip
	}

	pub fn cycle(&mut self) {
		// all opcodes are two bytes.
		// get the byte at memory[program counter] and memory[program counter + 1],
		// split them into nibbles for convenience.
		let w = self.memory[self.pc as usize] >> 4;
		let x = self.memory[self.pc as usize] & 0xF;
		let y = self.memory[(self.pc+1) as usize] >> 4;
		let z = self.memory[(self.pc+1) as usize] & 0xF;
		let yz = y << 4 | z;
		let xyz: u16 = (x as u16) << 8 | (y as u16) << 4 | (z as u16);
		let (_x, _y, _z) = (x as usize, y as usize, z as usize);
		let opcode = (w, x, y, z);

		if super::DEBUG {
			println!("=================\nregisters: {:02x?}", self.reg);
			println!("pc: 0x{:02x}, idx: 0x{:02x}, bytes at idx: {:02x?}", 
				self.pc, self.idx, 
				&self.memory[self.idx as usize..(self.idx+8) as usize]);
			println!("executing opcode {:02x?}", opcode);
		}

		match opcode {

			// skipping instruction 0XYZ

			// clear screen.
			(0x0, 0x0, 0xE, 0x0) => {
				self.draw_flag = true;
				self.gfx.iter_mut().for_each(|b| *b = 0);
				self.pc += 2;
			},

			// return from subroutine.
			(0x0, 0x0, 0xE, 0xE) => {
				self.sp -= 1;
				self.pc = self.stack[self.sp as usize];
			},

			// go to xyz.
			(0x1, _, _, _) => self.pc = xyz,

			// call subroutine at xyz.
			(0x2, _, _, _) => {
				self.stack[self.sp as usize] = self.pc + 2; // put next instruction on stack
				self.sp += 1; // increase stack pointer
				self.pc = xyz; // jump to subroutine
			},

			// skip next instruction if register x equals yz.
			(0x3, _, _, _) => {
				if self.reg[_x] == yz {
					self.pc += 2;
				}
				self.pc += 2;
			},

			// skip next instruction if register x doesn't equal yz.
			(0x4, _, _, _) => {
				if self.reg[_x] != yz {
					self.pc += 2;
				}
				self.pc += 2;
			},

			// skip next instruction if reg x == reg y.
			(0x5, _, _, 0x0) => {
				if self.reg[_x] == self.reg[_y] {
					self.pc += 2;
				}
				self.pc += 2;
			},

			// set reg x to yz.
			(0x6, _, _, _) => {
				self.reg[_x] = yz;
				self.pc += 2;
			},

			// add yz to reg x.
			(0x7, _, _, _) => {
				self.reg[_x] = self.reg[_x].wrapping_add(yz);
				self.pc += 2;
			},

			// set reg x to value of reg y.
			(0x8, _, _, 0x0) => {
				self.reg[_x] = self.reg[_y];
				self.pc += 2;
			},

			// set reg x to reg x | reg y.
			(0x8, _, _, 0x1) => {
				self.reg[_x] |= self.reg[_y];
				self.pc += 2;
			},

			// set reg x to reg x & reg y.
			(0x8, _, _, 0x2) => {
				self.reg[_x] &= self.reg[_y];
				self.pc += 2;
			},

			// UNDOCUMENTED. set reg x to reg x ^ reg y.
			(0x8, _, _, 0x3) => {
				self.reg[_x] ^= self.reg[_y];
				self.pc += 2;
			},

			// add reg y to reg x. reg f is set to 1 when there's a carry, and to 0 when there isn't. 
			(0x8, _, _, 0x4) => {
				let old_x = self.reg[_x];
				self.reg[_x] = self.reg[_x].wrapping_add(self.reg[_y]);
				self.reg[0xF] = if self.reg[_x] < old_x { 1 } else { 0 };
				self.pc += 2;
			},

			// reg y is subtracted from reg x. reg f is set to 0 when there's a borrow, and 1 when there isn't. 
			(0x8, _, _, 0x5) => {
				self.reg[0xF] = if self.reg[_x] < self.reg[_y] { 0 } else { 1 };
				self.reg[_x] = self.reg[_x].wrapping_sub(self.reg[_y]);
				self.pc += 2;
			},

			// WEIRD UNDOCUMENTED LEGACY ONE. TODO: add legacy mode?
			(0x8, _, _, 0x6) => {
				// first attempt. newer version?
				self.reg[0xF] = self.reg[_x] & 0x1;
				self.reg[_x] >>= 1;

				// legacy? according to https://github.com/mattmikolay/chip-8/wiki/CHIP%E2%80%908-Instruction-Set
				// self.reg[0xF] = self.reg[_y] & 0x1;
				// self.reg[_x] = self.reg[_y] >> 1;

				self.pc += 2;
			},

			// UNDOCUMENTED. sets reg x to reg y minus reg x. reg f is set to 0 when there's a borrow, and 1 when there isn't.
			(0x8, _, _, 0x7) => {
				self.reg[0xF] = if self.reg[_y] < self.reg[_x] { 0 } else { 1 };
				self.reg[_x] = self.reg[_y].wrapping_sub(self.reg[_x]);
				self.pc += 2;
			},

			// UNDOCUMENTED. store the most significant bit of reg x in reg f and left-shift reg x by 1.
			(0x8, _, _, 0xE) => {
				// according to https://en.wikipedia.org/wiki/CHIP-8#Opcode_table
				self.reg[0xF] = (self.reg[_x] & (1 << 7)) >> 7;
				self.reg[_x] <<= 1;

				// according to https://github.com/mattmikolay/chip-8/wiki/CHIP%E2%80%908-Instruction-Set
				// self.reg[0xF] = (self.reg[_y] & (1 << 7)) >> 7;
				// self.reg[_x] = self.reg[_y] << 1;
				// self.pc += 2;
			},

			// skips the next instruction if reg x doesn't equal reg y.
			(0x9, _, _, 0x0) => {
				if self.reg[_x] != self.reg[_y] {
					self.pc += 2;
				}
				self.pc += 2;
			},

			// Sets idx to the address xyz. 
			(0xA, _, _, _) => {
				self.idx = xyz;
				self.pc += 2;
			},

			// jump to xyz plus reg 0. 
			(0xB, _, _, _) => {
				self.pc = xyz + (self.reg[0x0] as u16);
			},

			// set reg x to the result of a bitwise and operation on a random number (Typically: 0 to 255) and yz.
			(0xC, _, _, _) => {
				let rand_val: u8 = thread_rng().gen();
				self.reg[_x] = yz & rand_val;
				self.pc += 2;
			},

			// draw sprites at coordinate reg x, reg y (NOT X AND Y AS I ORIGINALLY DID) a width of 8 and height of z.
			// get z sprites from memory starting at location idx.
			(0xD, _, _, _) => {
				self.draw_flag = true;
				let mut pixel_unset = false;
				let sprites = &self.memory[self.idx as usize .. (self.idx + (z as u16)) as usize];
				for i in 0.._z {	// for each row of 8 pixels (sprite)
					// x is columns, y is rows. gfx is a flat array. starting coordinate is ((y + row number) * PIXEL_W) + x.
					// every 8 bytes, we have to skip to next row, which means adding another PIXEL_W.

					if super::DEBUG {
						println!("drawing byte: 0b{:08b}", sprites[i]);
					}

					for j in 0..8 {
						let current_coordinate = self.reg[_x] as usize + ((self.reg[_y] as usize + i) * (PIXEL_W as usize)) + j;
						let current_sprite_bit = (sprites[i] & (1 << (7-j))) >> (7-j);

						if super::DEBUG {
							println!("drawing pixel 0b{:b} at {}, {}", 
								current_sprite_bit, 
								current_coordinate % PIXEL_W as usize,
								current_coordinate / PIXEL_W as usize
							);
						}

						if self.gfx[current_coordinate % self.gfx.len()] & current_sprite_bit != 0 { // if the current byte/pixel is 1, and the sprite bit is 1,
							pixel_unset = true; // then the xor operation will flip an on bit to off, meaning we need to record and set reg f.
						}
						self.gfx[current_coordinate % self.gfx.len()] ^= current_sprite_bit; // xor with sprite bit to draw
					}
				}
				self.reg[0xF] = if pixel_unset { 1 } else { 0 };
				self.pc += 2;

				if super::DEBUG {
					println!("screen:");
					for i in 0..PIXEL_H {
						for j in 0..PIXEL_W {
							print!("{} ", self.gfx[((PIXEL_W * i) + j) as usize])
						}
						println!();
					}
				}
			},

			// skip next instruction if key corresponding to reg x is pressed.
			(0xE, _, 0x9, 0xE) => {
				if self.key[self.reg[_x] as usize] != 0 {
					self.pc += 2;
				}
				self.pc += 2;
			},

			// skip next instruction if key corresponding to reg x isn't pressed.
			(0xE, _, 0xA, 0x1) => {
				if self.key[self.reg[_x] as usize] == 0 {
					self.pc += 2;
				}
				self.pc += 2;
			},

			// set reg x to value of delay timer.
			(0xF, _, 0x0, 0x7) => {
				self.reg[_x] = self.delay_timer;
				self.pc += 2;
			},

			// wait for key press and store in reg x.
			(0xF, _, 0x0, 0xA) => {
				// we don't check for input in the middle of a cycle, so we should just pass, not incrementing program counter,
				// and let the program come back to here until a key is registered.
				if self.key != [0; 16] {
					'key_checking: for (i, k) in self.key.iter().enumerate() { // including lifetime so we can break after only one key is stored to reg x
						if *k != 0 {
							self.reg[_x] = i as u8;
							self.pc += 2;
							break 'key_checking;
						}
					}
				}
			},

			// set delay timer to value of reg x.
			(0xF, _, 0x1, 0x5) => {
				self.delay_timer = self.reg[_x];
				self.pc += 2;
			},

			// set sound timer to value of reg x.
			(0xF, _, 0x1, 0x8) => {
				self.sound_timer = self.reg[_x];
				self.pc += 2;
			},

			// add value of reg x to idx.
			(0xF, _, 0x1, 0xE) => {
				self.idx += self.reg[_x] as u16;
				self.pc += 2;
			},

			// set idx to location of font char x.
			(0xF, _, 0x2, 0x9) => {
				self.idx = FONT_LOCATION + (x as u16 * 5);
				self.pc += 2;
			},

			// store the binary-coded decimal representation of reg x in memory[idx..idx+2].
			(0xF, _, 0x3, 0x3) => {
				self.memory[self.idx as usize] = self.reg[_x] % 100;
				self.memory[self.idx as usize +1] = (self.reg[_x] % 100) / 10;
				self.memory[self.idx as usize +2] = self.reg[_x] % 10;
				self.pc += 2;
			},

			// store reg 0 .. reg x (inclusive) in memory[idx..]. don't modify idx.
			(0xF, _, 0x5, 0x5) => {
				for i in 0 ..= _x {
					self.memory[self.idx as usize + i] = self.reg[i];
				}
				self.pc += 2;
			},

			// load reg 0 .. reg x (inclusive) from memory[idx..]. don't modify idx.
			(0xF, _, 0x6, 0x5) => {
				for i in 0 ..= _x {
					self.reg[i] = self.memory[self.idx as usize + i];
				}
				self.pc += 2;
			},

			oopsie => println!("illegal instruction: {:02x?}", oopsie),

		};
	}
}

const FONT_SET: [u8; 80] = [ 
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
