use std::{any::Any, collections::HashMap, iter::Map};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc, sync::Mutex};

use crate::TokenType;

use super::{interpreter::RuntimeError, Binary, Expr, ExprVisitor, Grouping, Literal, Stmt, StmtVisitor, Token, Unary, Value, Variable};
use super::{interpreter::Result};

pub struct Environment {
    pub values: HashMap<String, Value>,
    pub enclosing_env: Option<Arc<Mutex<Environment>>>
}


impl Environment {
    pub fn define(&mut self, name: &String, value: &Value) {
        self.values.insert(name.clone(), value.clone());    
    }

    pub fn assign(&mut self, name: &Token, value: &Value) -> Result<bool> {
        if self.values.contains_key(&name.lexeme) {
            self.define(&name.lexeme, value);
            Ok(true)
        } else if let Some(ref mut enclosing_env) = self.enclosing_env {
            let mut env = enclosing_env.lock().unwrap();
            env.assign(name, value)
        } else {
            Err(RuntimeError{token: name.clone(), message: format!("Can't assign to undefined variable '{}'", name.lexeme)})
        }
    }

    pub fn new(enclosing_env: Option<Arc<Mutex<Environment>>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing_env: enclosing_env.map(|env| {
                let guard = env.lock().unwrap();
                Arc::new(Mutex::new(guard))
            })
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