use crate::lox::ast;
use crate::lox::ast::{Expr, Binary, Unary, Grouping, Stmt, Assign};
use crate::token_type::{self, Literal, Token, TokenType};
use crate::lox::error;

use std::error::Error;

use super::{Call, FuncStmt, IfStmt, Logical, WhileStmt};
use super::{Variable, interpreter::RuntimeError};




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
    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut out = Vec::new();
        while !self.is_at_end() {
            // out.push(self.declaration().unwrap());
            match self.declaration() {
                Ok(stmt) => {
                    println!("stmt in parse: {stmt:?}");
                    out.push(stmt);
                }
                Err(e) => {
                    error(&e.token, &e.message);
                    self.synchronise();
                }
            }
        }
        out
    }

    fn synchronise(&mut self) {
        //discard tokens until at the beginning of the next declaration
        self.advance();
        while !self.is_at_end() {
            if self.previous().token_type == TokenType::SEMICOLON {return;}

            match self.peek().token_type {
                TokenType::CLASS |TokenType::FUN |TokenType::VAR |
                TokenType::FOR |TokenType::IF |TokenType::WHILE |
                TokenType::PRINT |TokenType::RETURN => {return;}
                _ => {self.advance();}
            }

        }
    }

    //TODO add in ParseError's. And synchronize??
    fn declaration(&mut self) -> Result<Stmt, RuntimeError> {
        // we only allow var declarations at this level,
        // i.e. top level - so within control statements not longer allowed
        // to var_decl I guess?
        if self.match_types(&[TokenType::VAR]) {
            self.var_declaration()

        } else if self.match_types(&[TokenType::FUN]) {
            self.func_declaration()
        } else {
            self.statement()
        }
        
    
        // res.unwrap_or_else()
        //synchronize? 
    }

    fn var_declaration(&mut self) -> Result<Stmt, RuntimeError> {
        // either "name;" or "name = expr;"
        let name = self.consume(TokenType::IDENTIFIER, "expected IDENTIFIER in var declaration")?;
        if self.match_types(&[TokenType::EQUAL]) {
            let initializer = self.expression()?;
            self.consume(TokenType::SEMICOLON, "Expected ';' after value")?;
            return Ok(Stmt::Var(Variable{name: name, initializer: initializer}))
        } else {
            // We set uninitialised variables to Nil. This seeems reasonable, although
            // we could instead raise a runtime error if accessing a non-initialised
            // variable.
            let initializer = Expr::Null;
            self.consume(TokenType::SEMICOLON, "Expected ';' after value")?;
            return Ok(Stmt::Var(Variable{name: name, initializer: initializer}))
        }
    }

    fn func_declaration(&mut self) -> Result<Stmt, RuntimeError> {
        let name = self.consume(TokenType::IDENTIFIER, "fun declaration lacking identifier")?;
        self.consume(TokenType::LEFT_PAREN, "expect '(' after func identifier decl")?;


        // immutable borrow here if no .clone()?
        let current = self.peek().clone();
        let mut parameters: Vec<Token> = Vec::new();
        while !self.check(TokenType::RIGHT_PAREN) {
            // error if >= 255 args
            if parameters.len() >= 255 {
                return Err(RuntimeError{token: current.clone(), message: "too many arguments".to_string()})?;
            }
            parameters.push(self.consume(TokenType::IDENTIFIER, "expected IDENTIFIER arg, got something else")?);
            if self.check(TokenType::COMMA) {
                self.advance();
            }
            // println!("args in finish_call: {args:?}");
        }
        self.consume(TokenType::RIGHT_PAREN, "Expected ')' after arguments")?;
        self.consume(TokenType::LEFT_BRACE, "Expected opening brace for func body")?;
        let body = self.block_statement()?;
        if let Stmt::Block(body) = body {
            return Ok(Stmt::Func(FuncStmt{
                name,
                parameters,
                body
            }));
        }
        Err(RuntimeError{token: name, message: "Expected block statement for function body".to_string()})
    }

    fn statement(&mut self) -> Result<Stmt, RuntimeError> {
        if self.match_types(&[TokenType::PRINT]) {
            self.print_statement()
        } else if self.match_types(&[TokenType::LEFT_BRACE]) {
            // This is a Stmt::Block(Vec<Stmt>), unlike the other
            // foolish single Stmt types.
            self.block_statement()
        } else if self.match_types(&[TokenType::IF]) {
            self.if_statement()
        } else if self.match_types(&[TokenType::WHILE]) {
            self.while_statement()
        } else if self.match_types(&[TokenType::FOR]) {
            self.for_statement()
        } else {
            self.expr_statement()
        }
        //placeholder
        // Stmt::Print(Expr::Literal(Literal::Nil))
    }

    fn print_statement(&mut self) -> Result<Stmt, RuntimeError> {
        let expr = self.expression()?;
        self.consume(TokenType::SEMICOLON, "Expected ';' after value")?;
        Ok(Stmt::Print(expr))
    }

    fn block_statement(&mut self) -> Result<Stmt, RuntimeError> {
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.check(TokenType::RIGHT_BRACE) && !self.is_at_end() {
            let current = self.peek();
            statements.push(self.declaration()?)
        }
        self.consume(TokenType::RIGHT_BRACE, "Expected '}' after block")?;
        Ok(Stmt::Block(statements))
    }

    fn if_statement(&mut self) -> Result<Stmt, RuntimeError> {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after IF");
        let condition = self.expression()?;
        self.consume(TokenType::RIGHT_PAREN, "Expected ')' closing IF condition");
        // NB not a declaration (if we want var_decl then must be in Block Stmt)
        let if_branch = Box::new(self.statement()?);
        let else_branch = if self.match_types(&[TokenType::ELSE]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        return Ok(Stmt::If(IfStmt{condition, if_branch, else_branch}))
    }

    fn while_statement(&mut self) -> Result<Stmt, RuntimeError> {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after WHILE");
        let condition = self.expression()?;
        self.consume(TokenType::RIGHT_PAREN, "Expected ')' closing WHILE condition");
        // NB not a declaration (if we want var_decl then must be in Block Stmt)
        let body = Box::new(self.statement()?);
        return Ok(Stmt::While(WhileStmt{condition, body}))
    }

    fn for_statement(&mut self) -> Result<Stmt, RuntimeError> {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after FOR");
        let initializer: Option<Stmt> = if self.match_types(&[TokenType::SEMICOLON]) {
            None
        } else if self.match_types(&[TokenType::VAR]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expr_statement()?)
        };

        let condition = if !self.check(TokenType::SEMICOLON) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::SEMICOLON, "Expected ';' before increment in FOR");

        let increment = if !self.check(TokenType::RIGHT_PAREN) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::RIGHT_PAREN, "Expected final ')' in FOR");
        
        let mut body = self.statement()?;

        if let Some(inc) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(inc)]);
        }
        let condition = condition.unwrap_or(Expr::Literal(Literal::Boolean(true)));
        body = Stmt::While(WhileStmt{condition: condition, body: Box::new(body)});
        
        if let Some(init) = initializer{
            body = Stmt::Block(vec![init, body]);
        }

        return Ok(body);
    }

    fn expr_statement(&mut self) -> Result<Stmt, RuntimeError> {
        let expr = self.expression()?;
        self.consume(TokenType::SEMICOLON, "Expected ';' after expression")?;
        Ok(Stmt::Expression(expr))
    }

    pub fn parse_expr(&mut self) -> Option<Expr>{
        self.expression().ok()
    }

    fn expression(&mut self) -> Result<Expr, RuntimeError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, RuntimeError> {
        let expr = self.or()?;
        if self.match_types(&[TokenType::EQUAL]) {
            let equals_token = self.previous();
            if let Expr::Variable ( token ) = expr {
                // Letting right be of type self.assignment, not one precedence level down
                // I think makes this right associative?
                let right = self.assignment()?;
                return Ok(Expr::Assign(Assign{name: token, value: Box::new(right)}))
            } else {
                return Err(RuntimeError{token: equals_token , message: "trailing equal sign in non assignment expression??".to_string()})?;
            }
        } else {
            // return Err(RuntimeError{token: self.previous() , message: "trailing equal sign in non assignment expression??".to_string()})?;
            return Ok(expr);
        }
    }

    fn or(&mut self) -> Result<Expr, RuntimeError> {
        let mut expr = self.and()?;
        while self.match_types(&[TokenType::OR]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::Logical(Logical{
                operator: operator,
                left: Box::new(expr),
                right: Box::new(right)});
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, RuntimeError> {
        let mut expr = self.equality()?;
        while self.match_types(&[TokenType::AND]) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Expr::Logical(Logical{
                operator: operator,
                left: Box::new(expr),
                right: Box::new(right)});
        }
        Ok(expr)
    }


    fn equality(&mut self) -> Result<Expr, RuntimeError> {
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

    fn comparison(&mut self) -> Result<Expr, RuntimeError> {
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

    fn term(&mut self) -> Result<Expr, RuntimeError> {
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

    fn factor(&mut self) -> Result<Expr, RuntimeError> {
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



    fn unary(&mut self) -> Result<Expr, RuntimeError> {
        if self.match_types(&[TokenType::BANG, TokenType::MINUS]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary(Unary{operator, right: Box::new(right)}))
        }
        return self.call()
    }

    fn call(&mut self) -> Result<Expr, RuntimeError> {
        let mut callee = self.primary()?;
        // println!("current token: {:?}", self.peek());

        while true {
            if self.match_types(&[TokenType::LEFT_PAREN]) {
                let args: Vec<Expr> = self.finish_call()?;
                let paren = self.previous();
                callee = Expr::Call(Call{callee: Box::new(callee), paren: paren, arguments: args});
                // println!("callee in func call: {callee:?}");
            } else {
                break
            }
        }
        return Ok(callee);
    }

    fn finish_call(&mut self) -> Result<Vec<Expr>, RuntimeError> {
        // immutable borrow here if no .clone()?
        let current = self.peek().clone();
        let mut args: Vec<Expr> = Vec::new();
        // println!("in finish_call");
        while !self.check(TokenType::RIGHT_PAREN) {
            // error if >= 255 args
            if args.len() >= 255 {
                return Err(RuntimeError{token: current.clone(), message: "too many arguments".to_string()})?;
            }
            args.push(self.expression()?);
            if self.check(TokenType::COMMA) {
                self.advance();
            }
            // println!("args in finish_call: {args:?}");
        }
        self.consume(TokenType::RIGHT_PAREN, "Expected ')' after arguments")?;

        return Ok(args);
    }

    fn primary(&mut self) -> Result<Expr, RuntimeError> {
        let current_token = self.peek();
        // println!("current_token in primary: {current_token:?}");


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
                self.consume(TokenType::RIGHT_PAREN, "Expect ')' after expression.")?;
                return Ok(Expr::Grouping(Grouping(Box::new(expr))))
            }

            // We store a Expression::Variable that will point to (via environment) a
            // Variable object
            Token{token_type: TokenType::IDENTIFIER, ..} => {
                Expr::Variable(current_token.clone())
            } 
            // I feel like this is not meant to happen
            _ => {
                // println!("catch all not meant to happen!!");
                // return Ok(Expr::Literal(Literal::String("aaah".to_string())))
                return Err(RuntimeError{token: current_token.clone(), message: "primary unable to match".to_string()})?;
                // return Ok(Expr::Literal(Literal::String("aaah".to_string())))

            }
        };
        self.advance();  // only the expression branch consumes
        Ok(out_token)


        // let placeholder = self.peek().lexeme.clone();
        // self.advance();
        // Expr::Literal(Literal::String(placeholder.clone()))
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, RuntimeError>{
        if self.check(token_type) {
            // println!("consumed {token_type:?}");
            return Ok(self.advance())
        }
        let token = self.peek();
        // error(token, message);
        // TODO why doesn't compile if comment out bottom?
        // panic!("token: {token:?} {message}");
        Err(RuntimeError{token: token.clone(), message: message.to_string()})
    }

    // fn consume(&mut self, token_type: TokenType, message: &str) -> Token{
    //     if self.check(token_type) {
    //         println!("consumed {token_type:?}");
    //         return self.advance()
    //     }
    //     let token = self.peek();
    //     error(token, message);
    //     // TODO why doesn't compile if comment out bottom?
    //     panic!("token: {token:?} {message}");
    // }

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

        let my_string = String::from("func(x, y, z)(2);");
        let mut my_scanner = Scanner::new(my_string);
        let tokens = my_scanner.scan_tokens();
        println!("tokens: {tokens:?}");

        let mut my_parser = Parser::new(tokens);
        let expr = my_parser.parse_expr().unwrap();
        println!("expr: {expr:?}");
        // my_parser.advance();
        // let expr2 = my_parser.expression();

        let mut printer = Printer;
        let out = printer.print(&expr);
        println!("printed: {out}");
        // let out2 = printer.print(&expr2);
        // println!("{out2}");
        // println!("{expr2:?}");

        assert!(3 == 3);
    }
}