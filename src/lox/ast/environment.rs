use std::{any::Any, collections::HashMap, iter::Map};

use crate::TokenType;

use super::{interpreter::RuntimeError, Binary, Expr, ExprVisitor, Grouping, Literal, Stmt, StmtVisitor, Token, Unary, Value, Variable};
use super::{interpreter::Result};

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

    pub fn get(&self, name: &Token) -> Result<Value> {
        if self.values.contains_key(&name.lexeme) {
            return Ok(self.values.get(&name.lexeme).unwrap().clone())
        } else {
            return Err(RuntimeError{token: name.clone(), message: format!("Undefined variable '{}'", name.lexeme)})
        }
    }
}