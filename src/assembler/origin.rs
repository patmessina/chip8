use super::utils::address_from_string;
use super::token::{Token, TokenType};

pub fn get_origin(tokens: &Vec<Token>) -> Result<u16, String> {
    let mut org_set = false;
    let mut origin: u16 = 0x200;
    for token in tokens {
        match token.token_type {
            TokenType::Origin => {
                // Make sure we do not have duplicate org settings
                if org_set {
                    return Err(
                        format!("Error on line {}: More than one org set.",
                        token.line))
                }
                org_set = true;

                // ensure we have exactly one value for org
                if token.args.len() != 1 {
                    return Err(
                        format!("Error on line {}: incorrect number of args for org.", 
                        token.line))
                }

                // convert string to address
                origin = match address_from_string(&token.args[0]) {
                    Ok(origin) => origin,
                    Err(e) => return Err(
                        format!("Error on line {}: {}", token.line, e))
                };

            }
            _ => {}

        }
    }

    Ok(origin)

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_origin() {
        struct TestCase {
            name: &'static str,
            tokens: Vec<Token>,
            expected: Result<u16, String>,
        }

        let test_cases = [
            TestCase {
                name: "Single token org",
                tokens: vec![
                    Token {
                        name: "org".to_string(),
                        token_type: TokenType::Origin,
                        line: 0,
                        args: vec!["0x200".to_string()],
                    }
                ],
                expected: Result::Ok(0x200),
            },
            TestCase {
                name: "First line org token",
                tokens: vec![
                    Token {
                        name: "org".to_string(),
                        token_type: TokenType::Origin,
                        line: 0,
                        args: vec!["0x200".to_string()],
                    },
                    Token {
                        name: "foo".to_string(),
                        token_type: TokenType::Label,
                        line: 1,
                        args: Vec::new(),
                    }
                ],
                expected: Result::Ok(0x200),
            },
            TestCase {
                name: "Last line org token",
                tokens: vec![
                    Token {
                        name: "foo".to_string(),
                        token_type: TokenType::Label,
                        line: 0,
                        args: Vec::new(),
                    },
                    Token {
                        name: "org".to_string(),
                        token_type: TokenType::Origin,
                        line: 1,
                        args: vec!["0x200".to_string()],
                    }
                ],
                expected: Result::Ok(0x200),
            },
            TestCase {
                name: "Too many args",
                tokens: vec![
                    Token {
                        name: "org".to_string(),
                        token_type: TokenType::Origin,
                        line: 0,
                        args: vec!["0x200".to_string(), "0x200".to_string()],
                    }
                ],
                expected: Result::Err(
                    "Error on line 0: incorrect number of args for org.".to_string()),
            },
            TestCase {
                name: "Not enough args",
                tokens: vec![
                    Token {
                        name: "org".to_string(),
                        token_type: TokenType::Origin,
                        line: 0,
                        args: Vec::new(),
                    }
                ],
                expected: Result::Err(
                    "Error on line 0: incorrect number of args for org.".to_string()),
            }

        ];

        for case in test_cases.iter() {
            let result = get_origin(&case.tokens);
            assert_eq!(result, case.expected, "Failed on test case: {}", case.name);
        }

    }


}
