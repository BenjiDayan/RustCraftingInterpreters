
use std::sync::Arc;
use std::sync::Mutex;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::token_type::Token;
use crate::token_type::Literal;
use crate::token_type::Value;

use environment::Environment;
use interpreter::{Interp, Result};
use printer::Printer;

pub mod parser;
pub mod printer;
pub mod environment;
// pub mod interpreter_old;
pub mod interpreter;

#[derive(Debug, Clone)]
pub struct LoxFunction {
    declaration: FuncStmt
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        self.declaration.parameters.len()
    }

    fn call(&self, mut interpreter: Interp, arguments: Vec<Value>) -> Result<Expr> {
        let mut environment = Environment::new(None);
        for i in 0..(arguments.len()-1) {
            let arg = &arguments[i];
            let param = &self.declaration.parameters[i];
            environment.define(&param.lexeme, arg);
        }
        let env2 = Arc::new(Mutex::new(environment));
        interpreter.execute_block(&self.declaration.body, env2);
        return Ok(Expr::Literal(Literal::Nil))

    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var(Variable),
    Block(Vec<Stmt>),
    If(IfStmt),
    While(WhileStmt),
    Func(FuncStmt),
}

#[derive(Debug, Clone)]
pub struct FuncStmt {
    name: Token,
    parameters: Vec<Token>,
    body: Vec<Stmt>
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    condition: Expr,
    body: Box<Stmt>,
}


