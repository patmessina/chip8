use std::collections::HashMap;

use super::utils::get_address;
use super::registers::Register;
use super::arg::ArgType;

// add
//
// fx1e - set I = I + vx
// 8xy4 - set vx = vx + vy and set vf = carry
// 7xnn - set vx = vx + nn vf not changed
pub fn add(args: &Vec<String>) -> Result<u16, String> {

    if args.len() != 2 {
        return Err(
            format!(
                "Invalid number of arguments for add: expected 2, got {}",
                args.len()))
    }

    let first_arg = ArgType::new(args[0].as_str())?;
    let second_arg = ArgType::new(args[1].as_str())?;

    let result = match first_arg {
        // if index register add to index
        ArgType::IndexRegister(_) => add_to_index(second_arg),
        // if register add another register or the byte that is expected
        ArgType::Register(first_reg) => add_vy_or_address(first_reg, second_arg),
        // otherwise who knows
        _ => {
            Err("Invalid argument for add: expected register in first argument".into())
        }
    };

    result
}

// add_vy_or_address
// add another reigster or add a byte to reg_x
fn add_vy_or_address(reg_x: Register, arg: ArgType) -> Result<u16, String> {

    let opcode = match arg {
        ArgType::Register(reg_y) => {
            0x8004 | (reg_x as u16) << 8 | (reg_y as u16) << 4
        },
        ArgType::Number(byte) => {
            if byte > 0xFF {
                return Err("Invalid address for add: expected 0xFF or less".into())
            }
            0x7000 | (reg_x as u16) << 8 | byte
        },
        _ => {
            return Err("Invalid argument for add: expected register in second argument".into())
        }
    };

    Ok(opcode)
}

// add_to_index
fn add_to_index(arg: ArgType) -> Result<u16, String> {
    let opcode = match arg {
        ArgType::Register(reg) => {
            0xF01E | (reg as u16) << 8
        },
        _ => {
            return Err("Invalid argument for add: expected register in second argument".into())
        }
    };
 
    Ok(opcode)
}

// 8xy2
//
// and v0 v1 binary and
pub fn and(args: &Vec<String>) -> Result<u16, String> {
    let opcode: u16;

    if args.len() != 2 {
        return Err(
            format!(
                "Invalid number of arguments for and: expected 2, got {}",
                args.len()))
    }

    let reg_x = Register::get_register(args[0].as_str())?;
    let reg_y = Register::get_register(args[1].as_str())?;

    opcode = 0x8002 | (reg_x as u16) << 8 | (reg_y as u16) << 4;

    Ok(opcode)
}

// DXYN
//
// draw v0 v1 0x1
pub fn drw(args: &Vec<String>) -> Result<u16, String> {
    let opcode: u16;
    if args.len() != 3 {
        return Err(
            format!(
                "Invalid number of arguments for drw: expected 3, got {}",
                args.len()))
    }

    let reg_x = Register::get_register(args[0].as_str())?;
    let reg_y = Register::get_register(args[1].as_str())?;

    let nibble = args[2].as_str().trim_start_matches("0x");
    let nibble = match u8::from_str_radix(nibble, 16) {
        Ok(nibble) => nibble,
        Err(_) => {
            return Err(
                format!(
                    "Invalid nibble for drw: expected hex value, got {}",
                    args[2].as_str()
            ))
        }
    };

    if nibble > 0xF {
        return Err(
            format!(
                "Invalid nibble for drw: expected 0xF or less, got {}",
                nibble
            ))
    }

    opcode = 0xD000 | (reg_x as u16) << 8 | (reg_y as u16) << 4 | nibble as u16;

    Ok(opcode)
}

// ld
//
//
pub fn ld(args: &Vec<String>) -> Result<u16, String> {

    if args.len() != 2 {
        return Err(
            format!(
                "Invalid number of arguments for ld: expected 2, got {}",
                args.len()))
    }


    let first_arg = ArgType::new(args[0].as_str())?;
    let second_arg = ArgType::new(args[1].as_str())?;

    let result = match first_arg {
        ArgType::IndexRegister(_) => ld_index(second_arg),
        ArgType::SoundTimer(_) => ld_st(second_arg),
        ArgType::DelayTimer(_) => ld_dt(second_arg),
        ArgType::Font(_) => ld_font(second_arg),
        ArgType::BCD(_) => ld_bcd(second_arg),
        ArgType::Register(reg_x) => ld_register(reg_x, second_arg),
        _ => {
            Err("Invalid first argument for ld".into())
        }
    };

    result
}

