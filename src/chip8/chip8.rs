extern crate sdl2;

use std::collections::HashMap;
use std::{fs, io};
use std::time::{Duration, Instant};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::audio::AudioDevice;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::Sdl;

use rand::Rng;

use log::{info, debug};
// use log::{info, warn, error, debug, trace};
// use log::info;

const SCREEN_WIDTH: u32 = 64;
const SCREEN_HEIGHT: u32 = 32;
const KEY_COUNT: usize = 16;

const SET_VX_FROM_VY_IN_SHIFT: bool = false;

// Chip8
pub struct Chip8Config {
    pub display_scale: u32,
    pub program: String,
}

impl Chip8Config {

    pub fn new() -> Self {
        Chip8Config {
            display_scale: 10,
            // program: "roms/ibm-logo.ch8".to_string(),
            program: "roms/test_opcode.ch8".to_string(),
        }
    }

    // print out config
    pub fn log(&self) {
        info!("Chip8Config");
        info!("  display_scale: {}", self.display_scale);
    }

}

pub struct Chip8 {
    memory: [u8; 4096],  // chip-8 has direct access to up to 4Kib of Ram
    display: [bool; (SCREEN_WIDTH * SCREEN_HEIGHT) as usize], // 64x32 pixels - monochrome 
    keys: Keys,  // 16 keys, 0-F
    key_map: HashMap<Keycode, usize>,
    display_scale: u32,
    pc: u16,  // program counter which points at the current instruction in memory
    stack: Stack,  // stack for 16-bit addresses which is used to call subroutines/functions
                         // and return from them
    i: u16,  // index register which is used to point at locations in memory
    v: [u8; 16],  // general purpose registers numbered 0 through F hexadecimal, ie. 0 through 15
                  // in decimal, called V0 through VF
                  //   VF is also used as a flag register, many instructions will set it to either
                  //   1 or 0 based on some rule, for example  using it as a carry flag
    delay_timer: u8,  // is used to decrement at a rate of 60 hz 
    sound_timer: u8, // an 8 bit sound timer which functions like the delay timer, but which also
                     // gives off a beeping sound as long as its not 0
    program: String,
    sdl_context: Sdl,
    canvas: Canvas<Window>,
    audio_device: AudioDevice<SquareWave>,
}

impl Chip8 {

    pub fn new(config: Option<Chip8Config>) -> Self {

        // if config is none, set a default config
        let config = match config {
            Some(config) => config,
            None => Chip8Config::new(),
        };

        config.log();

        // initialize sdl2
        // TODO: this is maybe a bit much to be returning... Maybe a return type?
        let (sdl_context, canvas, audio_device, key_map) = Chip8::init_sdl();

        // Create chip 8 instance
        let mut chip8 = Chip8 {
            memory: [0; 4096],
            program: config.program,
            keys: Keys::new(),
            key_map,
            stack: Stack::new(),  // stack for 16-bit addresses which is used to call subroutines/functions
            display: [false; (SCREEN_WIDTH * SCREEN_HEIGHT) as usize], // 64x32 pixels - monochrome -- super chip is 128*64
            display_scale: config.display_scale,
            pc: 0x200, // The first CHIP-8 interpreter (on the COSMAC VIP computer) was also
                       // located in RAM, from address 000 to 1FF. It would expect a CHIP-8 program
                       // to be loaded into memory after it, starting at address 200 (512 in
                       // decimal).
            i: 0,
            v: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            sdl_context,
            canvas,
            audio_device,
        };

        chip8.set_fonts();
        chip8.load_program().unwrap();

        // TODO: Display


        chip8

    }

