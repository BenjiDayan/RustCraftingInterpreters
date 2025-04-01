use crate::TokenType;

use super::{environment::Environment, Binary, Clock, Expr, ExprVisitor, Grouping, Literal, Logical, LoxCallable, NullFunc, Stmt, StmtVisitor, Token, Unary, Value, Variable};
use std::{any::{Any, TypeId}, cell::RefCell, fmt, rc::Rc, sync::Arc, sync::Mutex};

// use main::runtime_error func
use crate::runtime_error;



pub struct Interp {
    environment: Arc<Mutex<Environment>>,
    globals: Arc<Mutex<Environment>>
}

impl Interp {
    pub fn new() -> Self {
        let globals = Environment::new(None);
        // globals.define(
        //     &"clock".to_string(),
        //     Clock
        // );
        Self {
            environment: Arc::new(Mutex::new(Environment::new(None))),
            globals: Arc::new(Mutex::new(globals))
        }
    }
}

pub fn stringify(val: &Value) -> String {
    match val {
        Value::Number(s) => {
            let num_string = format!("{s}");
            // if val_string ends with ".0" then remove the ".0"
            if num_string.ends_with(".0") {
                // snip off last 2 characters
                num_string[..num_string.len()-2].to_string()
            } else {
                num_string
            }
        }
        Value::String(s) => format!("\"{s}\""),
        Value::Boolean(b) => b.to_string(),
        Value::Nil => "nil".to_string(),
    }
}

impl Interp {
    pub fn evaluate(&mut self, expr: &Expr) -> Result<Value> {
        self.visit_expr(expr)
    }

    pub fn interpret(&mut self, expr: &Expr) {
        let out = self.visit_expr(expr);
        if let Ok(val) = &out {
            let val_string: String = stringify(&val);
            println!("{val_string}");

        } else {
            // println!("{out:?}");
            runtime_error(out.unwrap_err());
        }
        // let out_string_rep = format!("{out:?}");
        // println!("{}", out_string_rep);
    }

    pub fn interpret_stmts(&mut self, stmts: &Vec<Stmt>) -> Result<()> {
        // visit each stmt one by one, evaluating. If any raise RuntimeError,
        // we return error. o/w finally return Ok(())
        for stmt in stmts {
            let res = self.visit_statement(stmt).unwrap_or_else(
                |error| {
                    runtime_error(error);
                }
            );
        }
        Ok(())
    }

    pub fn execute_block(&mut self, statements: &Vec<Stmt>, mut environment: Arc<Mutex<Environment>>) -> Result<()>{
        std::mem::swap(&mut self.environment, &mut environment);
        
        // Execute block with new environment
        let result = (|| {
            for stmt in statements {    
                self.visit_statement(stmt)?;
            }
            Ok(())
        })();

        std::mem::swap(&mut self.environment, &mut environment);
        result
    }
}


// TODO - this whole visitor pattern - in rust could we instead do like
// just a big match? Same for ExprVisitor.
// For Stmt this is already an enum of two categories.
// Expr actually has more categories??

// Stmt -> expression / print -> Expr
// but it seems that maybe this is so simple we don't setup so many trees for this.



impl StmtVisitor<Result<()>> for Interp {

    fn visit_expr_statement(&mut self, expr: &Expr) -> Result<()> {
        let val = self.visit_expr(expr)?;
        return Ok(());
    }

    fn visit_print_statement(&mut self, expr: &Expr) -> Result<()> {
        let val = self.visit_expr(expr)?;
        println!("{}", stringify(&val));
        return Ok(());
        // Err(RuntimeError::new(Token::new(TokenType::NIL, "".to_string(), Literal::Nil, 0), "Expected print statement".to_string()))
    }

    fn visit_if_statement(&mut self, if_stmt: &super::IfStmt) -> Result<()> {
        if is_truthy(&self.evaluate(&if_stmt.condition)?) {
            self.visit_statement(&if_stmt.if_branch)
        } else if let Some(else_stmt) = &if_stmt.else_branch {
            self.visit_statement(else_stmt)
        } else {
            Ok(())  // else branch was null so no statement to visit.
        }
    }

