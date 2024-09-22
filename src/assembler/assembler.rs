use log::{info,debug};

use std::{collections::HashMap, fs::File, io::{self, BufRead}};


pub struct Chip8Assembler {
    pub source_path: String,
    pub target_path: String,
    errors: Vec<String>,
    tokens: Vec<Token>,
    opcodes: Vec<u16>,
    labels: HashMap<String, u16>,
    registers: HashMap<String, u16>,
    org: u16,
}

impl Chip8Assembler {
    pub fn new(source_path: String, target_path: String) -> Chip8Assembler {
        let registers: HashMap<String, u16> = HashMap::from([
            ("v0".to_string(), 0),
            ("v1".to_string(), 1),
            ("v2".to_string(), 2),
            ("v3".to_string(), 3),
            ("v4".to_string(), 4),
            ("v5".to_string(), 5),
            ("v6".to_string(), 6),
            ("v7".to_string(), 7),
            ("v8".to_string(), 8),
            ("v9".to_string(), 9),
            ("va".to_string(), 10),
            ("vb".to_string(), 11),
            ("vc".to_string(), 12),
            ("vd".to_string(), 13),
            ("ve".to_string(), 14),
            ("vf".to_string(), 15),
        ]);

        Chip8Assembler {
            source_path,
            target_path,
            errors: Vec::new(),
            tokens: Vec::new(),
            opcodes: Vec::new(),
            labels: HashMap::new(),
            registers,
            org: 0x200,
        }
    }

    pub fn assemble(&mut self) -> Result<(), io::Error> {
        info!("Assembling program {} to {}", self.source_path, self.target_path);

        // TODO: read file in
        self.parse_file()?;

        self.set_origin()?;

        self.set_labels();

        // print out labels
        for (label, address) in &self.labels {
            debug!("Label: {} -- Address: {:X}", label, address);
        }

        self.parse()?;

        for err in &self.errors {
            println!("{}", err);
        }

        Ok(())
    }


