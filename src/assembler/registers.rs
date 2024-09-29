#[derive(PartialEq, Debug)]
#[derive(Clone,Copy)]
#[repr(u8)]
pub enum Register {
    V0 = 0x0,
    V1 = 0x1,
    V2 = 0x2,
    V3 = 0x3,
    V4 = 0x4,
    V5 = 0x5,
    V6 = 0x6,
    V7 = 0x7,
    V8 = 0x8,
    V9 = 0x9,
    VA = 0xa,
    VB = 0xb,
    VC = 0xc,
    VD = 0xd,
    VE = 0xe,
    VF = 0xf,
}

impl Register {

    pub fn get_register(name: &str) -> Result<Register, String> {
        match name {
            "v0" => Ok(Register::V0),
            "v1" => Ok(Register::V1),
            "v2" => Ok(Register::V2),
            "v3" => Ok(Register::V3),
            "v4" => Ok(Register::V4),
            "v5" => Ok(Register::V5),
            "v6" => Ok(Register::V6),
            "v7" => Ok(Register::V7),
            "v8" => Ok(Register::V8),
            "v9" => Ok(Register::V9),
            "va" => Ok(Register::VA),
            "vb" => Ok(Register::VB),
            "vc" => Ok(Register::VC),
            "vd" => Ok(Register::VD),
            "ve" => Ok(Register::VE),
            "vf" => Ok(Register::VF),
            _ => Err(format!("Invalid register name: {}", name)),
        }
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_register() {

        struct TestCase {
            name: &'static str,
            register: &'static str,
            expected: Result<Register, String>,
        }

        let test_cases = [
            TestCase {
                name: "Valid register",
                register: "v0",
                expected: Ok(Register::V0),
            },
            TestCase {
                name: "Invalid register",
                register: "v16",
                expected: Err("Invalid register name: v16".into()),
            },
        ];

        for test_case in test_cases.iter() {
            let result = Register::get_register(test_case.register);
            assert_eq!(result, test_case.expected, "{}", test_case.name);
        }

    }

}