fn ld_register(reg_x: Register, arg: ArgType) -> Result<u16, String> {
    let result = match arg {
        ArgType::DelayTimer(_) => {
            0xF007 | (reg_x as u16) << 8
        },
        ArgType::IndexRegister(_) => {
            0xF065 | (reg_x as u16) << 8
        },
        ArgType::Register(reg_y) => {
            0x8000 | (reg_x as u16) << 8 | (reg_y as u16) << 4
        },
        ArgType::Number(num) => {
            if num > 0xFF {
                return Err(
                    "Invalid second argument for ld: expected 0xFF or less".into())
            }
            0x6000 | (reg_x as u16) << 8 | num
        }
        _ => {
            return Err("Invalid second argument for ld".into())
        }
    };

    Ok(result)
}

fn ld_bcd(arg: ArgType) -> Result<u16, String> {
    let result = match arg {
        ArgType::Register(reg) => {
            0xF033 | (reg as u16) << 8
        },
        _ => {
            return Err("Invalid second argument for ld".into())
        }
    };

    Ok(result)
}

fn ld_font(arg: ArgType) -> Result<u16, String> {
    let result = match arg {
        ArgType::Register(reg) => {
            0xF029 | (reg as u16) << 8
        },
        _ => {
            return Err("Invalid second argument for ld".into())
        }
    };

    Ok(result)
}

fn ld_index(arg: ArgType) -> Result<u16, String> {
    let result = match arg {
        ArgType::Register(reg) => {
            0xF055 | (reg as u16) << 8
        },
        ArgType::Number(num) => {
            if num > 0xFFF {
                return Err("Invalid address for ld: expected 0xFFF or less".into())
            }
            0xA000 | num
        },
        _ => {
            return Err("Invalid argument for ld".into())
        }
    };

    Ok(result)
}

fn ld_dt(arg: ArgType) -> Result<u16, String> {
    let result = match arg {
        ArgType::Register(reg) => {
            0xF015 | (reg as u16) << 8
        },
        _ => {
            return Err("Invalid second argument for ld".into())
        }
    };

    Ok(result)
}

// ld sound timer
fn ld_st(arg: ArgType) -> Result<u16, String> {
    let result = match arg {
        ArgType::Register(reg) => {
            0xF018 | (reg as u16) << 8
        },
        _ => {
            return Err("Invalid second argument for ld".into())
        }
    };

    Ok(result)
}


// jmp
//
// jmp 0x200
// jmp to memory address from label or address
//
// jmp v0 0x200
// if v0 is used, jump to address + v0
pub fn jmp(labels: &HashMap<String, u16>, args: &Vec<String>) -> Result<u16, String> {

    let opcode: u16;
    match args.len() {
        1 => {
            let arg = args[0].as_str();
            let address = get_address(labels, arg)?;
            opcode = 0x1000 | address;
        },
        2 => {
            let register = Register::get_register(args[0].as_str())?;
            if register != Register::V0 {
                return Err("Invalid register for jmp: expected V0".into())
            }

            let address = get_address(labels, args[1].as_str())?;

            opcode = 0xB000 | address;
        },
        _ => {
            return Err(
                format!("Invalid number of arguments for jmp: expected 1 or 2, got {}",
                    args.len()))
        }
    }

    Ok(opcode)
}

// rnd
//
// rnd v0 0x10
// set v0 to random number & 0x10
pub fn rnd(args: &Vec<String>) -> Result<u16, String> {
    if args.len() != 2 {
        return Err(
            format!("Invalid number of arguments for rnd: expected 2, got {}",
                args.len()))
    }

    let first_arg = ArgType::new(args[0].as_str())?;
    let second_arg = ArgType::new(args[1].as_str())?;

    let reg_x = match first_arg {
        ArgType::Register(reg) => reg,
        _ => {
            return Err("Invalid first argument for rnd: expected register".into())
        }
    };

    let num = match second_arg {
        ArgType::Number(num) => num,
        _ => {
            return Err("Invalid second argument for rnd: expected number".into())
        }
    };

    if num > 0xFF {
        return Err("Invalid address for rnd: expected 0xFF or less".into())
    }

    // let register = register.to_u16() << 8;
    let register = (reg_x as u16) << 8;
    let opcode = 0xC000 | register | num;

    Ok(opcode)
}