    pub fn run(&mut self) {
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        let mut last_update = Instant::now();

        // Main loop
        loop {
            // Handle events for keyboard, window, etc.
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} => return,
                    Event::KeyDown { keycode: Some(keycode), .. } => {
                        if let Some(&key) = self.key_map.get(&keycode) {
                            self.keys.set_key(key, true);
                        }
                    },
                    Event::KeyUp { keycode: Some(keycode), .. } => {
                        if let Some(&key) = self.key_map.get(&keycode) {
                            self.keys.set_key(key, false);
                        }
                    },
                    _ => {}
                }
            }

            // update display every frame (60hz)
            // this could be at dif
            if last_update.elapsed() >= std::time::Duration::from_millis(16) {
                self.update_timers(); // This may need to be seperate from the fetch/decode/execute cycle
                self.draw();
                last_update = Instant::now();
            }
            // fetch, decode, execute
            let opcode = self.fetch_opcode();
            self.decode_and_execute(opcode);


            // delay to reduce cpu usage
            std::thread::sleep(Duration::from_micros(500));
        }
    }

    /*
     * Read the instruction that PC is currently pointing at from memory. An
     * instruction is two bytes, so you will need to read two successive bytes
     * from memory and combine them into one 16-bit instruction.
     */
    fn fetch_opcode(&mut self) -> u16 {
        // let opcode = (self.memory[self.pc as usize] as u16) << 8 
        //     | self.memory[self.pc as usize + 1] as u16;
        // let pc = self.pc as usize;
        // let byte1 = self.memory[pc] as u16;
        // let byte2 = self.memory[pc + 1] as u16;


        let pc = self.pc as usize;

        let byte1 = self.memory[pc] as u16;
        let byte2  = self.memory[pc + 1] as u16;

        self.pc += 2;

        byte1 << 8 | byte2
    }

    fn decode_and_execute(&mut self, opcode: u16) {
        
        // Mask off (with a “binary AND”) the first number in the instruction,
        // and have one case per number. Some of these cases will need separate
        // switch statements inside them to further decode the instruction.

        // Although every instruction will have a first nibble that tells you
        // what kind of instruction it is, the rest of the nibbles will have
        // different meanings. To differentiate these meanings, we usually call
        // them different things, but all of them can be any hexadecimal number
        // from 0 to F:
        
        // F: First nibble tells you what kind of instruction it is.
        // (opcode & 0xF000) >> 12 as u8,

        // X:  The second nibble. Used to look up one of the 16 registers
        // (VX) from V0 through VF.
        // (opcode & 0x0F00) >> 8 as u8,

            // Y: The third nibble. Also used to look up one of the 16 registers
            // (VY) from V0 through VF.
            // (opcode & 0x00F0) >> 4 as u8,


            // N: The fourth nibble. A 4-bit number.
            // (opcode & 0x000F) as u8,

        // NN: The second byte (third and fourth nibbles). An 8-bit immediate
        // number.
        // let nn = (opcode & 0x00FF) as u8;
        let nn = 0x00FF as u16;
        
        // NNN: The second, third and fourth nibbles. A 12-bit immediate memory
        // address.
        // let nnn = opcode & 0x0FFF as u16;
        let nnn = 0x0FFF as u16;

        match opcode & 0xF000 {
            0x0000 => match opcode & nn {
                0x00E0 => self.clear_screen(),
                0x00EE => self.return_from_subroutine(),
                _ => info!("Unknown opcode: 0x{:04X}", opcode),
            }
            0x1000 => self.jump(opcode),
            0x2000 => self.subroutine(opcode),
            0x3000 => self.skip_if_nn_is_equal(opcode),
            0x4000 => self.skip_if_nn_is_not_equal(opcode),
            0x5000 => self.skip_if_vx_and_vy_are_equal(opcode),
            0x6000 => self.set(opcode),
            0x7000 => self.add(opcode),
            0x8000 => match opcode & 0x000F {
                0x0000 => self.set_vx_from_vy(opcode),
                0x0001 => self.vx_binary_or_vy(opcode),
                0x0002 => self.vx_binary_and_vy(opcode),
                0x0003 => self.vx_binary_xor_vy(opcode),
                0x0004 => self.vx_add_vy(opcode),
                0x0005 => self.vx_subtract_vy(opcode),
                0x0006 => self.vx_shift_right(opcode, SET_VX_FROM_VY_IN_SHIFT),
                0x0007 => self.vx_subtract_from_vy(opcode),
                0x000E => self.vx_shift_left(opcode, SET_VX_FROM_VY_IN_SHIFT),
                _ => info!("Unknown opcode: 0x{:04X}", opcode),
            },
            0x9000 => self.skip_if_vx_and_vy_are_not_equal(opcode),
            0xA000 => self.set_index_register(opcode),
            0xB000 => self.jump_with_offset(opcode),
            0xC000 => self.random(opcode),
            0xD000 => self.draw_sprite(opcode),
            0xE000 => match opcode & 0x00FF {
                0x009E => self.skip_if_key_is_pressed(opcode),
                0x00A1 => self.skip_if_key_is_not_pressed(opcode),
                _ => info!("Unknown opcode: 0x{:04X}", opcode),
            },
            0xF000 => match opcode & 0x00FF {
                0x0007 => self.set_vx_to_delay_timer(opcode),
                0x000A => self.wait_for_keypress(opcode),
                0x0015 => self.set_delay_timer(opcode),
                0x0018 => self.set_sound_timer(opcode),
                0x001E => self.add_vx_to_index_register(opcode),
                0x0029 => self.set_index_to_font(opcode),
                0x0033 => self.store_bcd(opcode),
                0x0055 => self.store_registers(opcode),
                0x0065 => self.load_registers(opcode),
                _ => info!("Unknown opcode: 0x{:04X}", opcode),
            },
            _ => info!("Unknown opcode: 0x{:04X}", opcode),
        }
        
        
    }

    // FX55
    // For FX55, the value of each variable register from V0 to VX inclusive (if
    // X is 0, then only V0) will be stored in successive memory addresses,
    // starting with the one that’s stored in I. V0 will be stored at the
    // address in I, V1 will be stored in I + 1, and so on, until VX is stored
    // in I + X.
    fn store_registers(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        for i in 0..=x {
            self.memory[self.i as usize + i] = self.v[i];
        }
    }

    fn load_registers(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        for i in 0..=x {
            self.v[i] = self.memory[self.i as usize + i];
        }
    }
    
    // FX33
    // Binary-coded decimal (BCD) representation of VX, with the most significant
    //
    // This instruction is a little involved. It takes the number in VX (which
    // is one byte, so it can be any number from 0 to 255) and converts it to
    // three decimal digits, storing these digits in memory at the address in
    // the index register I. For example, if VX contains 156 (or 9C in
    // hexadecimal), it would put the number 1 at the address in I, 5 in address
    // I + 1, and 6 in address I + 2.
    fn store_bcd(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let value = self.v[x];
        let hundreds = value / 100;
        let tens = (value % 100) / 10;
        let ones = value % 10;
        self.memory[self.i as usize] = hundreds;
        self.memory[self.i as usize + 1] = tens;
        self.memory[self.i as usize + 2] = ones;
    }
    
    // FX29
    // Set I to the memory address of the hexadecimal character in VX.
    fn set_index_to_font(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let char_value = self.v[x] as u16;
        
        // Set I register to the memory address of the hexadecimal character in VX
        // We started the font at 0x50 in memory
        // Each character is 5 bytes long (look at each row)
        self.i = 0x50 + (char_value * 5);
    }

    // FX0A
    // This instruction “blocks”; it stops executing instructions and waits for
    // key input (or loops forever, unless a key is pressed).
    // That is we need to decrement the program counter by 2 so that the
    // instruction is executed again.
    fn wait_for_keypress(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let mut key_pressed = false;
        for i in 0..KEY_COUNT {
            if self.keys.is_key_pressed(i) {
                self.v[x] = i as u8;
                key_pressed = true;
            }
        }
        if !key_pressed {
            self.pc -= 2;
        }
    }
    
    
    // FX1E
    // The index register I will get the value in VX added to it
    // Unlike other arithmetic instructions, this did not affect VF on overflow
    // on the original COSMAC VIP. However, it seems that some interpreters set
    // VF to 1 if I “overflows” from 0FFF to above 1000 (outside the normal
    // addressing range). This wasn’t the case on the original COSMAC VIP, at
    // least, but apparently the CHIP-8 interpreter for Amiga behaved this way.
    // At least one known game, Spacefight 2091!, relies on this behavior. I
    // don’t know of any games that rely on this not happening, so perhaps it’s
    // safe to do it like the Amiga interpreter did.
    fn add_vx_to_index_register(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let sum = self.i + self.v[x] as u16;
        self.v[0xF] = if sum > 0xFFF { 1 } else { 0 };
        self.i = sum & 0xFFF;
    }

    // FX07
    // set vx to the current value of the delay timer
    fn set_vx_to_delay_timer(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.v[x] = self.delay_timer;
    }

    // FX15
    // Set the delay timer to the value in VX
    fn set_delay_timer(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.delay_timer = self.v[x];
    }

    // FX18 
    // sets the sound timer to the value in VX
    fn set_sound_timer(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.sound_timer = self.v[x];
    }

    // EX9E
    // EX9E will skip one instruction if the key stored in VX is pressed.
    // (increment pc by 2)
    fn skip_if_key_is_pressed(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        if self.keys.is_key_pressed(self.v[x] as usize) {
            self.pc += 2;
        }
    }

    // EXA1
    // EXA1 will skip one instruction if the key stored in VX is not pressed.
    // (increment pc by 2)
    fn skip_if_key_is_not_pressed(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        if !self.keys.is_key_pressed(self.v[x] as usize) {
            self.pc += 2;
        }
    }
    
    // CXNN
    // Generate a random number from 0 to NN, and then BINARY AND it with NN
    // then put the value in VX.
    fn random(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let nn = (opcode & 0x00FF) as u8;

        let mut rng = rand::thread_rng();
        let random_number: u8 = rng.gen_range(0..nn);
        self.v[x] = random_number & nn;
    }
    
    // 0xBNNN
    // In the original COSMAC VIP interpreter, this instruction jumped to the
    // address NNN plus the value in the register V0. This was mainly used for
    // “jump tables”, to quickly be able to jump to different subroutines based
    // on some input.
    // 
    // Starting with CHIP-48 and SUPER-CHIP, it was (probably unintentionally)
    // changed to work as BXNN: It will jump to the address XNN, plus the value
    // in the register VX. So the instruction B220 will jump to address 220 plus
    // the value in the register V2.
    //    
    // The BNNN instruction was not widely used, so you might be able to just
    // implement the first behavior (if you pick one, that’s definitely the one
    // to go with). If you want to support a wide range of CHIP-8 programs, make
    // this “quirk” configurable.
    // 
    // This just implements the original COSMAC VIP behavior
    fn jump_with_offset(&mut self, opcode: u16) {
        let address = opcode & 0x0FFF;
        self.pc = self.v[0] as u16 + address;
    }

    // 8XYE
    // See 8xy6 for more information
    // shift left instead of right
    fn vx_shift_left(&mut self, opcode: u16, set_vx: bool) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        if set_vx {
            self.v[x] = self.v[y];
        }
        self.v[0x000F] = self.v[x] >> 7;
        self.v[x] <<= 1;
    }

    // 8XY6
    // In the CHIP-8 interpreter for the original COSMAC VIP, this instruction
    // did the following: It put the value of VY into VX, and then shifted the
    // value in VX 1 bit to the right (8XY6) or left (8XYE). VY was not
    // affected, but the flag register VF would be set to the bit that was
    // shifted out.
    //    
    // However, starting with CHIP-48 and SUPER-CHIP in the early 1990s, these
    // instructions were changed so that they shifted VX in place, and ignored
    // the Y completely.
    fn vx_shift_right(&mut self, opcode: u16, set_vx: bool) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        if set_vx {
            self.v[x] = self.v[y];
        }
        self.v[0x000F] = self.v[x] & 0x0001;
        self.v[x] >>= 1;
    }
    
    // 8XY7
    // VX is set to VY - VX.
    // TODO: Not sure what to do if they are same size, currently 
    // setting it as so
    // If VX is greater than or equal VY, VF is set to 1, otherwise 0.
    fn vx_subtract_from_vy(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[0xF] = if self.v[y] >= self.v[x] { 1 } else { 0 };
        self.v[x] = self.v[y].wrapping_sub(self.v[x]);
    }

    // 8XY5
    // VX is set to VX - VY. 
    // TODO: Not sure what to do if they are same size, currently 
    // setting it as so
    // If VX is greater than or equal VY, VF is set to 1, otherwise 0.
    fn vx_subtract_vy(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[0xF] = if self.v[x] >= self.v[y] { 1 } else { 0 };
        // wrapping sub is a function on u8
        self.v[x] = self.v[x].wrapping_sub(self.v[y]);
    }

    // 8XY4
    // VX is set to VX + VY. VF is set to 1 if there is a carry, 0 if not.
    // VY is not affected
    // Unlike 7XNN, this addition will affect the carry flag, so if VX + VY is
    // greater than 255, VF will be set to 1. If it’s less than or equal to 255,
    // VF will be set to 0.
    fn vx_add_vy(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let sum = self.v[x] as u16 + self.v[y] as u16;
        self.v[x] = sum as u8;
        self.v[0xF] = if sum > 255 { 1 } else { 0 };
    }

    // 8XY3
    // VX is set to the bitwise/binary logical XOR of VX and VY.
    // VY is not affected
    fn vx_binary_xor_vy(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[x] ^= self.v[y];
    }

    // 8XY2 
    // VX is set to the bitwise/binary logical AND of VX and VY.
    // VY is not affected
    fn vx_binary_and_vy(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[x] &= self.v[y];
    }

    // 8XY1
    // VX is set to the bitwise/binary logical OR of VX and VY.
    // VY is not affected
    fn vx_binary_or_vy(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[x] |= self.v[y];
    }

    // 8XY0
    // VX is set to the value of VY.
    fn set_vx_from_vy(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[x] = self.v[y];
    }

    // 9XY0
    // 9XY0 skips if the values in VX and VY are not equal
    fn skip_if_vx_and_vy_are_not_equal(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        if self.v[x] != self.v[y] {
            self.pc += 2;
        }
    }

    // 5XY0
    // 5XY0 skips if the values in VX and VY are equal
    fn skip_if_vx_and_vy_are_equal(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        if self.v[x] == self.v[y] {
            self.pc += 2;
        }
    }

    // 4XNN
    // 4XNN will skip one instruction if the value in VX is NOT equal to NN
    fn skip_if_nn_is_not_equal(&mut self, opcode: u16) {
        let register = (opcode & 0x0F00) >> 8;
        let value = opcode & 0x00FF;
        if self.v[register as usize] != value as u8 {
            self.pc += 2;
        }
    }
    
    // 3XNN
    // 3XNN will skip one instruction if the value in VX is equal to NN
    fn skip_if_nn_is_equal(&mut self, opcode: u16) {
        let register = (opcode & 0x0F00) >> 8;
        let value = opcode & 0x00FF;
        if self.v[register as usize] == value as u8 {
            self.pc += 2;
        }
    }

    // subroutine
    // 2NNN calls the subroutine at memory location NNN. In other words, just
    // like 1NNN, you should set PC to NNN. However, the difference between a
    // jump and a call is that this instruction should first push the current PC
    // to the stack, so the subroutine can return later.
    fn subroutine(&mut self, opcode: u16) {
        let address = opcode & 0x0FFF;
        self.stack.push(self.pc).unwrap();
        self.pc = address;
    }

    // return_from_subroutine
    // Returning from a subroutine is done with 00EE, and it does this by
    // removing (“popping”) the last address from the stack and setting the PC
    // to it.
    fn return_from_subroutine(&mut self) {
        self.pc = self.stack.pop().unwrap();
    }

    // 0xDXYN
    //
    // This will draw an N pixels tall sprite from the memory location that the
    // I index register is holding to the screen, at the horizontal X coordinate
    // in VX and the Y coordinate in VY. All the pixels that are “on” in the
    // sprite will flip the pixels on the screen that it is drawn to (from left
    // to right, from most to least significant bit). If any pixels on the
    // screen were turned “off” by this, the VF flag register is set to 1.
    // Otherwise, it’s set to 0.
    fn draw_sprite(&mut self, opcode: u16) {
        let  x_index = ((opcode & 0x0F00) >> 8) as usize;
        let  y_index = ((opcode & 0x00F0) >> 4) as usize;

        // Set the X coordinate to the value in VX modulo 64 (or, equivalently, VX &
        // 63, where & is the binary AND operation)
        // & 63 is the same as % 64
        let x = (self.v[x_index] % 64) as u32;

        // Set the Y coordinate to the value in VY modulo 32 (or VY & 31)
        let y = (self.v[y_index] % 32) as u32;

        // Set VF to 0
        self.v[0xF] = 0;

        let height = (opcode & 0x000F) as usize;

        // dxyn 
        // for N rows (how tall)
        for row in 0..height {

            // Get the Nth byte of sprite data, counting from the memory address
            // in the I register (I is not incremented)
            let sprite_byte = self.memory[(self.i + row as u16) as usize];

            // stop if Y + row exceeds the screen height (don't wrap vertically)
            if y + row as u32 >= SCREEN_HEIGHT {
                break;
            }

            // For each of the 8 pixels/bits in this sprite row (from left to
            // right, ie. from most to least significant bit):
            for col in 0..8 {

                // Get the pixel value (0 or 1) at this position in the sprite row
                let sprite_pixel = (sprite_byte >> (7 - col)) & 0x1;

                // Stop if X + col exceeds the screen width (don't wrap horizontally)
                if x + col >= SCREEN_WIDTH {
                    break;
                }

                // Calculate the screen coordinates
                let screen_x = (x + col) % SCREEN_WIDTH;
                let screen_y = (y + row as u32) % SCREEN_HEIGHT;

                // Get the index of the pixel in the display array
                let pixel_index = (screen_y as u32 * SCREEN_WIDTH + screen_x as u32) as usize;

                // If the current pixel in the sprite row is on and the pixel at
                // coordinates X,Y on the screen is also on, turn off the pixel and
                // set VF to 1
            
                // if the current pixel in the sprite row is on...
                if sprite_pixel == 1 {
                    // if the pixel at the X and Y coordinates is also on...
                    // turn off the pixel and set VF to 1
                    if self.display[pixel_index] {
                        self.display[pixel_index] = false;
                        self.v[0xF] = 1;
                    } else {

                    //  if the current pixel in the sprite row is on and the screen
                    //  pixel is not, draw the pixel at the X and Y coordinates
                        self.display[pixel_index] = true;
                    }

                } 

            }

        }

    }

    fn set_index_register(&mut self, opcode: u16) {
        self.i = opcode & 0x0FFF;
    }
    
    // add the value nn to vx
    // Note that on most other systems, and even in some of the other CHIP-8
    // instructions, this would set the carry flag if the result overflowed 8
    // bits; in other words, if the result of the addition is over 255.  For
    // this instruction, this is not the case. If V0 contains FF and you execute
    // 7001, the CHIP-8’s flag register VF is not affected.
    fn add(&mut self, opcode: u16) {
        let register = (opcode & 0x0F00) >> 8;
        let value = opcode & 0x00FF;
        // self.v[register as usize] += value as u8;
        self.v[register as usize] = self.v[register as usize].wrapping_add(value as u8);
    }

    // set register VX to the value of NN
    fn set(&mut self, opcode: u16) {
        let register = (opcode & 0x0F00) >> 8;
        let value = opcode & 0x00FF;
        self.v[register as usize] = value as u8;
    }

    // This instruction should simply set PC to NNN, causing the program to jump 
    // to that memory location. Do not increment the pc aftwords, it jumps directly there
    fn jump(&mut self, opcode: u16) {
        self.pc = opcode & 0x0FFF;
    }

    // Clear the display
    fn clear_screen(&mut self) {
        self.display.fill(false);
    }

    // Initialize sdl2 with result sdl and canvas
    fn init_sdl() -> (Sdl, Canvas<Window>, AudioDevice<SquareWave>, HashMap<Keycode, usize>) {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("Chip8", 
                SCREEN_WIDTH * 10,
                SCREEN_HEIGHT * 10)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.present();

        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),  // mono
            samples: None,  // default sample size
        };

        let audio_subsystem = sdl_context.audio().unwrap();
        let audio_device = audio_subsystem.open_playback(
            None, &desired_spec, |spec| {
                // Show obtained AudioSpec
                info!("{:?}", spec);

                // initialize the audio callback
                SquareWave {
                    phase_inc: 440.0 / spec.freq as f32,
                    phase: 0.0,
                    volume: 0.25,
                }
        }).unwrap();

        let mut key_map: HashMap<Keycode, usize> = HashMap::new();
        key_map.insert(Keycode::Num1, 0x1);
        key_map.insert(Keycode::Num2, 0x2);
        key_map.insert(Keycode::Num3, 0x3);
        key_map.insert(Keycode::Num4, 0xC);
        key_map.insert(Keycode::Q, 0x4);
        key_map.insert(Keycode::W, 0x5);
        key_map.insert(Keycode::E, 0x6);
        key_map.insert(Keycode::R, 0xD);
        key_map.insert(Keycode::A, 0x7);
        key_map.insert(Keycode::S, 0x8);
        key_map.insert(Keycode::D, 0x9);
        key_map.insert(Keycode::F, 0xE);
        key_map.insert(Keycode::Z, 0xA);
        key_map.insert(Keycode::X, 0x0);
        key_map.insert(Keycode::C, 0xB);
        key_map.insert(Keycode::V, 0xF);

        (sdl_context, canvas, audio_device, key_map)
    }

    fn load_program(&mut self) -> Result<(), io::Error> {
        // read in binary file into a byte vector
        let program = fs::read(&self.program)?;

        if 0x200 + program.len() > self.memory.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Program is too large to fin in memory"));
        }

        self.memory[0x200..(0x200 + program.len())]
            .copy_from_slice(&program);

        Ok(())
    }

    fn draw(&mut self) {
        // clear screen
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();

        // set draw color for pixels that are "on"
        self.canvas.set_draw_color(Color::WHITE);

        // draw pixels
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let pixel_index = (y * SCREEN_WIDTH + x) as usize;
                if self.display[pixel_index] {
                    let rect = Rect::new(
                        (x as u32 * self.display_scale) as i32,
                        (y as u32 * self.display_scale) as i32,
                        self.display_scale,
                        self.display_scale,
                    );
                    self.canvas.fill_rect(rect).unwrap();
                }
            }

        }
        self.canvas.present();
    }

    fn update_timers(&mut self) {

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.play_sound(); // TODO: this should be using an event bus
            self.sound_timer -= 1;
        } 
        if self.sound_timer == 0 {
            self.stop_sound();
        }

    }

    pub fn log(&self) {
        info!("Chip8 info");
        info!("  pc: 0x{:04X}", self.pc);
        info!("  i: 0x{:04X}", self.i);
        info!("  delay_timer: 0x{:02X}", self.delay_timer);
        info!("  sound_timer: 0x{:02X}", self.sound_timer);
        info!("  display_scale: {}", self.display_scale);
        debug!("  display: {:?}", self.display);
        debug!("  stack: {:?}", self.stack.stack);
        info!("  v: {:?}", self.v);
        debug!("  memory: {:?}", self.memory);
        info!("Chip8 info end");
    }

    /*
     * set_fonts
     *
     * The CHIP-8 emulator should have a built-in font, with sprite data representing the
     * hexadecimal numbers from 0 through F. Each font character should be 4 pixels wide by 5
     * pixels tall. These font sprites are drawn just like regular sprites
     *
     * There’s a special instruction for setting I to a character’s address, so you can choose
     * where to put it. Anywhere in the first 512 bytes (000–1FF) is fine. For some reason, it’s
     * become popular to put it at 050–09F, so you can follow that convention if you want.
     *
     */

    fn set_fonts(&mut self) {
        let fonts: [u8; 80] = [
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

        // start at memory address 80
        self.memory[0x50..0x50 + fonts.len()].copy_from_slice(&fonts);
    }

    fn play_sound(&mut self) {
        self.audio_device.resume();
    }

    fn stop_sound(&mut self) {
        self.audio_device.pause();
    }

}

struct Stack {
    stack: [u16; 32],
    i: usize, // index to track top of stack
}

impl Stack {
    fn new() -> Self {
        Stack {
            stack: [0; 32],
            i: 0,
        }
    }

    fn push(&mut self, value: u16) -> Result<(), &str> {
        if self.i >= self.stack.len() {
            return Err("Stack overflow");
        }
        self.stack[self.i] = value;
        self.i += 1;
        Ok(())
    }

    fn pop(&mut self) -> Result<u16, &str> {
        if self.i == 0 {
            return Err("Stack underflow");
        }
        self.i -= 1;
        Ok(self.stack[self.i])
    }
}

struct Keys {
    state: [bool; KEY_COUNT],
}

impl Keys {

    fn new() -> Self {
        Keys {
            state: [false; KEY_COUNT],
        }
    }

    fn set_key(&mut self, key: usize, pressed: bool) {
        if (key as usize) < KEY_COUNT {
            self.state[key] = pressed;
        }
    }

    fn is_key_pressed(&self, key: usize) -> bool {
        if (key as usize) < KEY_COUNT {
            self.state[key]
        } else {
            false
        }
    }

}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {

    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x = if self.phase < 0.5 { self.volume } else { -self.volume };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }

}