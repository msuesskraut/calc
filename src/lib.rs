mod ast;
mod calc;
mod graph;
mod parser;
mod solver;

use crate::ast::{Number, Statement};
use crate::calc::{calc_operand, CalcError, TopLevelEnv};
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
    /// 3 kinds of statements are supported:
    /// - Expression:
    ///   ```
    ///   use rust_expression::Calculator;
    ///   let mut c = Calculator::new();
    ///   assert_eq!(Ok(Some(3.0)), c.execute("1 + 2"));
    ///   ```
    /// - Variable assignments:
    ///   ```
    ///   # use rust_expression::Calculator;
    ///   # let mut c = Calculator::new();
    ///   assert_eq!(Ok(None), c.execute("a := 6"));
    ///   assert_eq!(Ok(Some(36.0)), c.execute("a ^ 2"));
    ///   ```
    /// - Solving linear expressions:
    ///   ```
    ///   # use rust_expression::Calculator;
    ///   # let mut c = Calculator::new();
    ///   # c.execute("a := 6");
    ///   assert_eq!(Ok(Some(4.0)), c.execute("solve 3 * x - 2 = x + a for x"));
    ///   ```
    /// - Function definition:
    ///   ```
    ///   # use rust_expression::Calculator;
    ///   # let mut c = Calculator::new();
    ///   assert_eq!(Ok(None), c.execute("fun(x, y) := y - x"));
    ///   assert_eq!(Ok(Some(20.0)), c.execute("fun(1 + 2, 3 * 9) - 4"));
    ///   ```
    pub fn execute(&mut self, line: &str) -> Result<Option<Number>, Error> {
        let st = parse(line)?;
        match st {
            Statement::Expression { op } => Ok(Some(calc_operand(&op, &self.env)?)),
            Statement::Assignment { sym, op } => {
                self.env.put(sym, calc_operand(&op, &self.env)?);
                Ok(None)
            }
            Statement::SolveFor { lhs, rhs, sym } => {
                Ok(Some(solve_for(&lhs, &rhs, &sym, &self.env)?))
            }
            Statement::Function { name, fun } => {
                self.env.put_fun(name, fun);
                Ok(None)
            }
            Statement::Plot { name } => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_calc() {
        let mut calc = Calculator::new();
        assert_eq!(Ok(Some(3.0)), calc.execute("1 + 2"));
    }

    #[test]
    fn simple_assign() {
        let mut calc = Calculator::new();
        assert_eq!(Ok(None), calc.execute("a := 1"));
        assert_eq!(Ok(Some(1.0)), calc.execute("a"));
    }

    #[test]
    fn simple_function() {
        let mut calc = Calculator::new();
        assert_eq!(Ok(None), calc.execute("fun(x, y) := y - x"));
        assert_eq!(Ok(Some(20.0)), calc.execute("fun(1 + 2, 3 * 9) - 4"));
    }
}
