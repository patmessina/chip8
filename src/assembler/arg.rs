use super::registers::Register;

#[derive(PartialEq, Debug)]
#[derive(Clone,Copy)]
pub enum ArgType {
    BCD(&'static str),
    Font(&'static str),
    IndexRegister(&'static str),
    Register(Register),
    Number(u16),
    SoundTimer(&'static str),
    DelayTimer(&'static str),
}

impl ArgType {

    pub fn new(arg: &str) -> Result<ArgType, String> {

        let arg = arg.to_lowercase();

        let arg_type = match arg.as_str() {
            "b" => Ok(ArgType::BCD("b")),
            "i" => Ok(ArgType::IndexRegister("i")),
            "f" => Ok(ArgType::Font("f")),
            "dt" => Ok(ArgType::DelayTimer("dt")),
            "st" => Ok(ArgType::SoundTimer("st")),
            arg if arg.starts_with("v") => {
                match Register::get_register(&arg) {
                    Ok(r) => Ok(ArgType::Register(r)),
                    Err(e) => Err(e),
                }
            },
            _ => {
                let num = arg.trim_start_matches("0x");
                let number = match u16::from_str_radix(num, 16) {
                    Ok(n) => n,
                    Err(e) => return Err(format!("Error parsing number: {}", e)),
                };

                if number > 0xFFF {
                    return Err(format!("Number out of range: 0x{:X}", number));
                }
                Ok(ArgType::Number(number))
            },
        };

        arg_type
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        struct TestCase {
            name: &'static str,
            arg: &'static str,
            expected: Result<ArgType, String>,
        }

        let test_cases = [
            TestCase {
                name: "BCD",
                arg: "b",
                expected: Ok(ArgType::BCD("b")),
            },
            TestCase {
                name: "Index register",
                arg: "i",
                expected: Ok(ArgType::IndexRegister("i")),
            },
            TestCase {
                name: "Font",
                arg: "f",
                expected: Ok(ArgType::Font("f")),
            },
            TestCase {
                name: "Delay timer",
                arg: "dt",
                expected: Ok(ArgType::DelayTimer("dt")),
            },
            TestCase {
                name: "Sound timer",
                arg: "st",
                expected: Ok(ArgType::SoundTimer("st")),
            },
            TestCase {
                name: "Register",
                arg: "v0",
                expected: Ok(ArgType::Register(Register::V0)),
            },
            TestCase {
                name: "Number",
                arg: "0x200",
                expected: Ok(ArgType::Number(0x200)),
            },
            TestCase {
                name: "Invalid number",
                arg: "0x1000",
                expected: Err("Number out of range: 0x1000".into()),
            },
            TestCase {
                name: "Invalid register",
                arg: "v16",
                expected: Err("Invalid register name: v16".into()),
            },
        ];

        for test_case in test_cases.iter() {
            let result = ArgType::new(test_case.arg);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }
    }
}

