pub mod ast;
use crate::token_type::{Token, TokenType};


pub fn error(token: &Token, message: &str) {
    if token.token_type == TokenType::EOF {
        println!("ParseError: [line {}] at end {}", token.line, message);
    } else {
        println!("ParseError: [line {}] at '{}' {}", token.line,  token.lexeme, message);
    }
}