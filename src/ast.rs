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

#[derive(Debug, PartialEq)]
pub struct Assignment {
    pub sym: String,
    pub eq: Equation,
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Equation(Equation),
    Assignment(Assignment),
}

#[derive(Debug, Default)]
pub struct Env {
    env: HashMap<String, Number>,
}

impl Env {
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
        let env = Env::default();

        assert_eq!(None, env.get("x"));
    }

    #[test]
    fn read_env_var() {
        let mut env = Env::default();
        env.put("x".to_string(), 12.0);

        assert_eq!(Some(&12.0), env.get("x"));
    }
}
