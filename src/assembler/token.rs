#[derive(Clone)]
pub struct Token {
    pub name: String,
    pub token_type: TokenType,
    pub args: Vec<String>,
    pub line: usize,

}

impl Token {
    pub fn to_string(&self) -> String {

        let token_type = match self.token_type {
            TokenType::Instruction => "Instruction",
            TokenType::Label => "Label",
            TokenType::Origin => "Origin",
        };

        format!("Token {} of type {} on line {} with args {}", 
            self.name, token_type, self.line, self.args.join(" "))
    }
}

#[derive(Clone)]
pub enum TokenType {
    Instruction,
    Label,
    Origin,
}