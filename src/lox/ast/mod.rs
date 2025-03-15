
use crate::token_type::Token;
use crate::token_type::Literal;
use crate::token_type::Value;

use printer::Printer;

pub mod parser;
pub mod printer;
pub mod environment;
// pub mod interpreter_old;
pub mod interpreter;

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var(Variable),
    Block(Vec<Stmt>)
}

#[derive(Debug, Clone)]
pub struct Variable {
    name: Token,
    initializer: Expr
}

#[derive(Debug, Clone)]
pub enum Expr {
    Assign(Assign),
    Binary(Binary),
    Unary(Unary),
    Grouping(Grouping),
    // LiteralExpr(LiteralExpr),
    Literal(Literal),
    Variable(Token), // Token(IDENTIFIER, name, NIL, )
    Null
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
            Expr::Binary(binary) => self.visit_binary(binary),
            Expr::Unary(unary) => self.visit_unary(unary),
            Expr::Literal(literal) => self.visit_literal(literal),
            Expr::Grouping(grouping) => self.visit_grouping(grouping),
            Expr::Variable(token) => self.visit_variable(token),
            Expr::Null => self.visit_null(),
        }
    }
    fn visit_assignment(&mut self, assignment: &Assign) -> T;
    fn visit_binary(&mut self, binary: &Binary) -> T;
    fn visit_unary(&mut self, unary: &Unary) -> T;
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
            Stmt::Var(var) => self.visit_var_statement(var),
        }
    }
    fn visit_expr_statement(&mut self, expr: &Expr) -> T;
    fn visit_print_statement(&mut self, expr: &Expr) -> T;
    fn visit_block_statement(&mut self, statements: &Vec<Stmt>) -> T;
    fn visit_var_statement(&mut self, var: &Variable) -> T;
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
        let printer = printer::Printer;
        let out = printer.print(&expr);
        println!("{out}");
        assert!(out == "(* (- 123) (group 45.67))")
    }
}