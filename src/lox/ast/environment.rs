use std::{any::Any, collections::HashMap, iter::Map};

use crate::TokenType;

use super::{Binary, Expr, ExprVisitor, Grouping, Literal, Stmt, StmtVisitor, Token, Unary, Value, Variable};

pub struct Environment {
    pub values: HashMap<String, Value>
}


impl Environment {
    pub fn define(&mut self,name: String, value: Value) {
        self.values.insert(name, value);    
    }

    pub fn new() -> Self {
        Self {
            values: HashMap::new()
        }
    }
}