// call
//
// call 0x200
// call to memory address from label or address
pub fn call(labels: &HashMap<String, u16>, args: &Vec<String>) -> Result<u16, String> {
    if args.len() != 1 {
        return Err(
            format!("Invalid number of arguments for call: expected 1, got {}",
                args.len()))
    }

    let address = get_address(labels, args[0].as_str())?;

    let opcode = 0x2000 | address;
    Ok(opcode)
}

// or
//
// 8xy1
//
// or v0 v1 binary or
pub fn or(args: &Vec<String>) -> Result<u16, String> {
    let opcode: u16;

    if args.len() != 2 {
        return Err(
            format!("Invalid number of arguments for or: expected 2, got {}",
                args.len()))
    }

    let reg_x = Register::get_register(args[0].as_str())?;
    let reg_y = Register::get_register(args[1].as_str())?;

    opcode = 0x8001 | (reg_x as u16) << 8 | (reg_y as u16) << 4;

    Ok(opcode)
}

// skp
//
// ex9e
//
// skip if key is pressed  from vx
pub fn skp(args: &Vec<String>) -> Result<u16, String> {
    let opcode: u16;

    let reg_x = Register::get_register(args[0].as_str())?;

    opcode = 0xE09E | (reg_x as u16) << 8;

    Ok(opcode)
}

// sknp
//
// exa1 
//
// skip if key is not pressed from vx
pub fn sknp(args: &Vec<String>) -> Result<u16, String> {
    let opcode: u16;

    let reg_x = Register::get_register(args[0].as_str())?;

    opcode = 0xE0A1 | (reg_x as u16) << 8;
    Ok(opcode)
}

// se
// skip if equal
pub fn se(args: &Vec<String>) -> Result<u16, String> {

    if args.len() != 2 {
        return Err(
            format!("Invalid number of arguments for se: expected 2, got {}",
                args.len()))
    }

    let first_arg = ArgType::new(args[0].as_str())?;
    let second_arg = ArgType::new(args[1].as_str())?;

    let reg_x = match first_arg {
        ArgType::Register(reg) => reg,
        _ => {
            return Err("Invalid first argument for se: expected register".into())
        }
    };

    let opcode = match second_arg {
        ArgType::Register(reg) => {
            //5XY0
            //skip if vx and vy are equal
            0x5000 | (reg_x as u16) << 8 | (reg as u16) << 4
        },
        ArgType::Number(num) => {
            // 3xnn
            // skip if nn is equal to vx
            if num > 0xFF {
                return Err(
                    "Invalid address for se: expected 0xFF or less".into())
            }   
            0x3000 | (reg_x as u16) << 8 | num
        },
        _ => {
            return Err("Invalid second argument for se".into())
        }
    };

    Ok(opcode)
}

// shl
// 
// 8xye
pub fn shl(args: &Vec<String>) -> Result<u16, String> {
    let opcode: u16;

    if args.len() != 2 {
        return Err(
            format!("Invalid number of arguments for shl: expected 2, got {}",
                args.len()))
    }

    let reg_x = Register::get_register(args[0].as_str())?;
    let reg_y = Register::get_register(args[1].as_str())?;

    opcode = 0x800E | (reg_x as u16) << 8 | (reg_y as u16) << 4;

    Ok(opcode)

}

// shr
//
// 8xy6
pub fn shr(args: &Vec<String>) -> Result<u16, String> {
    let opcode: u16;

    if args.len() != 2 {
        return Err(
            format!("Invalid number of arguments for shr: expected 2, got {}",
                args.len()))
    }

    let reg_x = Register::get_register(args[0].as_str())?;
    let reg_y = Register::get_register(args[1].as_str())?;

    opcode = 0x8006 | (reg_x as u16) << 8 | (reg_y as u16) << 4;

    Ok(opcode)

}


