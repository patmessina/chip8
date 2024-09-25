mod chip8;
mod assembler;


fn main() {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("Invalid number of arguments");
        eprintln!("Usage: chip8 <emulate|assemble> <program>");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "assemble" => {

            let source_path = args[2].clone();
            let mut target_path = "output.ch8".to_string();
            if  args.len() > 3 {
                target_path = args[3].clone();
            }
            println!("Assembling program {} to {}", source_path, target_path);
            assembler::assemble(source_path).unwrap();


            // let mut ass = assembler::Chip8Assembler::new(source_path, target_path);
            // ass.assemble().unwrap()
        },
        "emulate" => {
            println!("Emulating program: {}", args[2]);
            let mut chip8_config = chip8::Chip8Config::new();
            chip8_config.program = args[2].clone();
            let mut chip8 = chip8::Chip8::new(Some(chip8_config));
            chip8.log();
            chip8.run();

        },
        _ => {
            println!("Usage: chip8 <emulate|assemble> <program>");
            std::process::exit(1);
        }
    }

}
