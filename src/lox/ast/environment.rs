use std::{collections::HashMap, sync::Arc, sync::Mutex};

use crate::TokenType;

use super::{interpreter::RuntimeError, Binary, Expr, ExprVisitor, Grouping, Literal, Stmt, StmtVisitor, Token, Unary, Value, Variable};
use super::interpreter::Result;

pub struct Environment {
    values: HashMap<String, Value>,
    enclosing_env: Option<Arc<Mutex<Environment>>>
}

impl Environment {
    pub fn new(enclosing_env: Option<Arc<Mutex<Environment>>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing_env
        }
    }

    pub fn define(&mut self, name: &String, value: &Value) {
        self.values.insert(name.clone(), value.clone());    
    }

    pub fn assign(&mut self, name: &Token, value: &Value) -> Result<bool> {
        if self.values.contains_key(&name.lexeme) {
            self.define(&name.lexeme, value);
            Ok(true)
        } else if let Some(ref enclosing_env) = self.enclosing_env {
            let mut env = enclosing_env.lock().unwrap();
            env.assign(name, value)
        } else {
            Err(RuntimeError{token: name.clone(), message: format!("Can't assign to undefined variable '{}'", name.lexeme)})
        }
    }

    pub fn get(&self, name: &Token) -> Result<Value> {
        if self.values.contains_key(&name.lexeme) {
            Ok(self.values.get(&name.lexeme).unwrap().clone())
        } else if let Some(ref enclosing_env) = self.enclosing_env {
            let env = enclosing_env.lock().unwrap();
            env.get(name)
        } else {
            Err(RuntimeError{token: name.clone(), message: format!("Undefined variable '{}'", name.lexeme)})
        }
    }
}