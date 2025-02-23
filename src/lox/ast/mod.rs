
use crate::token_type::Token;
use crate::token_type::Literal;
use crate::token_type::Value;

pub mod parser;
pub mod printer;
pub mod interpreter_old;
pub mod interpreter;

#[derive(Debug, Clone)]
pub enum Expr {
    Binary(Binary),
    Unary(Unary),
    Grouping(Grouping),
    // LiteralExpr(LiteralExpr),
    Literal(Literal)
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
    fn visit_expr(&self, expr: &Expr) -> T;
    fn visit_binary(&self, binary: &Binary) -> T;
    fn visit_unary(&self, unary: &Unary) -> T;
    fn visit_literal(&self, literal: &Literal) -> T;
    fn visit_grouping(&self, grouping: &Grouping) -> T;
}



pub struct Printer;
impl Printer {
    pub fn print(&self, expr: &Expr) -> String {
        self.visit_expr(expr)
    }
    fn parenthesize(&self, name: &str, expressions: Vec<&Expr>) -> String {
        let mut s = format!("({}", name);
        for expr in expressions {
            s.push_str(&format!(" {}", &self.visit_expr(expr)));
        }
        s + ")"
    }
}

impl ExprVisitor<String> for Printer {
    fn visit_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Binary(binary) => self.visit_binary(binary),
            Expr::Unary(unary) => self.visit_unary(unary),
            Expr::Literal(literal) => self.visit_literal(literal),
            Expr::Grouping(grouping) => self.visit_grouping(grouping),
        }
    }

    fn visit_binary(&self, binary: &Binary) -> String {
        self.parenthesize(&binary.operator.lexeme, vec![&binary.left, &binary.right])
    }

    fn visit_unary(&self, unary: &Unary) -> String {
        self.parenthesize(&unary.operator.lexeme, vec![&unary.right])
    }

    fn visit_grouping(&self, grouping: &Grouping) -> String {
        self.parenthesize("group", vec![&grouping.0])
    }

    fn visit_literal(&self, literal: &Literal) -> String {
        match literal {
            Literal::String(val) => val.clone(),
            Literal::Number(val) => val.to_string(),
            Literal::Boolean(val) => val.to_string(),
            Literal::Nil => "nil".to_owned(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::lox::ast::{Binary, Expr, Grouping, Literal, Unary};
    use crate::token_type::{Token, TokenType};

    use super::Printer;

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
        let printer = Printer;
        let out = printer.print(&expr);
        println!("{out}");
        assert!(out == "(* (- 123) (group 45.67))")
    }
}