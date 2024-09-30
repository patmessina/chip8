use std::collections::HashMap;

// address_from_string converts a string to a u16 address
pub fn address_from_string(s: &str) -> Result<u16, String> {
    let s = s.trim_start_matches("0x");
    let address = match u16::from_str_radix(s, 16) {
        Ok(address) => address,
        Err(e) => {
           return Err(
            format!("Error parsing address {} with error: {}", s, e))
        },
    };

    if address > 0xFFF {
        return Err(
            format!("Error parsing address {}: address out of range", s))
    }

    if address % 2 != 0 {
        return Err(
            format!("Error parsing address {}: address must be even", s))
    }

    Ok(address)
}

pub fn get_address(labels: &HashMap<String, u16>, arg: &str) -> Result<u16, String> {
    let address = match labels.get(arg) {
        Some(address) => *address,
        None => {
            match address_from_string(arg) {
                Ok(address) => address,
                Err(e) => return Err(e),
            }
        }
    };
    Ok(address)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_address() {
        struct TestCase {
            name: &'static str,
            labels: HashMap<String, u16>,
            arg: &'static str,
            expected: Result<u16, String>,
        }

        let test_cases = [
            TestCase {
                name: "Label",
                labels: {
                    let mut labels = HashMap::new();
                    labels.insert("foo".to_string(), 0x200);
                    labels
                },
                arg: "foo",
                expected: Ok(0x200),
            },
            TestCase {
                name: "Address",
                labels: HashMap::new(),
                arg: "0x200",
                expected: Ok(0x200),
            },
            TestCase {
                name: "Invalid address",
                labels: HashMap::new(),
                arg: "0xFFFF",
                expected: Err("Error parsing address FFFF: address out of range".into()),
            },
            TestCase {
                name: "Invalid address",
                labels: HashMap::new(),
                arg: "v0",
                expected: Err("Error parsing address v0 with error: invalid digit found in string".into()),
            },
            TestCase {
                name: "Empty address",
                labels: HashMap::new(),
                arg: "0x",
                expected: Err("Error parsing address  with error: cannot parse integer from empty string".into()),
            },
        ];

        for case in test_cases.iter() {
            let result = get_address(&case.labels, case.arg);
            assert_eq!(result, case.expected, "Failed on case: {}", case.name);
        }
    }

    #[test]
    fn test_address_from_string() {

        struct TestCase {
            address: &'static str,
            expected: Result<u16, String>,
        }

        let test_cases = [
            TestCase {
                address: "0x200",
                expected: Ok(0x200),
            },
            TestCase {
                address: "201",
                expected: Err("Error parsing address 201: address must be even".into()),
            },
            TestCase {
                address: "0xFFFF",
                expected: Err("Error parsing address FFFF: address out of range".into()),
            },
            TestCase {
                address: "v0",
                expected: Err("Error parsing address v0 with error: invalid digit found in string".into()),
            },
            TestCase{
                address: "0x",
                expected: Err("Error parsing address  with error: cannot parse integer from empty string".into()),
            },
        ];

        for case in test_cases.iter() {
            let result = address_from_string(case.address);
            assert_eq!(result, case.expected, "Failed on address: {}", case.address);
        }
    }



}
