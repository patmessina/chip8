mod chip8;


fn main() {
    env_logger::init();

    let mut chip8_config = chip8::Chip8Config::new();

    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        chip8_config.program = args[1].clone();
    } 

    let mut chip8 = chip8::Chip8::new(Some(chip8_config));

    chip8.log();
    chip8.run();

}
