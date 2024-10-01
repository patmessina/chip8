use super::token::{Token, TokenType};
use std::collections::HashMap;

pub fn get_labels(tokens: &Vec<Token>, _origin: u16) -> Result<HashMap<String, u16>, String> {

    let mut labels: HashMap<String, u16> = HashMap::new();

    // let mut pc = origin;
    let mut pc = 0x200;
    let mut errors: Vec<String> = Vec::new();

    for token in tokens {
        match token.token_type {
            TokenType::Label => {
                if labels.contains_key(&token.name) {
                    errors.push(
                        format!("Error on line {}: Label {} already defined.", 
                        token.line, token.name));
                    break
                }
                labels.insert(token.name.clone(), pc);
            },
            TokenType::Instruction => {
                pc += 2;
            },
            _=>{}
        }
    }

    Ok(labels)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_labels() {
        struct TestCase {
            name: &'static str,
            tokens: Vec<Token>,
            origin: u16,
            expected: Result<HashMap<String, u16>, String>,
        }

        let test_cases = [

            TestCase {
                name: "Empty tokens",
                tokens: Vec::new(),
                origin: 0x200,
                expected: Result::Ok(HashMap::new()),
            },

            TestCase {
                name: "No labels",
                tokens: vec![
                    Token {
                        name: "org".to_string(),
                        token_type: TokenType::Origin,
                        line: 0,
                        args: vec!["0x200".to_string()],
                    }
                ],
                origin: 0x200,
                expected: Result::Ok(HashMap::new()),
            },

            TestCase {
                name: "Single label",
                tokens: vec![
                    Token {
                        name: "foo".to_string(),
                        token_type: TokenType::Label,
                        line: 0,
                        args: Vec::new(),
                    }
                ],
                origin: 0x200,
                expected: Result::Ok(
                    HashMap::<String, u16>::from_iter(
                        vec![("foo".to_string(), 0x200)].into_iter())),
            },

            TestCase {
                name: "Two labels",
                tokens: vec![
                    Token {
                        name: "foo".to_string(),
                        token_type: TokenType::Label,
                        line: 0,
                        args: Vec::new(),
                    },
                    Token {
                        name: "ld".to_string(),
                        token_type: TokenType::Instruction,
                        line: 1,
                        args: vec!["v0".to_string(), "0x22".to_string()],
                    },
                    Token {
                        name: "foobar".to_string(),
                        token_type: TokenType::Label,
                        line: 2,
                        args: Vec::new(),
                    }
                ],
                origin: 0x200,
                expected: Result::Ok(
                    HashMap::<String, u16>::from_iter(
                        vec![("foo".to_string(), 0x200), ("foobar".to_string(), 0x202)].into_iter())),
            }

        ];

        for case in test_cases {
            let result = get_labels(&case.tokens, case.origin);
            assert_eq!(result, case.expected, "Failed on test case: {}", case.name);
        }

    }


}
