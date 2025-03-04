use crate::{token_type::Value, TokenType};

use super::{Binary, Expr, ExprVisitor, Grouping, Literal, Unary};
use std::any::{Any, TypeId};

macro_rules! numeric_op {
    ($a:expr, $b:expr, $op:tt) => {
        match ($a.downcast_ref::<f64>(), $b.downcast_ref::<f64>()) {
            (Some(x), Some(y)) => Box::new(x $op y),
            _ => panic!("Operands must be numbers")
        }
    };
}

pub struct Interpreter;

impl ExprVisitor<Box<dyn Any>> for Interpreter {
    fn visit_expr(&self, expr: &Expr) -> Box<dyn Any> {
        match expr {
            Expr::Binary(binary) => self.visit_binary(binary),
            Expr::Unary(unary) => self.visit_unary(unary),
            Expr::Literal(literal) => self.visit_literal(literal),
            Expr::Grouping(grouping) => self.visit_grouping(grouping),
        }
    }
    fn visit_binary(&self, binary: &Binary) -> Box<dyn Any> {
        let l = self.visit_expr(&binary.left);
        let r = self.visit_expr(&binary.right);
        
        match binary.operator.token_type {
            // Numeric comparisons and arithmetic
            TokenType::GREATER => numeric_op!(l, r, >),
            TokenType::GREATER_EQUAL => numeric_op!(l, r, >=),
            TokenType::LESS => numeric_op!(l, r, <),
            TokenType::LESS_EQUAL => numeric_op!(l, r, <=),
            TokenType::MINUS => numeric_op!(l, r, -),
            TokenType::SLASH => numeric_op!(l, r, /),
            TokenType::STAR => numeric_op!(l, r, *),
            
            TokenType::PLUS => {
                // Addition handles both numbers and strings
                if let (Some(a), Some(b)) = (l.downcast_ref::<f64>(), r.downcast_ref::<f64>()) {
                    Box::new(a + b)
                } else if let (Some(a), Some(b)) = (l.downcast_ref::<String>(), r.downcast_ref::<String>()) {
                    Box::new(a.clone() + b)
                } else {
                    panic!("Operands must be two numbers or two strings")
                }
            }

            TokenType::BANG_EQUAL => Box::new(!is_equal(&l, &r)),
            TokenType::EQUAL_EQUAL => Box::new(is_equal(&l, &r)),
            
            _ => Box::new(())
        }
    }


    fn visit_unary(&self, unary: &Unary) -> Box<dyn Any> {
        let value = self.visit_expr(&unary.right);
        match unary.operator.token_type {
            TokenType::MINUS => {
                let num = value.downcast_ref::<f64>()
                    .expect("Operand must be a number");
                Box::new(-num)
            }
            TokenType::BANG => {
                return  Box::new(! isTruthy(value));
            }
            _ => {
                panic!("Unexpected unary operator")
            }
        }
    }

    fn visit_grouping(&self, grouping: &Grouping) -> Box<dyn Any> {
        self.visit_expr(&grouping.0)
    }

    fn visit_literal(&self, literal: &Literal) -> Box<dyn Any> {
        // get out the value
        let value = literal.val();
        match value {
            Value::String(s) => Box::new(s),
            Value::Number(n) => Box::new(n),
            Value::Boolean(b) => Box::new(b),
            Value::Nil => Box::new(()),
        }
    }
}

fn isTruthy(thing: Box<dyn Any>) -> bool {
    if thing.as_ref().type_id() == TypeId::of::<()>() {
        return false;
    }
    if let Some(boolean) = thing.downcast_ref::<bool>() {
        return *boolean;
    }
    return true;
}

fn is_equal(a: &Box<dyn Any>, b: &Box<dyn Any>) -> bool {
    if a.as_ref().type_id() != b.as_ref().type_id() {
        return false;
    }
    
    match a.as_ref().type_id() {
        t if t == TypeId::of::<f64>() => {
            a.downcast_ref::<f64>().unwrap() == b.downcast_ref::<f64>().unwrap()
        }
        t if t == TypeId::of::<String>() => {
            a.downcast_ref::<String>().unwrap() == b.downcast_ref::<String>().unwrap()
        }
        t if t == TypeId::of::<bool>() => {
            a.downcast_ref::<bool>().unwrap() == b.downcast_ref::<bool>().unwrap()
        }
        t if t == TypeId::of::<()>() => true,  // nil == nil
        _ => false
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
    // use crate::lox::ast::{Binary, Expr, Grouping, Literal, Printer, Unary};
    // use crate::token_type::{Token, TokenType};
    // use crate::scanner::Scanner;
    // use crate::lox::ast::parser::Parser;
    use crate::lox::ast::interpreter_old::{example, print_any};

    #[test]
    fn test_parser() {
        println!("hi TEST START");

        example();
    }

    use crate::lox::ast::{Binary, Expr, Grouping, Literal, Printer, Unary};
    use crate::token_type::{Token, TokenType};
    use crate::scanner::Scanner;
    use crate::lox::ast::parser::Parser;
    use crate::lox::ast::interpreter_old::{Interpreter, ExprVisitor, isTruthy};

    #[test]
    fn test_parser2() {
        println!("hi TEST START");
        let temp = Token::new(TokenType::NUMBER, "4".to_string(),
        Literal::Number(4.0), 0);
        println!("asdf {:?}", temp.literal.val());

        let my_string = String::from("(2 + 3) * 4");
        let mut my_scanner = Scanner::new(my_string);
        let tokens = my_scanner.scan_tokens();
        println!("tokens: {tokens:?}");

        let mut my_parser = Parser::new(tokens);
        let expr = my_parser.parse_expr().unwrap();
        println!("expr: {expr:?}");
        // my_parser.advance();
        // let expr2 = my_parser.expression();

        let printer = Printer;
        let out = printer.print(&expr);
        println!("printed: {out}");

        let my_interpreter = Interpreter;
        let out = my_interpreter.visit_expr(&expr);
        println!("interpreted:");
        print_any(&out);
        println!("is truthy: {:?}", isTruthy(out));
        
    }
}


pub fn print_any(value: &Box<dyn Any>) {
    let type_id = value.as_ref().type_id();
    
    if type_id == TypeId::of::<f64>() {
        println!("{}", value.downcast_ref::<f64>().unwrap());
    } else if type_id == TypeId::of::<String>() {
        println!("{}", value.downcast_ref::<String>().unwrap());
    } else if type_id == TypeId::of::<bool>() {
        println!("{}", value.downcast_ref::<bool>().unwrap());
    } else if type_id == TypeId::of::<()>() {
        println!("nil");
    } else {
        println!("Unknown type: {:?}", type_id);
    }
}