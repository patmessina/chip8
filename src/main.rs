mod chip8;


fn main() {
    env_logger::init();

    let mut chip8 = chip8::Chip8::new(None::<chip8::Chip8Config>);

    chip8.log();
    chip8.run();

}
