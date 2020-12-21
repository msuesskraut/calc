use crate::ast::*;

use lazy_static::lazy_static;
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::Parser;

#[derive(Debug, PartialEq, Eq)]
pub enum ParserError {
    InvalidNumber(String),
    InvalidOperation(String),
    InvalidOperand(String),
    InvalidEquation(String),
    InvalidSymbol(String),
    InvalidStatement(String),
    EmptyStatement,
    MissingAssignmentTarget(String),
    MissingAssignment(String),
    MissingAssignmentEquation(String),
}

#[derive(Parser)]
#[grammar = "equation.pest"]
pub struct EquationParser;

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use Assoc::*;
        use Rule::*;

        PrecClimber::new(vec![
            Operator::new(add, Left) | Operator::new(subtract, Left),
            Operator::new(multiply, Left) | Operator::new(divide, Left) | Operator::new(rem, Left),
            Operator::new(power, Right),
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

fn parse_term(
    lhs: Result<Operand, ParserError>,
    op: Pair<Rule>,
    rhs: Result<Operand, ParserError>,
) -> Result<Operand, ParserError> {
    let lhs = lhs?;
    let rhs = rhs?;
    match op.as_rule() {
        Rule::add => Ok(new_operand_term(lhs, Operation::Add, rhs)),
        Rule::subtract => Ok(new_operand_term(lhs, Operation::Sub, rhs)),
        Rule::multiply => Ok(new_operand_term(lhs, Operation::Mul, rhs)),
        Rule::divide => Ok(new_operand_term(lhs, Operation::Div, rhs)),
        Rule::rem => Ok(new_operand_term(lhs, Operation::Rem, rhs)),
        Rule::power => Ok(new_operand_term(lhs, Operation::Pow, rhs)),
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
        parse_term,
    )
}

fn parse_assignment(assignment: Pairs<Rule>) -> Result<Assignment, ParserError> {
    let mut it = assignment;

    let sym = it.next().ok_or(ParserError::MissingAssignmentTarget(it.as_str().to_string()))?;    

    let sym = if Rule::symbol == sym.as_rule() {
        Ok(sym.as_str())
    } else {
        Err(ParserError::InvalidSymbol(sym.as_str().to_string()))
    }?;
    let sym = sym.to_string();

    let eq = parse_operand(it.next().ok_or(ParserError::MissingAssignmentEquation(it.as_str().to_string()))?.into_inner())?;
    let eq = Equation { eq };
    Ok(Assignment { sym, eq })
}

fn parse_statement(statements: Pairs<Rule>) -> Result<Statement, ParserError> {
    for statement in statements {
        return match statement.as_rule() {
            Rule::assignment => {
                let assign = parse_assignment(statement.into_inner())?;
                Ok(Statement::Assignment(assign))
            },
            Rule::equation => {
                let eq = parse_operand(statement.into_inner())?;
                Ok(Statement::Equation(Equation { eq }))
            },
            r => Err(ParserError::InvalidStatement(format!("Unexpected rule: {:?}", r))),
        };
    }

    return Err(ParserError::EmptyStatement);
}

pub fn parse(cmd: &str) -> Result<Statement, ParserError> {
    match EquationParser::parse(Rule::statement, cmd) {
        Ok(rules) => parse_statement(rules),
        Err(e) => Err(ParserError::InvalidEquation(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_number() {
        let eq = Operand::Number(12.2);
        assert_eq!(Ok(Statement::Equation(Equation { eq })), parse("12.2"));
    }

    #[test]
    fn parse_symbol() {
        let eq = Operand::Symbol("x".to_string());
        assert_eq!(Ok(Statement::Equation(Equation { eq })), parse("x"));
    }

    #[test]
    fn parse_symbol_add() {
        let term = {
            let lhs = Operand::Symbol("x".to_string());
            let rhs = Operand::Number(1.0);
            let op = Operation::Add;
            Term { op, lhs, rhs }
        };
        let eq = Operand::Term(Box::new(term));
        assert_eq!(Ok(Statement::Equation(Equation { eq })), parse("x + 1"));
    }

    #[test]
    fn parse_term_add() {
        let lhs = Operand::Number(3.0);
        let rhs = Operand::Number(-4.0);
        let op = Operation::Mul;
        let eq = Operand::Term(Box::new(Term { op, lhs, rhs }));
        assert_eq!(Ok(Statement::Equation(Equation { eq })), parse("3 * -4"));
    }

    #[test]
    fn parse_term_mul() {
        let lhs = Operand::Number(1.0);
        let rhs = Operand::Number(2.0);
        let op = Operation::Add;
        let eq = Operand::Term(Box::new(Term { op, lhs, rhs }));
        assert_eq!(Ok(Statement::Equation(Equation { eq })), parse("1 + 2"));
    }

    #[test]
    fn parse_term_precedence_add_mul() {
        let lhs = Operand::Number(1.0);
        let rhs = {
            let lhs = Operand::Number(2.0);
            let rhs = Operand::Symbol("val".to_string());
            let op = Operation::Mul;
            Operand::Term(Box::new(Term { op, lhs, rhs }))
        };
        let op = Operation::Add;
        let eq = Operand::Term(Box::new(Term { op, lhs, rhs }));
        assert_eq!(Ok(Statement::Equation(Equation { eq })), parse("1 + 2 * val"));
    }

    #[test]
    fn parse_term_precedence_sub_div_pow() {
        let lhs = Operand::Number(1.0);
        let rhs = {
            let lhs = {
                let lhs = Operand::Number(2.0);
                let rhs = Operand::Symbol("exp".to_string());
                let op = Operation::Pow;
                Operand::Term(Box::new(Term { op, lhs, rhs }))
            };
            let rhs = Operand::Symbol("val".to_string());
            let op = Operation::Mul;
            Operand::Term(Box::new(Term { op, lhs, rhs }))
        };
        let op = Operation::Add;
        let eq = Operand::Term(Box::new(Term { op, lhs, rhs }));
        assert_eq!(Ok(Statement::Equation(Equation { eq })), parse("1 + 2 ^ exp * val"));
    }

    #[test]
    fn parse_a_is_1() {
        let assign = {
            let sym = "a".to_string();
            let eq = {
                let eq = Operand::Number(1.0);
                Equation { eq }
            };
            Assignment { sym, eq }
        };
        let statement = Statement::Assignment(assign);
        assert_eq!(Ok(statement), parse("a := 1"));
    }
}
