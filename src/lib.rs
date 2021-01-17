mod ast;
mod calc;
mod graph;
mod parser;
mod solver;

pub use crate::ast::Number;
use crate::ast::Statement;
use crate::calc::{calc_operand, CalcError, TopLevelEnv};
use crate::graph::GraphError;
pub use crate::graph::{Area, Graph, Range};
use crate::parser::{parse, ParserError};
use crate::solver::{solve_for, SolverError};

use thiserror::Error;

/// Calculator error
#[derive(Debug, PartialEq, Eq, Error)]
pub enum Error {
    /// errors derived from parsers
    #[error(transparent)]
    ParserError(#[from] ParserError),
    /// errors derived from calculator
    #[error(transparent)]
    CalcError(#[from] CalcError),
    /// errors derived from solver
    #[error(transparent)]
    SolverError(#[from] SolverError),
    /// errors derived from graph
    #[error(transparent)]
    GraphError(#[from] GraphError),
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Void,
    Number(Number),
    Solved { variable: String, value: Number },
    Graph(Graph),
}

/// # Calculator
///
/// See it in action on [https://msuesskraut.github.io/calc/index.html](https://msuesskraut.github.io/calc/index.html).
/// Further examples are in [`Calculator::execute`].
#[derive(Debug, Default)]
pub struct Calculator {
    env: TopLevelEnv,
}

impl Calculator {
    /// constructs an calculator without any known variables
    pub fn new() -> Self {
        Self::default()
    }

    /// Executes a command line.
    /// These kinds of statements are supported:
    /// - Expression:
    ///   ```
    ///   use rust_expression::{Calculator, Value};
    ///   let mut c = Calculator::new();
    ///   assert_eq!(Ok(Value::Number(3.0)), c.execute("1 + 2"));
    ///   ```
    /// - Variable assignments:
    ///   ```
    ///   # use rust_expression::{Calculator, Value};
    ///   # let mut c = Calculator::new();
    ///   assert_eq!(Ok(Value::Void), c.execute("a := 6"));
    ///   assert_eq!(Ok(Value::Number(36.0)), c.execute("a ^ 2"));
    ///   ```
    /// - Solving linear expressions:
    ///   ```
    ///   # use rust_expression::{Calculator, Value};
    ///   # let mut c = Calculator::new();
    ///   # c.execute("a := 6");
    ///   assert_eq!(Ok(Value::Solved {variable: "x".to_string(), value: 4.0}), c.execute("solve 3 * x - 2 = x + a for x"));
    ///   ```
    /// - Function definition:
    ///   ```
    ///   # use rust_expression::{Calculator, Value};
    ///   # let mut c = Calculator::new();
    ///   # c.execute("a := 6");
    ///   assert_eq!(Ok(Value::Void), c.execute("fun(x, y) := y - x"));
    ///   assert_eq!(Ok(Value::Number(2.0)), c.execute("fun(1.5 * 2, 3 + a) - 4"));
    ///   ```
    /// - Create a plot:
    ///   ```
    ///   # use rust_expression::{Calculator, Value};
    ///   # use rust_expression::Area;
    ///   # let mut c = Calculator::new();
    ///   assert_eq!(Ok(Value::Void), c.execute("f(x) := x ^ 2"));
    ///
    ///   match c.execute("plot f") {
    ///       Ok(Value::Graph(graph)) => {
    ///           let area = Area::new(-100., -100., 100., 100.);
    ///           let screen = Area::new(0., 0., 60., 40.);
    ///           let plot = graph.plot(&area, &screen).unwrap();
    ///           assert_eq!(Some(20.), plot.points[30]);
    ///       }
    ///       // ...
    ///   #   _ => unimplemented!(),
    ///   }
    ///   ```
    pub fn execute(&mut self, line: &str) -> Result<Value, Error> {
        let st = parse(line)?;
        match st {
            Statement::Expression { op } => Ok(Value::Number(calc_operand(&op, &self.env)?)),
            Statement::Assignment { sym, op } => {
                self.env.put(sym, calc_operand(&op, &self.env)?);
                Ok(Value::Void)
            }
            Statement::SolveFor { lhs, rhs, sym } => Ok(Value::Solved {
                variable: sym.to_string(),
                value: solve_for(&lhs, &rhs, &sym, &self.env)?,
            }),
            Statement::Function { name, fun } => {
                self.env.put_fun(name, fun);
                Ok(Value::Void)
            }
            Statement::Plot { name } => Ok(Value::Graph(Graph::new(
                &name,
                &self.env
            )?)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_calc() {
        let mut calc = Calculator::new();
        assert_eq!(Ok(Value::Number(3.0)), calc.execute("1 + 2"));
    }

    #[test]
    fn simple_assign() {
        let mut calc = Calculator::new();
        assert_eq!(Ok(Value::Void), calc.execute("a := 1"));
        assert_eq!(Ok(Value::Number(1.0)), calc.execute("a"));
    }

    #[test]
    fn simple_function() {
        let mut calc = Calculator::new();
        assert_eq!(Ok(Value::Void), calc.execute("fun(x, y) := y - x"));
        assert_eq!(
            Ok(Value::Number(20.0)),
            calc.execute("fun(1 + 2, 3 * 9) - 4")
        );
    }

    #[test]
    fn simple_solve_for() {
        let mut calc = Calculator::new();
        assert_eq!(
            Ok(Value::Solved {
                variable: "y".to_string(),
                value: 4.0
            }),
            calc.execute("solve 3 * y - 2 = y + 6 for y")
        );
    }

    #[test]
    fn simple_plot() {
        let mut calc = Calculator::new();
        assert_eq!(Ok(Value::Void), calc.execute("f(x) := x ^ 2"));
        let graph = calc.execute("plot f").unwrap();
        assert!(matches!(&graph, Value::Graph(_)));
        if let Value::Graph(graph) = graph {
            let plot = graph.plot(&Area::new(-100., -100., 100., 100.), &Area::new(0., 0., 80., 30.)).unwrap();
            assert!(!plot.points.is_empty());
        }
    }
}