#[derive(Debug, Clone)]
pub struct IfStmt {
    condition: Expr,
    // NB if you need "multiple" statements in the if_branch,
    // This is handled by a Stmt::Block.
    if_branch: Box<Stmt>,
    else_branch: Option<Box<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct Variable {
    name: Token,
    initializer: Expr
}

#[derive(Debug, Clone)]
pub enum Expr {
    Assign(Assign),
    Logical(Logical),
    Binary(Binary),
    Unary(Unary),
    Call(Call),
    Grouping(Grouping),
    // LiteralExpr(LiteralExpr),
    Literal(Literal),
    Variable(Token), // Token(IDENTIFIER, name, NIL, )
    Null
}

#[derive(Debug, Clone)]
pub struct Call {
    callee: Box<Expr>,
    paren: Token,
    arguments: Vec<Expr>
}

// Stmt::Var is for `var x = 4;` etc.,
// whereas Expr::Assign is for `x = 4;`
// These are assuredly different things?? Well the first is
// clearly also an intitialsing assignment, yet the latter is
// also meant to be evaluatable as an expression? And even
// right associative. x = y = 3 means x = (y = 3), though what
// (y=3) evaluates to idk?
#[derive(Debug, Clone)]
pub struct Assign {
    name: Token,
    value: Box<Expr>
}

#[derive(Debug, Clone)]
pub struct Logical {
    operator: Token,  // actually only AND or OR
    left: Box<Expr>,
    right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Binary {
    operator: Token,
    left: Box<Expr>,
    right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Unary {
    operator: Token,
    right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Grouping(Box<Expr>);

// #[derive(Debug)]
// pub struct LiteralExpr{
//     literal: Literal
// }

// TODO: make this Derive-able
pub trait ExprVisitor<T> {
    // NOTE: would it be better to make these associated functions without &self?
    // fn visit_expr(&self, expr: &Expr) -> T;
    fn visit_expr(&mut self, expr: &Expr) -> T {
            match expr {
            Expr::Assign(assign) => self.visit_assignment(assign),
            Expr::Logical(logical) => self.visit_logical(logical),
            Expr::Binary(binary) => self.visit_binary(binary),
            Expr::Unary(unary) => self.visit_unary(unary),
            Expr::Call(call) => self.visit_call(call),
            Expr::Literal(literal) => self.visit_literal(literal),
            Expr::Grouping(grouping) => self.visit_grouping(grouping),
            Expr::Variable(token) => self.visit_variable(token),
            Expr::Null => self.visit_null(),
        }
    }
    fn visit_assignment(&mut self, assignment: &Assign) -> T;
    fn visit_logical(&mut self, logical: &Logical) -> T;
    fn visit_binary(&mut self, binary: &Binary) -> T;
    fn visit_unary(&mut self, unary: &Unary) -> T;
    fn visit_call(&mut self, call: &Call) -> T;
    fn visit_literal(&mut self, literal: &Literal) -> T;
    fn visit_grouping(&mut self, grouping: &Grouping) -> T;
    fn visit_variable(&mut self, token: &Token) -> T;
    fn visit_null(&mut self) -> T;
}

// wtf?? unclear if we actually need this??
pub trait StmtVisitor<T> {
    fn visit_statement(&mut self, stmt: &Stmt) -> T {
        match stmt {
            Stmt::Expression(expr) => self.visit_expr_statement(expr),
            Stmt::Print(expr) => self.visit_print_statement(expr),
            Stmt::Block(statements) => self.visit_block_statement(statements),
            Stmt::While(while_stmt) => self.visit_while_statement(while_stmt),
            Stmt::Func(func_stmt) => self.visit_func_statement(func_stmt),
            Stmt::Var(var) => self.visit_var_statement(var),
            Stmt::If(if_stmt) => self.visit_if_statement(if_stmt)
        }
    }
    fn visit_expr_statement(&mut self, expr: &Expr) -> T;
    fn visit_print_statement(&mut self, expr: &Expr) -> T;
    fn visit_block_statement(&mut self, statements: &Vec<Stmt>) -> T;
    fn visit_while_statement(&mut self, while_stmt: &WhileStmt) -> T;
    fn visit_func_statement(&mut self, func_stmt: &FuncStmt) -> T;
    fn visit_var_statement(&mut self, var: &Variable) -> T;
    fn visit_if_statement(&mut self, if_stmt: &IfStmt) -> T;
}

pub trait LoxCallable {
    fn call(&self, interpreter: Interp, arguments: Vec<Value>) -> Result<Expr>;
    fn arity(&self) -> usize;
}

pub struct NullFunc;

impl LoxCallable for NullFunc{
    fn arity(&self) -> usize{ 0}
    fn call(&self, interpreter: Interp, arguments: Vec<Value>) -> Result<Expr>{
        Ok(Expr::Literal(Literal::Nil))
    }
}

pub struct Clock;
impl LoxCallable for Clock{
    fn arity(&self) -> usize{ 0}
    fn call(&self, interpreter: Interp, arguments: Vec<Value>) -> Result<Expr>{
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
        Ok(Expr::Literal(Literal::Number(time)))
    }
}


// pub struct Printer;
// impl Printer {
//     pub fn print(&self, expr: &Expr) -> String {
//         self.visit_expr(expr)
//     }
//     fn parenthesize(&self, name: &str, expressions: Vec<&Expr>) -> String {
//         let mut s = format!("({}", name);
//         for expr in expressions {
//             s.push_str(&format!(" {}", &self.visit_expr(expr)));
//         }
//         s + ")"
//     }
// }

// impl ExprVisitor<String> for Printer {
//     // Instead of having every Expr subclass have an .accept(ExprVisitor<R>) method
//     // which returns ExprVisitor<R>.visit_mysubclass(), instead I've defined this
//     // visit_epxr method which uses match to dispatch efficiently (I believe?) to
//     // the right visit_mysubclass method.
//     // I.e. accept on the subclasses is replaced by one visit_mysubclass multimatching
//     // on this here implementation.
//     // but this is actually wasted code - we should have this be a one-off surely?
//     // a generic?? like it should be defined on ExprVisitor<R> instead.
//     // TODO confirm if that's
//     // fn visit_expr(&self, expr: &Expr) -> String {
//     //     match expr {
//     //         Expr::Binary(binary) => self.visit_binary(binary),
//     //         Expr::Unary(unary) => self.visit_unary(unary),
//     //         Expr::Literal(literal) => self.visit_literal(literal),
//     //         Expr::Grouping(grouping) => self.visit_grouping(grouping),
//     //     }
//     // }

//     fn visit_binary(&self, binary: &Binary) -> String {
//         self.parenthesize(&binary.operator.lexeme, vec![&binary.left, &binary.right])
//     }

//     fn visit_unary(&self, unary: &Unary) -> String {
//         self.parenthesize(&unary.operator.lexeme, vec![&unary.right])
//     }

//     fn visit_grouping(&self, grouping: &Grouping) -> String {
//         self.parenthesize("group", vec![&grouping.0])
//     }

//     fn visit_literal(&self, literal: &Literal) -> String {
//         match literal {
//             Literal::String(val) => val.clone(),
//             Literal::Number(val) => val.to_string(),
//             Literal::Boolean(val) => val.to_string(),
//             Literal::Nil => "nil".to_owned(),
//         }
//     }

//     fn visit_variable(&self, token: &Token) -> String {
//         format!("var {}", token.lexeme)
//     }

//     fn visit_null(&self) -> String {
//         "nil".to_owned()
//     }
// }

#[cfg(test)]
mod test {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::lox::ast::{Binary, Expr, Grouping, Literal, Unary};
    use crate::token_type::{Token, TokenType};

    use super::printer;

    #[test]
    fn test_ast_printer() {
        let expr = Expr::Binary(Binary {
            operator: Token::new(TokenType::STAR, "*".to_owned(), Literal::Nil, 0),
            left: Box::new(Expr::Unary(Unary {
                operator: Token::new(TokenType::MINUS, "-".to_owned(), Literal::Nil, 0),
                right: Box::new(Expr::Literal(Literal::Number(123.))),
            })),
            right: Box::new(Expr::Grouping(Grouping(Box::new(Expr::Literal(
                Literal::Number(45.67),
            )))))
        });
        let mut printer = printer::Printer;
        let out = printer.print(&expr);
        println!("{out}");
        assert!(out == "(* (- 123) (group 45.67))");
        println!("EPOCH: {:?} time since EPOCH: {:?}", UNIX_EPOCH, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
    }
}