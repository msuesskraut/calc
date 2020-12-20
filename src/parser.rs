use crate::ast::*;

use lazy_static::lazy_static;
use pest::{Parser, iterators::{Pair, Pairs}};
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest_derive::Parser;

#[derive(Debug, PartialEq, Eq)]
pub enum ParserError {
    InvalidNumber(String),
    InvalidOperation(String),
    InvalidOperand(String),
    InvalidEquation(String),
}

#[derive(Parser)]
#[grammar = "equation.pest"]
pub struct EquationParser;

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use Rule::*;
        use Assoc::*;

        PrecClimber::new(vec![
            Operator::new(add, Left) | Operator::new(subtract, Left),
            Operator::new(multiply, Left) | Operator::new(divide, Left) | Operator::new(rem, Left),
            Operator::new(power, Right)
        ])
    };
}

fn parse_num(pair: Pair<Rule>) -> Result<Operand, ParserError> {
    match pair.as_str().parse::<f64>() {
        Ok(num) => Ok(Operand::Number(num)),
        Err(_) => Err(ParserError::InvalidNumber(pair.as_str().to_string())),
    }
}

fn new_operand_term(lhs: Operand, op: Operation, rhs: Operand) -> Operand {
    Operand::Term(Box::new(Term { op, lhs, rhs }))
}

fn parse_term(lhs: Result<Operand, ParserError>, op: Pair<Rule>, rhs: Result<Operand, ParserError>) -> Result<Operand, ParserError> {
    let lhs = lhs?;
    let rhs = rhs?;
    match op.as_rule() {
        Rule::add      => Ok(new_operand_term(lhs, Operation::Add, rhs)),
        Rule::subtract => Ok(new_operand_term(lhs, Operation::Sub, rhs)),
        Rule::multiply => Ok(new_operand_term(lhs, Operation::Mul, rhs)),
        Rule::divide   => Ok(new_operand_term(lhs, Operation::Div, rhs)),
        Rule::rem      => Ok(new_operand_term(lhs, Operation::Rem, rhs)),
        Rule::power    => Ok(new_operand_term(lhs, Operation::Pow, rhs)),
        _ => Err(ParserError::InvalidOperation(op.as_str().to_string())),
    }
}

fn parse_operand(expression: Pairs<Rule>) -> Result<Operand, ParserError> {
    PREC_CLIMBER.climb(
        expression,
        |pair: Pair<Rule>| match pair.as_rule() {
            Rule::num => parse_num(pair),
            Rule::expr => parse_operand(pair.into_inner()),
            Rule::symbol => Ok(Operand::Symbol(pair.as_str().to_string())),
            _ => Err(ParserError::InvalidOperand(pair.as_str().to_string())),
        },
        parse_term
    )
}

pub fn parse_equation(eq: &str) -> Result<Equation, ParserError> {
    match EquationParser::parse(Rule::equation, eq) {
        Ok(rules) => Ok(Equation { eq: parse_operand(rules)? }),
        Err(e) => Err(ParserError::InvalidEquation(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_number() {
        let eq = Operand::Number(12.0);
        assert_eq!(Ok(Equation { eq }), parse_equation("12.0"));
    }

    #[test]
    fn parse_symbol() {
        let eq = Operand::Symbol("x".to_string());
        assert_eq!(Ok(Equation { eq }), parse_equation("x"));
    }
}