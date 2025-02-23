pub mod ast;
use crate::token_type::{Token, TokenType};


pub fn error(token: &Token, message: &str) {
    if token.token_type == TokenType::EOF {
        panic!("{} at end {}", token.line, message);
    } else {
        panic!("{} at '{}' {}", token.line,  token.lexeme, message);
    }
}