    fn visit_while_statement(&mut self, while_stmt: &super::WhileStmt) -> Result<()> {
        while (is_truthy(&self.evaluate(&while_stmt.condition)?)) {
            self.visit_statement(&while_stmt.body)?
        }
        Ok(())
    }

    fn visit_func_statement(&mut self, func_stmt: &super::FuncStmt) -> Result<()> {
        Ok(())
    }

    fn visit_block_statement(&mut self, statements: &Vec<Stmt>) -> Result<()> {
        // Create new environment with current environment as enclosing
        let block_env = Arc::new(Mutex::new(Environment::new(Some(Arc::clone(&self.environment)))));
        
        let result = self.execute_block(statements, block_env);
        
        // // Swap environments using std::mem::swap to avoid cloning
        // let mut temp_env = block_env;
        // std::mem::swap(&mut self.environment, &mut temp_env);
        
        // // Execute block with new environment
        // let result = (|| {
        //     for stmt in statements {    
        //         self.visit_statement(stmt)?;
        //     }
        //     Ok(())
        // })();

        // // Restore previous environment
        // std::mem::swap(&mut self.environment, &mut temp_env);
        
        result
    }

    fn visit_var_statement(&mut self, var: &Variable) -> Result<()> {
        // at this point we surely need to save the value in the environment
        // NB if var x; (without definition), we actually set .initializer
        // to Expr::Null in parser.
        let val = self.visit_expr(&var.initializer)?;
        self.environment.lock().unwrap().define(&var.name.lexeme, &val);
        return Ok(());
    }
}


//NB: I don't think with this matching system, we need to "check number operand"
impl ExprVisitor<Result<Value>> for Interp {
    fn visit_assignment(&mut self, assignment: &super::Assign) -> Result<Value> {
        let value = self.visit_expr(&assignment.value)?;
        self.environment.lock().unwrap().assign(&assignment.name, &value)?;
        Ok(value)
    }

    fn visit_logical(&mut self, logical: &Logical) -> Result<Value> {
        let left = self.evaluate(&logical.left)?;
        if logical.operator.token_type == TokenType::OR {
            if is_truthy(&left) {return Ok(left);}
        } else {
            if !is_truthy(&left) {return Ok(left);}
        }
        return self.evaluate(&logical.right);
    }

    fn visit_binary(&mut self, binary: &Binary) -> Result<Value> {
        let l = self.visit_expr(&binary.left)?;
        let r = self.visit_expr(&binary.right)?;


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
                        _ => Err(RuntimeError::new(binary.operator.clone(), "Unexpected binary operator for numbers".to_string()))    
                        //panic!("Unexpected binary operator for numbers")
                    }
                    }
                    (Value::String(a), Value::String(b)) => {
                        if let TokenType::PLUS = binary.operator.token_type {
                            Ok(Value::String(a.clone() + &b))
                        } else {
                            // panic!("Unexpected binary operator for strings")
                            Err(RuntimeError::new(binary.operator.clone(), "Unexpected binary operator for strings".to_string()))
                        }
                    }
                    // _ => panic!("no more valid operators etc.")
                    _ => Err(RuntimeError::new(binary.operator.clone(), "no more valid operators etc.".to_string()))
                }
            }
        }
    }


    fn visit_unary(&mut self, unary: &Unary) -> Result<Value> {
        let value = self.visit_expr(&unary.right)?;
        match unary.operator.token_type {
            TokenType::MINUS => {
                if let Value::Number(num) = value {
                    Ok(Value::Number(-num))
                } else {
                    // panic!("unary minus - value must be a number");
                    Err(RuntimeError::new(unary.operator.clone(), "unary minus - value must be a number".to_string()))
                }
            }
            TokenType::BANG => {
                Ok(Value::Boolean(!is_truthy(&value)))
            }
            _ => {
                // panic!("Unexpected unary operator")
                Err(RuntimeError::new(unary.operator.clone(), "Unexpected unary operator".to_string()))
            }
        }
    }

    fn visit_call(&mut self, call: &super::Call) -> Result<Value> {
        let callee = self.visit_expr(&call.callee)?;  // get the function
        // TODO - check if callee is a function

        let args: Result<Vec<Value>> = call.arguments.iter().map(|arg| self.visit_expr(arg)).collect();
        let args = args?;

        // Cast LoxCallable; check that it's indeed callable??
        let func = NullFunc; // LoxCallable

        if func.arity() != args.len() {
            return Err(RuntimeError::new(call.paren.clone(), format!(
                "Arity mismatch: func wants: {}, # args given: {}", func.arity(), args.len())
            ));
        }


        Ok(Value::Nil) // TODO!
    }

    fn visit_grouping(&mut self, grouping: &Grouping) -> Result<Value> {
        self.visit_expr(&grouping.0)
    }

    fn visit_literal(&mut self, literal: &Literal) -> Result<Value> {
        Ok(literal.val())
    }

    fn visit_variable(&mut self, token: &Token) -> Result<Value> {
        return self.environment.lock().unwrap().get(token);

        // let value = self.environment.values.get(&token.lexeme).cloned();
        // let foo = value.ok_or(RuntimeError::new(token.clone(), "couldn't visit variable".to_string()));
        // foo
        // or(
        //     RuntimeError::new(token.clone(), "couldn't visit variable".to_string())
        // );
        // value


        // Ok(variable.name.literal.val())
        // format!("var:{}", token.lexeme).to_string()
    }

    fn visit_null(&mut self) -> Result<Value> { Ok(Value::Nil) }
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
pub struct RuntimeError{
    pub token: Token,
    pub message: String
}

