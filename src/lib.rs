mod ast;
mod calc;
mod parser;
mod solver;

use crate::ast::{Number, Statement};
use crate::calc::{calc_operand, CalcError, Env};
use crate::parser::{parse, ParserError};
use crate::solver::{solve_for, SolverError};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    ParserError(ParserError),
    CalcError(CalcError),
    SolverError(SolverError),
}

impl From<ParserError> for Error {
    fn from(err: ParserError) -> Self {
        Error::ParserError(err)
    }
}

impl From<CalcError> for Error {
    fn from(err: CalcError) -> Self {
        Error::CalcError(err)
    }
}

impl From<SolverError> for Error {
    fn from(err: SolverError) -> Self {
        Error::SolverError(err)
    }
}

#[derive(Debug, Default)]
pub struct Calculator {
    env: Env,
}

impl Calculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute(&mut self, line: &str) -> Result<Option<Number>, Error> {
        let st = parse(line)?;
        match st {
            Statement::Expression { op } => Ok(Some(calc_operand(&op, &self.env)?)),
            Statement::Assignment { sym, op } => {
                self.env.put(sym, calc_operand(&op, &self.env)?);
                Ok(None)
            }
            Statement::SolveFor { lhs, rhs, sym } => Ok(Some(solve_for(&lhs, &rhs, &sym)?)),
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
}
