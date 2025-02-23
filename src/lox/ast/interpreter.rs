use crate::TokenType;

use super::{Binary, Expr, ExprVisitor, Grouping, Literal, Unary, Value, Token};
use std::{any::{Any, TypeId}, fmt};

pub struct Interp;

//NB: I don't think with this matching system, we need to "check number operand"

impl ExprVisitor<Value> for Interp {
    fn visit_expr(&self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Binary(binary) => self.visit_binary(binary),
            Expr::Unary(unary) => self.visit_unary(unary),
            Expr::Literal(literal) => self.visit_literal(literal),
            Expr::Grouping(grouping) => self.visit_grouping(grouping),
        }
    }
    fn visit_binary(&self, binary: &Binary) -> Result<Value> {
        let l = self.visit_expr(&binary.left);
        let r = self.visit_expr(&binary.right);


        match binary.operator.token_type {
            TokenType::EQUAL_EQUAL => Ok(Value::Boolean(is_equal(&l, &r))),
            TokenType::BANG_EQUAL => Ok(Value::Boolean(!is_equal(&l, &r))),
            _ => {
                match (l, r) {
                    (Value::Number(a), Value::Number(b)) => {
                    match binary.operator.token_type {
                        TokenType::GREATER => Ok(Value::Boolean(a > b)),
                        TokenType::GREATER_EQUAL => Ok(Value::Boolean(a >= b)),
                        TokenType::LESS => Ok(Value::Boolean(a < b)),
                        TokenType::LESS_EQUAL => Ok(Value::Boolean(a <= b)),
                        TokenType::MINUS => Ok(Value::Number(a - b)),
                        TokenType::SLASH => Ok(Value::Number(a / b)),
                        TokenType::STAR => Ok(Value::Number(a * b)),
                        TokenType::PLUS => Ok(Value::Number(a + b)),
                        _ => 
                        
                        panic!("Unexpected binary operator for numbers")
                    }
                    }
                    (Value::String(a), Value::String(b)) => {
                        if let TokenType::PLUS = binary.operator.token_type {
                            Ok(Value::String(a.clone() + &b))
                        } else {
                            panic!("Unexpected binary operator for strings")
                        }
                    }
                    _ => panic!("no more valid operators etc.")
                }
            }
        }
    }


    fn visit_unary(&self, unary: &Unary) -> Result<Value> {
        let value = self.visit_expr(&unary.right);
        match unary.operator.token_type {
            TokenType::MINUS => {
                if let Value::Number(num) = value {
                    Ok(Value::Number(-num))
                } else {
                    panic!("unary minus - value must be a number");
                }
            }
            TokenType::BANG => {
                Ok(Value::Boolean(!is_truthy(&value)))
            }
            _ => {
                panic!("Unexpected unary operator")
            }
        }
    }

    fn visit_grouping(&self, grouping: &Grouping) -> Result<Value> {
        self.visit_expr(&grouping.0)
    }

    fn visit_literal(&self, literal: &Literal) -> Result<Value> {
        Ok(literal.val())
    }
}

fn is_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        (Value::Nil, Value::Nil) => true,
        _ => false
    }
}

fn is_truthy(thing: &Value) -> bool {

    match thing {
        Value::Nil => false,
        Value::Boolean(b) => *b,
        _ => true
    }
}

#[derive(Debug, Clone)]
struct RuntimeError{
    token: Token
}

impl RuntimeError {
    fn new(token: Token) -> Self {
        Self { token }
    }
}

pub type Result<T> = std::result::Result<T, RuntimeError>;

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Runtime error at token: {:?}; line: {:?}", self.token, self.token.line)
    }
}




fn example() {
    // Create some boxed values
    let boxed_bool = Box::new(true) as Box<dyn Any>;
    let boxed_nil = Box::new(()) as Box<dyn Any>;
    
    // Using as_ref() and type_id()
    assert_eq!(boxed_nil.as_ref().type_id(), TypeId::of::<()>());  // true
    println!("what is this type anyway? {:?}", TypeId::of::<()>());
    assert_eq!(boxed_bool.as_ref().type_id(), TypeId::of::<bool>());  // true
    
    // Using downcast_ref
    let bool_ref = boxed_bool.downcast_ref::<bool>();  // Some(&true)
    let wrong_type = boxed_bool.downcast_ref::<i32>();  // None
    
    match bool_ref {
        Some(value) => println!("Found a boolean: {}", value),
        None => println!("Not a boolean!")
    }
}

#[cfg(test)]
mod test {
    use crate::lox::ast::{Binary, Expr, Grouping, Literal, Printer, Unary};
    use crate::token_type::{Token, TokenType};
    use crate::scanner::Scanner;
    use crate::lox::ast::parser::Parser;
    use crate::lox::ast::interpreter::{Interp, ExprVisitor, is_truthy, RuntimeError};

    #[test]
    fn test_parser2() {
        println!("hi TEST START");
        let temp = Token::new(TokenType::NUMBER, "4".to_string(),
        Literal::Number(4.0), 0);
        println!("asdf {:?}", temp.literal.val());

        let vec: Vec<i32> = vec![1,2,3];
        println!("vec[3]: {:?}", vec.first().ok_or(RuntimeError::new(temp)));

        // let my_string = String::from("2 * (3 / -\"muffin\")");
        let my_string = String::from("1 + 2 * 3");
        let mut my_scanner = Scanner::new(my_string);
        let tokens = my_scanner.scan_tokens();
        println!("tokens: {tokens:?}");

        let mut my_parser = Parser::new(tokens);
        let expr = my_parser.parse().unwrap();
        println!("expr: {expr:?}");
        // my_parser.advance();
        // let expr2 = my_parser.expression();

        let printer = Printer;
        let out = printer.print(&expr);
        println!("printed: {out}");

        let my_interpreter = Interp;
        let out = my_interpreter.visit_expr(&expr);
        println!("interpreted: {:?}", out);
        println!("is truthy: {:?}", is_truthy(&out));
        
    }
}

