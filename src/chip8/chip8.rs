extern crate sdl2;

use std::{fs, io};
use std::time::{Duration, Instant};

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::audio::AudioDevice;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::Sdl;

use log::{info, debug};
// use log::{info, warn, error, debug, trace};
// use log::info;

const SCREEN_WIDTH: u32 = 64;
const SCREEN_HEIGHT: u32 = 32;

// Chip8
pub struct Chip8Config {
    display_scale: u32,
    program: String,
}

impl Chip8Config {

    pub fn new() -> Self {
        Chip8Config {
            display_scale: 10,
            program: "roms/ibm-logo.ch8".to_string(),
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
    display_scale: u32,
    pc: u16,  // program counter which points at the current instruction in memory
    stack: [u16; 32],  // stack for 16-bit addresses which is used to call subroutines/functions
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
        let (sdl_context, canvas, audio_device) = Chip8::init_sdl();

        // Create chip 8 instance
        let mut chip8 = Chip8 {
            memory: [0; 4096],
            program: config.program,
            stack: [0; 32],  // stack for 16-bit addresses which is used to call subroutines/functions
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
                    sdl2::event::Event::Quit {..} => return,
                    _ => {}
                }
            }

            // update display every frame (60hz)
            // this could be at dif
            if last_update.elapsed() >= std::time::Duration::from_millis(16) {
                self.update_timers(); // This may need to be seperate from the fetch/decode/execute cycle
                self.draw();
                // fetch, decode, execute
                let opcode = self.fetch_opcode();
                self.decode_and_execute(opcode);
                last_update = Instant::now();
            }


            // delay to reduce cpu usage
            std::thread::sleep(Duration::from_millis(1));
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
        
        let nibbles = (
            // F: First nibble tells you what kind of instruction it is.
            // (opcode & 0xF000) >> 12 as u8,
            0xF000 as u16,
            // X:  The second nibble. Used to look up one of the 16 registers
            // (VX) from V0 through VF.
            // (opcode & 0x0F00) >> 8 as u8,
            0x0F00 as u16,
            // Y: The third nibble. Also used to look up one of the 16 registers
            // (VY) from V0 through VF.
            // (opcode & 0x00F0) >> 4 as u8,
            0x00F0 as u16,
            // N: The fourth nibble. A 4-bit number.
            // (opcode & 0x000F) as u8,
            0x000F as u16,
        );

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
                _ => info!("Unknown opcode: 0x{:04X}", opcode),
            }
            0x1000 => self.jump(opcode),
            0x6000 => self.set(opcode),
            0x7000 => self.add(opcode),
            0xA000 => self.set_index_register(opcode),
            0xD000 => draw_sprite(opcode),
            _ => info!("Unknown opcode: 0x{:04X}", opcode),
        }
        
        
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
        let x = self.v[x_index] % 64;

        // Set the Y coordinate to the value in VY modulo 32 (or VY & 31)
        let y = self.v[y_index] % 62;

        // Set VF to 0
        self.v[0xF] = 0;

        // For N rows:
        // Increment Y (VY is not incremented)
        // Stop if you reach the bottom edge of the screen

        // for N rows
        for row in 0..(opcode & 0x000F) as usize {
            // Get the Nth byte of sprite data, counting from the memory address
            // in the I register (I is not incremented)
            let sprite_byte = self.memory[(self.i + row as u16) as usize];

            // For each of the 8 pixels/bits in this sprite row (from left to
            // right, ie. from most to least significant bit):
            for col in 0..8 {

            // Get the pixel value (0 or 1) at this position in the sprite row
            let sprite_pixel = (sprite_byte >> (7 - col)) & 0x1;

            // If the current pixel in the sprite row is on and the pixel at
            // coordinates X,Y on the screen is also on, turn off the pixel and
            // set VF to 1
            
            //  Or if the current pixel in the sprite row is on and the screen pixel is not, draw the pixel at the X and Y coordinates

            //  If you reach the right edge of the screen, stop drawing this row

            //  Increment X (VX is not incremented)

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
        self.v[register as usize] += value as u8;
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
    fn init_sdl() -> (Sdl, Canvas<Window>, AudioDevice<SquareWave>) {
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

        (sdl_context, canvas, audio_device)
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
            if self.sound_timer == 1 {
                self.play_sound(); // TODO: this should be using an event bus
            }
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
        debug!("  stack: {:?}", self.stack);
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