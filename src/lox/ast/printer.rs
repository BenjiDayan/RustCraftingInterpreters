use super::{Binary, Expr, ExprVisitor, Grouping, Literal, Unary};

pub struct Printer;
impl Printer {
    pub fn print(&self, expr: &Expr) -> String {
        self.visit_expr(expr)
    }
    // -> "(name expr[0] expr[1] ... )"
    fn parenthesize(&self, name: &str, expressions: Vec<&Expr>) -> String {
        let mut s = format!("({}", name);
        for expr in expressions {
            s.push_str(&format!(" {}", &self.visit_expr(expr)));
        }
        s + ")"
    }
}

// Printer is allowed to visit expressions
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
            Literal::Nil => "nil".to_owned(),
            Literal::Boolean(val) => val.to_string()
        }
    }
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
        let printer = Printer;
        let out = printer.print(&expr);
        println!("{out}");
        assert!(out == "(* (- 123) (group 45.67))")
    }
}