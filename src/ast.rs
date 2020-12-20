use std::collections::HashMap;

pub type Number = f64;

#[derive(Debug, PartialEq)]
pub enum Operand {
    Number(Number),
    Symbol(String),
    Term(Box<Term>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
}

#[derive(Debug, PartialEq)]
pub struct Term {
    pub op: Operation,
    pub lhs: Operand,
    pub rhs: Operand,
}

#[derive(Debug, PartialEq)]
pub struct Equation {
    pub eq: Operand,
}

#[derive(Debug, Default)]
pub struct Env {
    env: HashMap<String, Number>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
        }
    }

    pub fn get(&self, sym: &str) -> Option<&Number> {
        self.env.get(sym)
    }

    pub fn put(&mut self, sym: String, num: Number) {
        self.env.insert(sym, num);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_env_empty() {
        let env = Env::new();

        assert_eq!(None, env.get("x"));
    }

    #[test]
    fn read_env_var() {
        let mut env = Env::new();
        env.put("x".to_string(), 12.0);

        assert_eq!(Some(&12.0), env.get("x"));
    }
}
