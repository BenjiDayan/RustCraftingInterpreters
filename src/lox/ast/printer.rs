use crate::token_type::Token;

use super::{Assign, Binary, Call, Expr, ExprVisitor, Grouping, Literal, Logical, Unary, Variable};

pub struct Printer;
impl Printer {
    pub fn print(&mut self, expr: &Expr) -> String {
        self.visit_expr(expr)
    }
    // -> "(name expr[0] expr[1] ... )"
    fn parenthesize(&mut self, name: &str, expressions: Vec<&Expr>) -> String {
        let mut s = format!("({}", name);
        for expr in expressions {
            s.push_str(&format!(" {}", &self.visit_expr(expr)));
        }
        s + ")"
    }
}

// Printer is allowed to visit expressions
impl ExprVisitor<String> for Printer {
    fn visit_assignment(&mut self, assignment: &Assign) -> String {
        self.parenthesize("assign=", vec![&Expr::Variable(assignment.name.clone()), &assignment.value])
    }

    fn visit_logical(&mut self, logical: &Logical) -> String {
        self.parenthesize(&logical.operator.lexeme, vec![&logical.left, &logical.right])
    }

    fn visit_binary(&mut self, binary: &Binary) -> String {
        self.parenthesize(&binary.operator.lexeme, vec![&binary.left, &binary.right])
    }

    fn visit_unary(&mut self, unary: &Unary) -> String {
        self.parenthesize(&unary.operator.lexeme, vec![&unary.right])
    }
    fn visit_call(&mut self, call: &Call) -> String {
        let args: Vec<&Expr> = call.arguments.iter().collect();
        self.parenthesize("func_call with args", args)
    }

    fn visit_grouping(&mut self, grouping: &Grouping) -> String {
        self.parenthesize("group", vec![&grouping.0])
    }

    fn visit_literal(&mut self, literal: &Literal) -> String {
        match literal {
            Literal::String(val) => val.clone(),
            Literal::Number(val) => val.to_string(),
            Literal::Nil => "nil".to_owned(),
            Literal::Boolean(val) => val.to_string()
        }
    }

    fn visit_variable(&mut self, token: &Token) -> String {
        format!("var:{}", token.lexeme).to_string()
    }

    fn visit_null(&mut self) -> String { "null".to_string() }
}

#[cfg(test)]
mod test {
    use crate::lox::
        ast::{Binary, Expr, Grouping, Literal, Unary};
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
            right: Box::new(Expr::Grouping(Grouping(Box::new(Expr::Literal(Literal::Number(45.67),
            ))))),
        });
        let mut printer = Printer;
        let out = printer.print(&expr);
        println!("{out}");
        assert!(out == "(* (- 123) (group 45.67))")
    }
}