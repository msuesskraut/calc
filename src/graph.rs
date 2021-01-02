use crate::calc::{calc_operand, Env};
use crate::{
    ast::{Function, Number},
    calc::TopLevelEnv,
};

use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum GraphError {
    #[error("Unknown function `{0}` to plot")]
    UnknownFunction(String),
}

struct ArgEnv<'a> {
    name: &'a str,
    value: Number,
    env: &'a dyn Env,
}

impl<'a> Env for ArgEnv<'a> {
    fn get(&self, sym: &str) -> Option<&Number> {
        if sym == self.name {
            Some(&self.value)
        } else {
            self.env.get(sym)
        }
    }

    fn get_fun(&self, fun: &str) -> Option<&Function> {
        self.env.get_fun(fun)
    }
}

#[derive(Debug, PartialEq)]
pub struct Graph {
    fun: Function,
}

impl Graph {
    pub fn x_name(&self) -> &str {
        &self.fun.args[0]
    }

    pub fn calc(&self, x: Number, env: &dyn Env) -> Option<Number> {
        let call_env = ArgEnv {
            name: self.x_name(),
            value: x,
            env,
        };
        calc_operand(&self.fun.body, &call_env).ok()
    }
}

#[derive(Debug, PartialEq)]
pub struct Range {
    pub min: Number,
    pub max: Number,
}

impl Range {
    pub fn new(min: Number, max: Number) -> Range {
        Range { min, max }
    }
}

#[derive(Debug, PartialEq)]
pub struct Plot {
    pub x_range: Range,
    pub y_range: Range,
    pub graph: Graph,
}

impl Plot {
    pub fn new(name: &str, env: &TopLevelEnv) -> Result<Plot, GraphError> {
        let fun = env
            .get_fun(name)
            .ok_or_else(|| GraphError::UnknownFunction(name.to_string()))?;
        let x_range = Range::new(-100., 100.);
        let y_range = Range::new(-100., 100.);
        Ok(Plot {
            x_range,
            y_range,
            graph: Graph {
                fun: fun.clone(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Operand, Operation, Term};
    use crate::calc::TopLevelEnv;

    #[test]
    fn function_arg_x() {
        let mut env = TopLevelEnv::default();
        env.put("x".to_string(), -19.0);
        let name = "x";
        let value = 42.0;
        let env = ArgEnv {
            name,
            value,
            env: &env,
        };
        assert_eq!(Some(&42.0), env.get("x"));
    }

    #[test]
    fn function_arg_y() {
        let mut env = TopLevelEnv::default();
        env.put("y".to_string(), -19.0);
        let name = "x";
        let value = 42.0;
        let env = ArgEnv {
            name,
            value,
            env: &env,
        };
        assert_eq!(Some(&-19.0), env.get("y"));
    }

    #[test]
    fn function_call() {
        let fun = Function {
            args: vec!["x".to_string()],
            body: Operand::Symbol("x".to_string()),
        };
        let env = TopLevelEnv::default();
        let graph = Graph { fun };
        assert_eq!(Some(1.0), graph.calc(1.0, &env));
    }

    #[test]
    fn construct_plot() {
        let mut env = TopLevelEnv::default();
        let term = {
            let lhs = Operand::Symbol("x".to_string());
            let rhs = Operand::Number(2.0);
            let op = Operation::Pow;
            Term { lhs, rhs, op }
        };
        let body = Operand::Term(Box::new(term));
        let fun = Function {
            args: vec!["x".to_string()],
            body,
        };
        env.put_fun("f".to_string(), fun);
        let plot = Plot::new("f", &env).unwrap();
        assert_eq!(Some(4.0), plot.graph.calc(2.0, &env));
    }
}
