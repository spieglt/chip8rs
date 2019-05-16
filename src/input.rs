
use sdl2::keyboard::Scancode;
use std::collections::HashSet;

/*
Keypad                   Keyboard
+-+-+-+-+                +-+-+-+-+
|1|2|3|C|                |1|2|3|4|
+-+-+-+-+                +-+-+-+-+
|4|5|6|D|                |Q|W|E|R|
+-+-+-+-+       =>       +-+-+-+-+
|7|8|9|E|                |A|S|D|F|
+-+-+-+-+                +-+-+-+-+
|A|0|B|F|                |Z|X|C|V|
+-+-+-+-+                +-+-+-+-+
*/

pub fn get_keys(chip_keys: &mut [u8; 16], event_pump: &sdl2::EventPump) {
    // clear processor key state
    chip_keys.iter_mut().for_each(|x| *x = 0);
    // get currently active keys
    let pressed_keys: HashSet<Scancode> = event_pump.keyboard_state().pressed_scancodes().collect();
    for key in pressed_keys.iter() {
        match key {
            Scancode::Num1  => chip_keys[0x1] = 1,
            Scancode::Num2  => chip_keys[0x2] = 1,
            Scancode::Num3  => chip_keys[0x3] = 1,
            Scancode::Num4  => chip_keys[0xC] = 1,
            Scancode::Q     => chip_keys[0x4] = 1,
            Scancode::W     => chip_keys[0x5] = 1,
            Scancode::E     => chip_keys[0x6] = 1,
            Scancode::R     => chip_keys[0xD] = 1,
            Scancode::A     => chip_keys[0x7] = 1,
            Scancode::S     => chip_keys[0x8] = 1,
            Scancode::D     => chip_keys[0x9] = 1,
            Scancode::F     => chip_keys[0xE] = 1,
            Scancode::Z     => chip_keys[0xA] = 1,
            Scancode::X     => chip_keys[0x0] = 1,
            Scancode::C     => chip_keys[0xB] = 1,
            Scancode::V     => chip_keys[0xF] = 1,
            _               => (),
        }
    }
}
