//! Variable environment for storing and retrieving variables during evaluation.

use crate::ast::Value;
use std::collections::HashMap;

/// Environment holding variable bindings.
#[derive(Debug, Clone)]
pub struct Environment {
    variables: HashMap<String, Value>,
}

impl Environment {
    /// Create a new empty environment.
    pub fn new() -> Self {
        Environment {
            variables: HashMap::new(),
        }
    }

    /// Get a variable's value.
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    /// Set a variable's value.
    pub fn set(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    /// Check if a variable exists.
    pub fn contains(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// Remove a variable.
    pub fn remove(&mut self, name: &str) -> Option<Value> {
        self.variables.remove(name)
    }

    /// Clear all variables.
    pub fn clear(&mut self) {
        self.variables.clear();
    }

    /// Get all variable names.
    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.variables.keys().cloned().collect();
        names.sort();
        names
    }

    /// Get the number of variables.
    pub fn len(&self) -> usize {
        self.variables.len()
    }

    /// Check if the environment is empty.
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get() {
        let mut env = Environment::new();
        env.set("x", Value::Number(42.0));
        assert_eq!(env.get("x"), Some(&Value::Number(42.0)));
    }

    #[test]
    fn test_contains() {
        let mut env = Environment::new();
        env.set("x", Value::Number(1.0));
        assert!(env.contains("x"));
        assert!(!env.contains("y"));
    }

    #[test]
    fn test_remove() {
        let mut env = Environment::new();
        env.set("x", Value::Number(1.0));
        assert!(env.remove("x").is_some());
        assert!(!env.contains("x"));
    }

    #[test]
    fn test_clear() {
        let mut env = Environment::new();
        env.set("x", Value::Number(1.0));
        env.set("y", Value::Number(2.0));
        env.clear();
        assert!(env.is_empty());
    }
}
