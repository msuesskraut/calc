mod ast;
mod calc;
mod parser;

use parser::parse_equation;

pub use crate::ast::{ Env, Number };
pub use crate::parser::ParserError;
pub use crate::calc::CalcError;

use crate::calc::calc_equation;

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

    pub fn assign(&mut self, sym: &str, eq: &str) -> Result<(), Error> {
        let ast = parse_equation(eq)?;
        let res = calc_equation(ast, &self.env)?;
        self.env.put(sym.to_string(), res);
        Ok(())
    }

    pub fn read(&self, sym: &str) -> Option<&Number> {
        self.env.get(sym)
    }

    pub fn calc(&self, eq: &str) -> Result<Number, Error> {
        let eq = parse_equation(eq)?;
        let res = calc_equation(eq, &self.env)?;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculator_assign() {
        let mut c = Calculator::new();
        c.assign("x", "3").unwrap();
        assert_eq!(Some(&3.0), c.read("x"));
    }

    #[test]
    fn calculator_calc() {
        let c = Calculator::new();
        assert_eq!(3.0, c.calc("1 + 2").unwrap());
    }
}