// skip if not equal
pub fn sne(args: &Vec<String>) -> Result<u16, String> {

    if args.len() != 2 {
        return Err(
            format!("Invalid number of arguments for sne: expected 2, got {}",
                args.len()))
    }

    let first_arg = ArgType::new(args[0].as_str())?;
    let second_arg = ArgType::new(args[1].as_str())?;

    let reg_x = match first_arg {
        ArgType::Register(reg) => reg,
        _ => {
            return Err("Invalid first argument for sne: expected register".into())
        }
    };

    let opcode = match second_arg {
        ArgType::Register(reg) => {
            //9XY0
            //skip if vx and vy are not equal
            0x9000 | (reg_x as u16) << 8 | (reg as u16) << 4
        },
        ArgType::Number(num) => {
            // 4XNN
            // skip if nn is not equal to vx
            if num > 0xFF {
                return Err(
                    "Invalid address for sne: expected 0xFF or less".into())
            }
            0x4000 | (reg_x as u16) << 8 | num
        },
        _ => {
            return Err("Invalid second argument for sne".into())
        }
    };

    Ok(opcode)
}

// sub
//
// 8xy5
pub fn sub(args: &Vec<String>) -> Result<u16, String> {

    let opcode: u16;

    if args.len() != 2 {
        return Err(
            format!("Invalid number of arguments for sub: expected 2, got {}",
                args.len()))
    }

    let reg_x = Register::get_register(args[0].as_str())?;
    let reg_y = Register::get_register(args[1].as_str())?;

    opcode = 0x8005 | (reg_x as u16) << 8 | (reg_y as u16) << 4;

    Ok(opcode)

}

// subn
//
// 8xy7
pub fn subn(args: &Vec<String>) -> Result<u16, String> {
    let opcode: u16;

    if args.len() != 2 {
        return Err(
            format!("Invalid number of arguments for subn: expected 2, got {}",
                args.len()))
    }

    let reg_x = Register::get_register(args[0].as_str())?;
    let reg_y = Register::get_register(args[1].as_str())?;

    opcode = 0x8007 | (reg_x as u16) << 8 | (reg_y as u16) << 4;

    Ok(opcode)
}

// FX0A
//
// Wait for a key press, store the value of the key in VX
pub fn wkp(args: &Vec<String>) -> Result<u16, String> {
    let opcode: u16;
    if args.len() != 1 {
        return Err(
            format!("Invalid number of arguments for wkp: expected 1, got {}",
                args.len()))
    }

    let reg = Register::get_register(args[0].as_str())?;

    opcode = 0xF00A | (reg as u16) << 8;
    Ok(opcode)
}

