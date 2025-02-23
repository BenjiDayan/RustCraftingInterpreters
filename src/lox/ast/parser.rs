use crate::lox::ast;
use crate::lox::ast::{Expr, Binary, Unary, Grouping};
use crate::token_type::{Literal,
    Token, TokenType};
use crate::lox::error;

use std::error::Error;




pub struct Parser {
    tokens: Vec<Token>,
    current: i32
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self{
        Parser{
            tokens,
            current: 0
        }
    }


    pub fn parse(&mut self) -> Option<Expr>{
        self.expression().ok()
    }

    fn expression(&mut self) -> Result<Expr, Box<dyn Error>> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, Box<dyn Error>> {
        let mut expr = self.comparison()?;
        
        while self.match_types(
            &[TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary(Binary{
                operator: operator,
                left: Box::new(expr),
                right: Box::new(right)
            })
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, Box<dyn Error>> {
        let mut expr = self.term()?;

        while self.match_types(
            &[TokenType::GREATER, TokenType::GREATER_EQUAL,
            TokenType::LESS, TokenType::LESS_EQUAL]
        ) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary(Binary{
                operator,
                left: Box::new(expr),
                right: Box::new(right)
            })
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, Box<dyn Error>> {
        let mut expr = self.factor()?;

        while self.match_types(
            &[TokenType::MINUS, TokenType::PLUS]
        ) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary(Binary{
                operator,
                left: Box::new(expr),
                right: Box::new(right)
            })
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, Box<dyn Error>> {
        let mut expr = self.unary()?;

        while self.match_types(
            &[TokenType::STAR, TokenType::SLASH]
        ) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary(Binary{
                operator,
                left: Box::new(expr),
                right: Box::new(right)
            })
        }
        Ok(expr)
    }



    fn unary(&mut self) -> Result<Expr, Box<dyn Error>> {
        if self.match_types(&[TokenType::BANG, TokenType::MINUS]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary(Unary{operator, right: Box::new(right)}))
        }
        return self.primary()
    }

    fn primary(&mut self) -> Result<Expr, Box<dyn Error>> {
        let current_token = self.peek();
        println!("current_token in primary: {current_token:?}");


        let out_token = match current_token {
            Token{token_type: TokenType::FALSE, ..} =>
                Expr::Literal(Literal::Boolean(false)),
            Token{token_type: TokenType::TRUE, ..} =>
                Expr::Literal(Literal::Boolean(true)),
            Token{token_type: TokenType::NIL, ..} =>
                Expr::Literal(Literal::Nil),
            Token{token_type: TokenType::NUMBER, ..} | Token{token_type: TokenType::STRING, ..} =>
                Expr::Literal(current_token.literal.clone()),
            Token{token_type: TokenType::LEFT_PAREN, ..} => {
                self.advance();  // past the '('
                let expr = self.expression()?;
                self.consume(TokenType::RIGHT_PAREN, "Expect ')' after expression.");
                return Ok(Expr::Grouping(Grouping(Box::new(expr))))
            }
            // I feel like this is not meant to happen
            _ => {
                println!("catch all not meant to happen!!");
                // return Ok(Expr::Literal(Literal::String("aaah".to_string())))
                Err("aaah2")?;
                return Ok(Expr::Literal(Literal::String("aaah".to_string())))

            }
        };
        self.advance();  // only the expression branch consumes
        Ok(out_token)


        // let placeholder = self.peek().lexeme.clone();
        // self.advance();
        // Expr::Literal(Literal::String(placeholder.clone()))
    }
    
    fn consume(&mut self, token_type: TokenType, message: &str) -> Token{
        if self.check(token_type) {println!("consumed {token_type:?}"); return self.advance()}
        let token = self.peek();
        error(token, message);
        // TODO why doesn't compile if comment out bottom?
        panic!("token: {token:?} {message}");
    }

    fn match_types(&mut self, types: &[TokenType]) -> bool {
        for &token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    //check if current token is of type token_type
    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        let foo = self.peek();
        // println!("foo: {foo:?}; {token_type:?}");
        foo.token_type == token_type
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current as usize]
    }

    fn previous(&self) -> Token {
        self.tokens[(self.current - 1) as usize].clone()
    }
}


#[cfg(test)]
mod test {
    use crate::lox::ast::{Binary, Expr, Grouping, Literal, Printer, Unary};
    use crate::token_type::{Token, TokenType};
    use crate::scanner::Scanner;
    use crate::lox::ast::parser::Parser;

    #[test]
    fn test_parser() {
        println!("hi TEST START");

        let my_string = String::from("1 == 3 + 4");
        let mut my_scanner = Scanner::new(my_string);
        let tokens = my_scanner.scan_tokens();
        println!("tokens: {tokens:?}");

        let mut my_parser = Parser::new(tokens);
        let expr = my_parser.expression().unwrap();
        println!("expr: {expr:?}");
        // my_parser.advance();
        // let expr2 = my_parser.expression();

        let printer = Printer;
        let out = printer.print(&expr);
        println!("printed: {out}");
        // let out2 = printer.print(&expr2);
        // println!("{out2}");
        // println!("{expr2:?}");

        assert!(3 == 3);
    }
}