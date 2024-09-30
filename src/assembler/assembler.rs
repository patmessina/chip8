use log::{debug, info};

use std::{fs::File, io::{self, BufRead, Write}};

use super::token::{Token, TokenType};
use super::origin::get_origin;
use super::labels::get_labels;
use super::opcodes;

pub fn assemble(source: String, target: String) -> Result<(), String> {


    let tokens = parse_file(source)?;

    for token in &tokens {
        debug!("{}", token.to_string())
    }

    let origin = get_origin(&tokens)?;
    debug!("Origin: 0x{:X}", origin);


    let mut opcodes = new_opcodes(origin)?;
    debug!("Opcodes: {:?}",
        opcodes.iter().map(|x| format!("{:X}", x)).collect::<Vec<String>>());

    let labels = get_labels(&tokens, origin)?;
    for (label, address) in labels.iter() {
        debug!("Label: {} Address: 0x{:X}", label, address);
    }

    
    let mut errors: Vec<String> = Vec::new();

    // parse out 
   for token in tokens {
       match  token.token_type {
           TokenType::Instruction => {

               let opcode_result = match token.name.to_lowercase().as_str() {
                   "add" => opcodes::add(&token.args),
                   "and" => opcodes::and(&token.args),
                   "call" => opcodes::call(&labels, &token.args),
                   "cls" => Ok(0x00E0),
                   "drw" => opcodes::drw(&token.args),
                   "jmp" => opcodes::jmp(&labels, &token.args),
                   "ld" => opcodes::ld(&token.args),
                   "or" => opcodes::or(&token.args),
                   "ret" => Ok(0x00EE),
                   "rnd" => opcodes::rnd(&token.args),
                   "se" => opcodes::se(&token.args),
                   "shl" => opcodes::shl(&token.args),
                   "shr" => opcodes::shr(&token.args),
                   "sknp" => opcodes::sknp(&token.args),
                   "skp" => opcodes::skp(&token.args),
                   "sne" => opcodes::sne(&token.args),
                   "sub" => opcodes::sub(&token.args),
                   "subn" => opcodes::subn(&token.args),
                   "wkp" => opcodes::wkp(&token.args),
                   "xor" => opcodes::xor(&token.args),
                   _ => Err(format!("Unknown instruction {}", token.name)),
               };

               match opcode_result {
                   Ok(opcode) => {
                       opcodes.push(opcode);
                   },
                   Err(e) => {
                       errors.push(format!("Error on line {}: {}", token.line, e));
                   }
               }

           },
           _ => {}
       }
   } 

    debug!("Opcodes: {:?}",
        opcodes.iter().map(|x| format!("{:X}", x)).collect::<Vec<String>>());

    if errors.len() > 0 {
        return Err(errors.join("\n"))
    }

    info!("Writing to file: {}", target);
    save(target, opcodes)?;

    Ok(())
}

fn save(target: String, opcodes: Vec<u16>) -> Result<(), String> {
    let mut file = match File::create(target) {
        Ok(file) => file,
        Err(e) => return Err(format!("Error creating file: {}", e)),
    };

    for opcode in opcodes {
        match file.write_all(&opcode.to_be_bytes()) {
            Ok(_) => {},
            Err(e) => return Err(format!("Error writing to file: {}", e)),
        }
    }

    Ok(())
}

// new_opcodes creates a vector of u16 opcodes with a length of origin
fn new_opcodes(origin: u16) -> Result<Vec<u16>, String> {
    if origin % 2 != 0 {
        return Err("Origin must be even".to_string())
    }
    let num_opcodes = origin / 2;
    let opcodes = vec![0; num_opcodes as usize];
    Ok(opcodes)
}

// Read file into tokens
fn parse_file(source: String) -> Result<Vec<Token>, String> {

    let file = File::open(source).unwrap();
    let reader = io::BufReader::new(file);

    let mut errors: Vec<String> = Vec::new();
    let mut tokens: Vec<Token> = Vec::new();

    for (i, line) in reader.lines().enumerate() {

        let line = match line {
            Ok(line) => line,
            Err(e) => {
                errors.push(format!("Error on line {}: error reading line: {}", i, e));
                continue;
            }
        };

        if errors.len() > 0 {
            return Err(errors.join("\n"))
        }

        let line = line.trim();

        // Ignore empty lines and comments
        if line.is_empty() || line.starts_with("//") {
            continue;
        }


        // split line on whitespace
        let parts: Vec<&str> = line.split_whitespace().collect();


        let mut args: Vec<String> = Vec::new();
        if parts.len() > 1 {
            args = parts[1..].iter().map(|s| s.to_string()).collect();
        }

        let name;
        let token_type: TokenType;

        match parts[0] {
            "org" => {
                name = "org";
                token_type = TokenType::Origin;
            },
            label if parts[0].ends_with(':') => {
                name = label.trim_end_matches(':');
                token_type =  TokenType::Label;
            },
            _ => {
                name = parts[0];
                token_type = TokenType::Instruction;
            }
        }

        let name = name.to_string();
        tokens.push(Token{
            name,
            token_type,
            line: i,
            args,
        });

        if errors.len() > 0 {
            return Err(errors.join("\n"))
        }

    }

    Ok(tokens)
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_opcodes() {
        struct TestCase {
            origin: u16,
            expected: Result<Vec<u16>, String>,
        }

        let test_cases = [
            TestCase {
                origin: 0x200,
                // 0x200 / 2 = 0x100 -- 2 bytes per opcode
                expected: Ok(vec![0; 0x100 as usize]),
            },
            TestCase {
                origin: 0x2,
                expected: Ok(vec![0; 0x1 as usize]),
            },
            TestCase {
                origin: 0x201,
                expected: Err("Origin must be even".to_string()),
            },
        ];

        for case in test_cases.iter() {
            assert_eq!(new_opcodes(case.origin), case.expected);
        }

    }

    // #[test]
    // fn test_parse_file() {
    // }

}