// 8xy3
//
// xor v0 v1 binary xor
pub fn xor(args: &Vec<String>) -> Result<u16, String> {
    let opcode: u16;

    if args.len() != 2 {
        return Err(
            format!("Invalid number of arguments for xor: expected 2, got {}",
                args.len()))
    }

    let reg_x = Register::get_register(args[0].as_str())?;
    let reg_y = Register::get_register(args[1].as_str())?;

    opcode = 0x8003 | (reg_x as u16) << 8 | (reg_y as u16) << 4;

    Ok(opcode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        struct TestCase {
            name: &'static str,
            args: Vec<String>,
            expected: Result<u16, String>,
        }
        let test_cases = [
            TestCase {
                name: "Valid register and register - v0 and v1",
                args: vec!["v0".to_string(), "v1".to_string()],
                expected: Ok(0x8014),
            },
            TestCase {
                name: "Valid register and number - v0 and 0x10",
                args: vec!["v0".to_string(), "0x10".to_string()],
                expected: Ok(0x7010),
            },
            TestCase {
                name: "Valid index register and register - i and v1",
                args: vec!["i".to_string(), "v1".to_string()],
                expected: Ok(0xF11E),
            },
            TestCase {
                name: "Invalid number of arguments",
                args: vec!["v0".to_string()],
                expected: Err("Invalid number of arguments for add: expected 2, got 1".into()),
            },
            TestCase {
                name: "Invalid register",
                args: vec!["v1".to_string(), "v16".to_string()],
                expected: Err("Invalid register name: v16".into()),
            },
            TestCase {
                name: "Invalid number",
                args: vec!["v0".to_string(), "0xFFF".to_string()],
                expected: Err("Invalid address for add: expected 0xFF or less".into()),
            },
        ];

        for test_case in test_cases.iter() {
            let result = add(&test_case.args);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }
    }

    #[test]
    fn test_and() {
        struct TestCase {
            name: &'static str,
            args: Vec<String>,
            expected: Result<u16, String>,
        }

        let test_cases = [
            TestCase {
                name: "Valid registers - v0 and v1",
                args: vec!["v0".to_string(), "v1".to_string()],
                expected: Ok(0x8012),
            },
            TestCase {
                name: "Valid registers - va and vf",
                args: vec!["va".to_string(), "vf".to_string()],
                expected: Ok(0x8AF2),
            },
        ];

        for test_case in test_cases.iter() {
            let result = and(&test_case.args);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }


    }

    #[test]
    fn test_drw() {
        struct TestCase {
            name: &'static str,
            args: Vec<String>,
            expected: Result<u16, String>,
        }
        let test_cases = [
            TestCase {
                name: "Valid arguments",
                args: vec!["v0".to_string(), "v1".to_string(), "0x1".to_string()],
                expected: Ok(0xD011),
            },
            TestCase {
                name: "Invalid number of arguments",
                args: vec!["v0".to_string(), "v1".to_string()],
                expected: Err("Invalid number of arguments for drw: expected 3, got 2".into()),
            },
            TestCase {
                name: "Invalid nibble",
                args: vec!["v0".to_string(), "v1".to_string(), "0x10".to_string()],
                expected: Err("Invalid nibble for drw: expected 0xF or less, got 16".into()),
            },
        ];

        for test_case in test_cases.iter() {
            let result = drw(&test_case.args);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }
    }

    #[test]
    fn test_ld_register() {
        struct TestCase {
            name: &'static str,
            args: (Register, ArgType),
            expected: Result<u16, String>,
        }

        let test_cases = [
            TestCase {
                name: "Valid register and register - v0 and v1",
                args: (Register::V0, ArgType::Register(Register::V1)),
                expected: Ok(0x8010),
            },
            TestCase {
                name: "Valid register and number - v0 and 0x10",
                args: (Register::V0, ArgType::Number(0x10)),
                expected: Ok(0x6010),
            },
            TestCase {
                name: "Valid register and index register - v0 and i",
                args: (Register::V0, ArgType::IndexRegister("i".into())),
                expected: Ok(0xF065),
            },
            TestCase {
                name: "Invalid second argument",
                args: (Register::V0, ArgType::Font("f".into())),
                expected: Err("Invalid second argument for ld".into()),
            },
            TestCase {
                name: "Invalid number",
                args: (Register::V0, ArgType::Number(0x1000)),
                expected: Err("Invalid second argument for ld: expected 0xFF or less".into()),
            },
        ];

        for test_case in test_cases.iter() {
            let result = ld_register(test_case.args.0, test_case.args.1);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }

    }

    #[test]
    fn test_ld_font() {
        struct TestCase {
            name: &'static str,
            args: ArgType,
            expected: Result<u16, String>,
        }

        let test_cases = [
            TestCase {
                name: "Valid register",
                args: ArgType::Register(Register::V0),
                expected: Ok(0xF029),
            },
            TestCase {
                name: "Invalid argument",
                args: ArgType::Number(0x200),
                expected: Err("Invalid second argument for ld".into()),
            },
        ];

        for test_case in test_cases.iter() {
            let result = ld_font(test_case.args);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }
    }

    #[test]
    fn test_ld_bcd() {
        struct TestCase {
            name: &'static str,
            args: ArgType,
            expected: Result<u16, String>,
        }
        let test_cases = [
            TestCase {
                name: "Valid register",
                args: ArgType::Register(Register::V0),
                expected: Ok(0xF033),
            },
            TestCase {
                name: "Invalid argument",
                args: ArgType::Number(0x200),
                expected: Err("Invalid second argument for ld".into()),
            },
        ];

        for test_case in test_cases.iter() {
            let result = ld_bcd(test_case.args);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }
    }

    #[test]
    fn test_ld_index() {
        struct TestCase {
            name: &'static str,
            args: ArgType,
            expected: Result<u16, String>,
        }

        let test_cases = [
            TestCase {
                name: "Valid register",
                args: ArgType::Register(Register::V0),
                expected: Ok(0xF055),
            },
            TestCase {
                name: "Valid number",
                args: ArgType::Number(0x200),
                expected: Ok(0xA200),
            },
            TestCase {
                name: "Invalid argument",
                args: ArgType::IndexRegister("i".into()),
                expected: Err("Invalid argument for ld".into()),
            },
        ];

        for test_case in test_cases.iter() {
            let result = ld_index(test_case.args);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }
    }

    #[test]
    fn test_jmp() {
        struct TestCase {
            name: &'static str,
            labels: HashMap<String, u16>,
            args: Vec<String>,
            expected: Result<u16, String>,
        }

        let test_cases = [
            TestCase {
                name: "Valid address",
                labels: HashMap::new(),
                args: vec!["0x200".to_string()],
                expected: Ok(0x1200),
            },
            TestCase {
                name: "Valid register",
                labels: HashMap::new(),
                args: vec!["v0".to_string(), "0x200".to_string()],
                expected: Ok(0xB200),
            },
            TestCase {
                name: "Invalid register",
                labels: HashMap::new(),
                args: vec!["v1".to_string(), "0x200".to_string()],
                expected: Err("Invalid register for jmp: expected V0".into()),
            },
            TestCase {
                name: "Valid Label",
                labels: HashMap::from([
                        ("foo".to_string(), 0x2ff)
                    ]),
                args: vec![ "foo".to_string() ],
                expected: Ok(0x12ff),
            },
            TestCase {
                name: "Invalid number of arguments",
                labels: HashMap::new(),
                args: vec!["v0".to_string(), "0x200".to_string(), "0x200".to_string()],
                expected: Err("Invalid number of arguments for jmp: expected 1 or 2, got 3".into()),
            },
        ];

        for test_case in test_cases.iter() {
            let result = jmp(&test_case.labels, &test_case.args);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }

    }

    #[test]
    fn test_or() {
        struct TestCase {
            name: &'static str,
            args: Vec<String>,
            expected: Result<u16, String>,
        }

        let test_cases = [
            TestCase {
                name: "Valid registers - v0 and v1",
                args: vec!["v0".to_string(), "v1".to_string()],
                expected: Ok(0x8011),
            },
            TestCase {
                name: "Valid registers - va and vf",
                args: vec!["va".to_string(), "vf".to_string()],
                expected: Ok(0x8AF1),
            },
        ];

        for test_case in test_cases.iter() {
            let result = or(&test_case.args);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }
    }


    #[test]
    fn test_rnd() {
        struct TestCase {
            name: &'static str,
            args: Vec<String>,
            expected: Result<u16, String>,
        }

        let test_cases = [
            TestCase {
                name: "Valid arguments",
                args: vec!["v0".to_string(), "0x10".to_string()],
                expected: Ok(0xC010),
            },
            TestCase {
                name: "Invalid number of arguments",
                args: vec!["v0".to_string()],
                expected: Err("Invalid number of arguments for rnd: expected 2, got 1".into()),
            },
            TestCase {
                name: "Invalid address",
                args: vec!["v0".to_string(), "0x100".to_string()],
                expected: Err("Invalid address for rnd: expected 0xFF or less".into()),
            },
        ];

        for test_case in test_cases.iter() {
            let result = rnd(&test_case.args);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }
    }


    #[test]
    fn test_call() {
        struct TestCase {
            name: &'static str,
            labels: HashMap<String, u16>,
            args: Vec<String>,
            expected: Result<u16, String>,
        }

        let test_cases = [
            TestCase {
                name: "Valid address",
                labels: HashMap::new(),
                args: vec!["0x200".to_string()],
                expected: Ok(0x2200),
            },
            TestCase {
                name: "Invalid number of arguments",
                labels: HashMap::new(),
                args: vec!["0x200".to_string(), "0x200".to_string()],
                expected: Err("Invalid number of arguments for call: expected 1, got 2".into()),
            },
            TestCase {
                name: "Valid Label",
                labels: HashMap::from([
                        ("foo".to_string(), 0x2ff)
                    ]),
                args: vec![ "foo".to_string() ],
                expected: Ok(0x22ff),
            },
        ];

        for test_case in test_cases.iter() {
            let result = call(&test_case.labels, &test_case.args);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }
    }

    #[test]
    fn test_shl() {
        struct TestCase {
            name: &'static str,
            args: Vec<String>,
            expected: Result<u16, String>,
        }

        let test_cases = [
            TestCase {
                name: "Valid registers - v0 and v1",
                args: vec!["v0".to_string(), "v1".to_string()],
                expected: Ok(0x801E),
            },
            TestCase {
                name: "Valid registers - va and vf",
                args: vec!["va".to_string(), "vf".to_string()],
                expected: Ok(0x8AFE),
            },
            TestCase {
                name: "Invalid number of arguments",
                args: vec!["v0".to_string()],
                expected: Err("Invalid number of arguments for shl: expected 2, got 1".into()),
            },
        ];

        for test_case in test_cases.iter() {
            let result = shl(&test_case.args);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }
    }

    #[test]
    fn test_shr() {
        struct TestCase {
            name: &'static str,
            args: Vec<String>,
            expected: Result<u16, String>,
        }

        let test_cases = [
            TestCase {
                name: "Valid registers - v0 and v1",
                args: vec!["v0".to_string(), "v1".to_string()],
                expected: Ok(0x8016),
            },
            TestCase {
                name: "Valid registers - va and vf",
                args: vec!["va".to_string(), "vf".to_string()],
                expected: Ok(0x8AF6),
            },
            TestCase {
                name: "Invalid number of arguments",
                args: vec!["v0".to_string()],
                expected: Err("Invalid number of arguments for shr: expected 2, got 1".into()),
            },
        ];


        for test_case in test_cases.iter() {
            let result = shr(&test_case.args);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }
    }

    #[test]
    fn test_se() {
        struct TestCase {
            name: &'static str,
            args: Vec<String>,
            expected: Result<u16, String>,
        }

        let test_cases = [
            TestCase {
                name: "Valid registers - v0 and v1",
                args: vec!["v0".to_string(), "v1".to_string()],
                expected: Ok(0x5010),
            },
            TestCase {
                name: "Valid registers - va and vf",
                args: vec!["va".to_string(), "vf".to_string()],
                expected: Ok(0x5af0),
            },
            TestCase {
                name: "Valid register and address - v0 and 0x10",
                args: vec!["v0".to_string(), "0x10".to_string()],
                expected: Ok(0x3010),
            },
            TestCase {
                name: "Invalid number of arguments",
                args: vec!["v0".to_string()],
                expected: Err("Invalid number of arguments for se: expected 2, got 1".into()),
            },
            TestCase {
                name: "Invalid address - 0x100",
                args: vec!["v0".to_string(), "0x100".to_string()],
                expected: Err("Invalid address for se: expected 0xFF or less".into()),
            },
        ];

        for test_case in test_cases.iter() {
            let result = se(&test_case.args);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }
    }

    #[test]
    fn test_sne() {
        struct TestCase {
            name: &'static str,
            args: Vec<String>,
            expected: Result<u16, String>,
        }
        let test_cases = [
            TestCase {
                name: "Valid registers - v0 and v1",
                args: vec!["v0".to_string(), "v1".to_string()],
                expected: Ok(0x9010),
            },
            TestCase {
                name: "Valid registers - va and vf",
                args: vec!["va".to_string(), "vf".to_string()],
                expected: Ok(0x9af0),
            },
            TestCase {
                name: "Valid register and address - v0 and 0x10",
                args: vec!["v0".to_string(), "0x10".to_string()],
                expected: Ok(0x4010),
            },
            TestCase {
                name: "Invalid number of arguments",
                args: vec!["v0".to_string()],
                expected: Err("Invalid number of arguments for sne: expected 2, got 1".into()),
            },
            TestCase {
                name: "Invalid address - 0x100",
                args: vec!["v0".to_string(), "0x100".to_string()],
                expected: Err("Invalid address for sne: expected 0xFF or less".into()),
            },
        ];

        for test_case in test_cases.iter() {
            let result = sne(&test_case.args);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }
    }

    #[test]
    fn test_skp() {
        struct TestCase {
            name: &'static str,
            args: Vec<String>,
            expected: Result<u16, String>,
        }
        let test_cases = [  
            TestCase {
                name: "Valid register - v0",
                args: vec!["v0".to_string()],
                expected: Ok(0xE09E),
            },
            TestCase {
                name: "Valid register - vf",
                args: vec!["vf".to_string()],
                expected: Ok(0xEF9E),
            },
        ];

        for test_case in test_cases.iter() {
            let result = skp(&test_case.args);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }
    }

    #[test]
    fn test_sknp() {
        struct TestCase {
            name: &'static str,
            args: Vec<String>,
            expected: Result<u16, String>,
        }
        let test_cases = [
            TestCase {
                name: "Valid register - v0",
                args: vec!["v0".to_string()],
                expected: Ok(0xE0A1),
            },
            TestCase {
                name: "Valid register - vf",
                args: vec!["vf".to_string()],
                expected: Ok(0xEFA1),
            },
        ];

        for test_case in test_cases.iter() {
            let result = sknp(&test_case.args);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }
    }

    #[test]
    fn test_sub() {
        struct TestCase {
            name: &'static str,
            args: Vec<String>,
            expected: Result<u16, String>,
        }

        let test_cases = [
            TestCase {
                name: "Valid registers - v0 and v1",
                args: vec!["v0".to_string(), "v1".to_string()],
                expected: Ok(0x8015),
            },
            TestCase {
                name: "Valid registers - va and vf",
                args: vec!["va".to_string(), "vf".to_string()],
                expected: Ok(0x8AF5),
            },
            TestCase {
                name: "Invalid number of arguments",
                args: vec!["v0".to_string()],
                expected: Err("Invalid number of arguments for sub: expected 2, got 1".into()),
            },
        ];

        for test_case in test_cases.iter() {
            let result = sub(&test_case.args);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }
    }

    #[test]
    fn test_subn() {
        struct TestCase {
            name: &'static str,
            args: Vec<String>,
            expected: Result<u16, String>,
        }

        let test_cases = [
            TestCase {
                name: "Valid registers - v0 and v1",
                args: vec!["v0".to_string(), "v1".to_string()],
                expected: Ok(0x8017),
            },
            TestCase {
                name: "Valid registers - va and vf",
                args: vec!["va".to_string(), "vf".to_string()],
                expected: Ok(0x8AF7),
            },
            TestCase {
                name: "Invalid number of arguments",
                args: vec!["v0".to_string()],
                expected: Err("Invalid number of arguments for subn: expected 2, got 1".into()),
            },
        ];

        for test_case in test_cases.iter() {
            let result = subn(&test_case.args);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }
    }



    #[test]
    fn test_wkp() {
        struct TestCase {
            name: &'static str,
            args: Vec<String>,
            expected: Result<u16, String>,
        }
        let test_cases = [
            TestCase {
                name: "Valid register - v0",
                args: vec!["v0".to_string()],
                expected: Ok(0xF00A),
            },
            TestCase {
                name: "Valid register - v5",
                args: vec!["v5".to_string()],
                expected: Ok(0xF50A),
            },
            TestCase {
                name: "Invalid number of arguments",
                args: vec![],
                expected: Err("Invalid number of arguments for wkp: expected 1, got 0".into()),
            },
        ];

        for test_case in test_cases.iter() {
            let result = wkp(&test_case.args);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }
    }

    #[test]
    fn test_xor() {
        struct TestCase {
            name: &'static str,
            args: Vec<String>,
            expected: Result<u16, String>,
        }

        let test_cases = [
            TestCase {
                name: "Valid registers - v0 and v1",
                args: vec!["v0".to_string(), "v1".to_string()],
                expected: Ok(0x8013),
            },
            TestCase {
                name: "Valid registers - va and vf",
                args: vec!["va".to_string(), "vf".to_string()],
                expected: Ok(0x8AF3),
            },
        ];

        for test_case in test_cases.iter() {
            let result = xor(&test_case.args);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }
    }



}