impl RuntimeError {
    fn new(token: Token, message: String) -> Self {
        Self { token, message }
    }
}

pub type Result<T> = std::result::Result<T, RuntimeError>;

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Runtime error at token: {:?}; line: {:?}; message: {:?}", self.token, self.token.line, self.message)
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
        println!("vec[3]: {:?}", vec.first().ok_or(RuntimeError::new(temp, "vec[3]".to_string())));

        // // let my_string = String::from("2 * (3 -\"muffin\")");
        // let my_string = String::from("1 + 2.2 * 3; 10*5;");
        // // let my_string: String = String::from("\"hi there\" + \" how are you\"");
        // let mut my_scanner = Scanner::new(my_string.clone());
        // let tokens = my_scanner.scan_tokens();
        // println!("tokens: {tokens:?}");

        // let mut my_parser = Parser::new(tokens);
        // let expr = my_parser.parse().unwrap();
        // println!("expr: {expr:?}");

        // let mut my_scanner = Scanner::new(my_string.clone());
        // let tokens = my_scanner.scan_tokens();
        // let mut my_parser = Parser::new(tokens);
        // let stmts = my_parser.parse_statements();
        // for stmt in &stmts{
        //     println!("stmt: {stmt:?}");
        // }


        // my_parser.advance();
        // let expr2 = my_parser.expression();

        // let printer = Printer;
        // let out = printer.print(&expr);
        // println!("printed: {out}");

        let mut my_interpreter = Interp::new();
        // let out = my_interpreter.visit_expr(&expr);
        // println!("interpreted: {:?}", out);
        // my_interpreter.interpret(&expr);
        // println!("is truthy: {:?}", is_truthy(&out));



        // let my_string = String::from("2 * (3 -\"muffin\")");
        // let my_string = String::from("print 1 + 2.2 * 3; print 10*5;");
        let my_string = String::from("print 2*3; var x = 3; print x;");
        // let my_string: String = String::from("\"hi there\" + \" how are you\"");
        let mut my_scanner = Scanner::new(my_string.clone());
        let tokens = my_scanner.scan_tokens();
        println!("tokens: {tokens:?}");

        let mut my_parser = Parser::new(tokens);
        let stmts = my_parser.parse();
        for stmt in &stmts{
            println!("stmt: {stmt:?}");
        }

        my_interpreter.interpret_stmts(&stmts);
        
    }
}