    fn parse_file(&mut self) -> Result<(), io::Error> {

        let file = File::open(&self.source_path)?;
        let reader = io::BufReader::new(file);

        for (i, line) in reader.lines().enumerate() {
            let line = match line {
                Ok(line) => line,
                Err(e) => {
                    self.errors.push(format!("Error reading line number {}: {}", i, e));
                    continue;
                }
            };

            let line = line.trim();

            // Ignore empty lines
            if line.is_empty() {
                continue;
            }

            // Ignore comments
            if line.starts_with("//") {
                continue;
            }

            // split line on whitespace
            let parts: Vec<&str> = line.split_whitespace().collect();

            match parts[0] {
                "org" => {
                    if parts.len() != 2 {
                        self.errors.push(format!("Invalid origin directive on line {}", i));
                        continue;
                    }
                    let address = parts[1].strip_prefix("0x").unwrap_or(parts[1]);
                    match u16::from_str_radix(address, 16) {
                        Ok(address) => {
                            self.tokens.push(Token::new(i as i32,
                                TokenType::Origin(address)));
                        },
                        Err(e) => {
                            self.errors.push(format!("Invalid address on line {} -- value {}: {}", 
                            i, parts[1], e));
                        }
                    }
                },
                label if parts[0].ends_with(':') => {
                    let label = label.trim_end_matches(':');
                    self.tokens.push(Token::new(i as i32,
                        TokenType::Label(label.to_string())));
                },
                _ => {

                    let mut args: Vec<String> = Vec::new();

                    if parts.len() > 1 {
                        args = parts[1..].iter().map(|s| s.to_string()).collect();
                    }

                    // instruction
                    let instruction = Instruction {
                        name: parts[0].to_string(),
                        args,
                    };

                    self.tokens.push(Token::new(i as i32,
                         TokenType::Instruction(instruction)));
                }
            }

        }

        for token in &self.tokens {
            debug!("{}", token.to_string());
        }

        if self.errors.len() > 0 {
            self.print_errors();
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                "Errors parsing file"));
        }

        Ok(())
    }

    // parse instructions into opcodes
    fn parse(&mut self) -> Result<(), io::Error> {

        // for token in &self.tokens {
        for i in 0..self.tokens.len() {
        // for token in &self.tokens {
            let token = self.tokens[i].clone();

            // clone token
            match token.token_type {
                TokenType::Label(_) => {},
                TokenType::Origin(address) => {},
                TokenType::Instruction(instruction) => {
                    match instruction.name.to_lowercase().as_str() {
                        "cls" => {
                            self.opcodes.push(0x00E0);
                        },
                        "jmp" => {
                            self.jmp(token.line, &instruction.args);
                        },
                        "ret" => self.opcodes.push(0x00EE),
                        "rnd" => self.rnd(token.line, &instruction.args),
                        "sub" => self.sub(token.line, &instruction.args),
                        _ => {
                            self.errors.push(format!("Unknown instruction: {} on line {}", 
                                instruction.name, token.line));
                        }
                    }
                }
            }
        }

        debug!("Opcodes: {:?}", self.opcodes.iter().map(|x| format!("{:X}", x)).collect::<Vec<String>>());

        Ok(())
    }

    fn rnd(&mut self, line: i32, args: &Vec<String>) {
        if args.len() != 2 {
            self.errors.push(
                format!("Error on line {}: Invalid number of arguments for rnd", line));
            return
        }

        let register = match self.registers.get(args[0].as_str()) {
            Some(register) => register,
            None => {
                self.errors.push(
                    format!("Error on line {}: Invalid register {}", line, args[0]));
                return
            }
        };

        let num = args[1].strip_prefix("0x").unwrap_or(args[1].as_str());
        let num = match u8::from_str_radix(num, 16) {
            Ok(num) => num,
            Err(e) => {
                self.errors.push(
                    format!("Error on line {}: Invalid number {}", line, args[1]));
                return
            }
        };


        // let op: u16 = 0xC000;
        let register = register << 8;
        self.opcodes.push(0xC000 | register | num as u16);


    }

    // Given a &str, find the address from label or value
    fn get_address(&self, arg: &str) -> Result<u16, io::Error>  {
        let address = match self.labels.get(arg) {
            Some(address) => *address,
            None => {
                let arg = arg.strip_prefix("0x").unwrap_or(arg);
                    match u16::from_str_radix(arg, 16) {
                        Ok(address) => address,
                        Err(e) => {
                            return Err(io::Error::new(io::ErrorKind::InvalidData,
                            format!("Invalid address {}", arg)))
                        }
                    }
            }
        };

        return Ok(address)
    }

    fn sub(&mut self, line: i32, args: &Vec<String>) {
        match args.len() {
            1 => {
                let arg = args[0].as_str();
                // get address from label or string
                let address = match self.get_address(arg) {
                    Ok(address) => address,
                    Err(err) => {
                        self.errors.push(
                            format!("Error on line {}: {}", line, err));
                        return
                    }
                };

                self.opcodes.push(0x2000 | address);

            }
            _ => {
                self.errors.push(
                    format!("Error on line {}: Invalid number of arguments for sub instruction", 
                        line));
            }
        }
    }

    fn jmp(&mut self, line: i32, args: &Vec<String>) {
        match args.len() {
            // 1 argument is jump to an address (NNN)
            1 => {
                let arg = args[0].as_str();
                // check if arg is a label. If it is, replace with address
                let address = match self.get_address(arg) {
                    Ok(address) => address,
                    Err(err) => {
                        self.errors.push(
                            format!("Error on line {}: {}", line, err));
                        return
                    }
                };

                self.opcodes.push(0x1000 | address);
            }
            // jump to address nnn plus the value in v0
            2 => {

                let reg_arg = args[0].as_str();
                let address_arg = args[1].as_str();

                let reg = match self.registers.get(reg_arg) {
                    Some(reg) => *reg,
                    None => {
                        self.errors.push(
                            format!("Invalid register: {} on line {}", 
                            reg_arg, line));
                        return;
                    }
                };

                if reg != 0 {
                    self.errors.push(
                        format!("Invalid register - register must be v0: {} on line {}", 
                        reg_arg, line));
                    return;
                }

                let address = match self.get_address(address_arg) {
                    Ok(address) => address,
                    Err(err) => {
                        self.errors.push(
                            format!("Error on line {}: {}", line, err));
                        return
                    }
                };
                self.opcodes.push(0xB000 | address);
            }
            _ => {
                self.errors.push(
                    format!("Invalid number of arguments for jmp instruction: line {}", 
                        line));
            }
        }
    }

    fn set_origin(&mut self) -> Result<(), io::Error> {
        let mut org_set = false;
        for token in &self.tokens {
            match &token.token_type {
                TokenType::Origin(address) => {
                    if org_set {
                        return Err(io::Error::new(io::ErrorKind::InvalidData,
                            "Multiple origin directives found"));
                    }
                    self.org = *address;
                    org_set = true;
                    break;
                },
                _ => {}
            }
        }

        // fill to origin
        for i in 0..self.org {
            self.opcodes.push(0);
        }

        Ok(())
    }

    fn print_errors(&self) {
        for error in &self.errors {
            eprintln!("{}", error);
        }
    }

    fn set_labels(&mut self) {
        let mut pc = self.org;
        for token in &self.tokens {
            match &token.token_type {
                TokenType::Label(label) => {
                    self.labels.insert(label.clone(), pc);
                },
                TokenType::Instruction(instruction) => {
                    pc += 2;
                },
                _ => {}
            }
        }
    }



}

#[derive(Clone)]
enum TokenType {
    Label(String),
    Instruction(Instruction),
    Origin(u16),
}

#[derive(Clone)]
struct Token {
    line: i32,
    token_type: TokenType,
}

impl Token {

    fn new(line: i32, token_type: TokenType) -> Token {
        Token {
            line,
            token_type,
        }
    }

    fn to_string(&self) -> String {
        match &self.token_type {
            TokenType::Label(label) => {
                format!("Label: {}", label)
            },
            TokenType::Instruction(instruction) => {
                format!("Instruction: {} -- Args: {}",
                    instruction.name,
                    instruction.args.join(", "))
            },
            TokenType::Origin(address) => {
                format!("Origin: {:X}", address)
            }
        }
    }
}

#[derive(Clone)]
struct Instruction {
    name: String,
    args: Vec<String>,
}


/*

    00E0 (clear screen)
    1NNN (jump)
    6XNN (set register VX)
    7XNN (add value to register VX)
    ANNN (set index register I)
    DXYN (display/draw)

 */