use std::collections::HashMap;

use super::utils::{get_address, address_from_string};
use super::registers::Register;

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

pub fn rnd(args: &Vec<String>) -> Result<u16, String> {
    if args.len() != 2 {
        return Err(
            format!("Invalid number of arguments for rnd: expected 2, got {}",
                args.len()))
    }

    let register = Register::get_register(args[0].as_str())?;

    // TODO: Test that this is not larger than 0xFF
    let address = address_from_string(args[1].as_str())?;
    if address > 0xFF {
        return Err("Invalid address for rnd: expected 0xFF or less".into())
    }

    // let register = register.to_u16() << 8;
    let register = (register as u16) << 8;
    let opcode = 0xC000 | register | address;

    Ok(opcode)
}

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

#[cfg(test)]
mod tests {
    use super::*;

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

}
