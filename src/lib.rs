mod ast;
mod calc;
mod parser;
mod solver;

use parser::parse;

use crate::ast::{Number, Statement};
use crate::calc::{Env, CalcError, calc_expression};
use crate::parser::ParserError;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    ParserError(ParserError),
    CalcError(CalcError),
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
            Statement::Expression(eq) => Ok(Some(calc_expression(&eq, &self.env)?)),
            Statement::Assignment(assign) => {
                self.env
                    .put(assign.sym, calc_expression(&assign.eq, &self.env)?);
                Ok(None)
            }
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
