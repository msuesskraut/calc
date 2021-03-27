#![allow(clippy::upper_case_acronyms)]

use crate::ast::*;

use lazy_static::lazy_static;
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::Parser;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum ParserError {
    #[error("Invalid number - expected a floating number `{0}`")]
    InvalidNumber(String),
    #[error("Invalid operation - expected +, -, *, /, %, or ^ `{0}`")]
    InvalidOperation(String),
    #[error("Invalid operand - expected variable, number or term, but got `{0}`")]
    InvalidOperand(String),
    #[error("Invalid expression - expected variable, number or term, but got `{0}`")]
    InvalidExpression(String),
    #[error("Invalid symbol - expected  `{0}`")]
    InvalidSymbol(String),
    #[error(
        "Invalid statement - expected assignment, expression, or solve statement, but got `{0}`"
    )]
    InvalidStatement(String),
    #[error("Expected statement, but got an empty line")]
    EmptyStatement,
    #[error("Missing assignment target - expected symbol, but got `{0}`")]
    MissingAssignmentTarget(String),
    #[error("Expected an assignment `:=`, but got `{0}`")]
    MissingAssignment(String),
    #[error("Expected an expression, but got `{0}`")]
    MissingAssignmentExpression(String),
    #[error("Expected expression in solve left from the `=`, but got `{0}`")]
    MissingSolveForLeftExpression(String),
    #[error("Expected expression in solve right from the `=`, but got `{0}`")]
    MissingSolveForRightExpression(String),
    #[error("Expected variable name after `for`, but got `{0}`")]
    MissingSolveForSymbol(String),
    #[error("No function name found")]
    MissingFunctionName,
    #[error("Expected expression as function body, but got nothing")]
    MissingFunctionBody,
    #[error("Expected expression as parameter value, but got `{0}`")]
    ExpectedParamExpression(String),
    #[error("Plot is missing a function name, but got nothing")]
    PlotMissingFunction,
    #[error("Expected function name, but got {0}")]
    PlotUnexpectedSymbol(String),
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

fn parse_fun_call(fun_call: Pairs<Rule>) -> Result<Operand, ParserError> {
    let mut it = fun_call;

    let name = it
        .next()
        .ok_or(ParserError::MissingFunctionName)?
        .as_str()
        .to_string();

    let mut params = Vec::new();
    for p in it {
        if p.as_rule() == Rule::expr {
            params.push(parse_operand(p.into_inner())?);
        } else {
            return Err(ParserError::ExpectedParamExpression(p.as_str().to_string()));
        }
    }
    Ok(Operand::FunCall(FunCall { name, params }))
}

fn parse_operand(expression: Pairs<Rule>) -> Result<Operand, ParserError> {
    PREC_CLIMBER.climb(
        expression,
        |pair: Pair<Rule>| match pair.as_rule() {
            Rule::num => parse_num(pair),
            Rule::expr => parse_operand(pair.into_inner()),
            Rule::symbol => Ok(Operand::Symbol(pair.as_str().to_string())),
            Rule::fun_call => parse_fun_call(pair.into_inner()),
            _ => Err(ParserError::InvalidOperand(pair.as_str().to_string())),
        },
        parse_term,
    )
}

fn parse_assignment(assignment: Pairs<Rule>) -> Result<Statement, ParserError> {
    let mut it = assignment;

    let sym = it
        .next()
        .ok_or_else(|| ParserError::MissingAssignmentTarget(it.as_str().to_string()))?;

    let sym = if Rule::symbol == sym.as_rule() {
        Ok(sym.as_str())
    } else {
        Err(ParserError::InvalidSymbol(sym.as_str().to_string()))
    }?;
    let sym = sym.to_string();

    let op = parse_operand(
        it.next()
            .ok_or_else(|| ParserError::MissingAssignmentExpression(it.as_str().to_string()))?
            .into_inner(),
    )?;
    Ok(Statement::Assignment { sym, op })
}

fn parse_solve_for(solve_for: Pairs<Rule>) -> Result<Statement, ParserError> {
    let mut it = solve_for;

    let lhs = parse_operand(
        it.next()
            .ok_or_else(|| ParserError::MissingSolveForLeftExpression(it.as_str().to_string()))?
            .into_inner(),
    )?;
    let rhs = parse_operand(
        it.next()
            .ok_or_else(|| ParserError::MissingSolveForRightExpression(it.as_str().to_string()))?
            .into_inner(),
    )?;
    let sym = it
        .next()
        .ok_or_else(|| ParserError::MissingSolveForSymbol(it.as_str().to_string()))?;
    let sym = if Rule::symbol == sym.as_rule() {
        Ok(sym.as_str())
    } else {
        Err(ParserError::InvalidSymbol(sym.as_str().to_string()))
    }?;
    let sym = sym.to_string();

    Ok(Statement::SolveFor { lhs, rhs, sym })
}

fn parse_function(function: Pairs<Rule>) -> Result<Statement, ParserError> {
    let mut it = function;

    let name = it
        .next()
        .ok_or(ParserError::MissingFunctionName)?
        .as_str()
        .to_string();

    let mut args = Vec::new();
    for p in it {
        if p.as_rule() == Rule::symbol {
            args.push(p.as_str().to_string());
        } else {
            let body = parse_operand(p.into_inner())?;
            return Ok(Statement::Function {
                name,
                fun: Function::Custom(CustomFunction { args, body }),
            });
        }
    }

    Err(ParserError::MissingFunctionBody)
}

fn parse_plot(plot: Pairs<Rule>) -> Result<Statement, ParserError> {
    let mut it = plot;
    let fun = it.next().ok_or(ParserError::PlotMissingFunction)?;
    match fun.as_rule() {
        Rule::symbol => Ok(Statement::Plot {
            name: fun.as_str().to_string(),
        }),
        _ => Err(ParserError::PlotUnexpectedSymbol(fun.as_str().to_string())),
    }
}

fn parse_statement(statements: Pairs<Rule>) -> Result<Statement, ParserError> {
    let mut it = statements;
    let statement = it.next().ok_or(ParserError::EmptyStatement)?;
    match statement.as_rule() {
        Rule::assignment => parse_assignment(statement.into_inner()),
        Rule::expr => Ok(Statement::Expression {
            op: parse_operand(Pairs::single(statement))?,
        }),
        Rule::solvefor => parse_solve_for(statement.into_inner()),
        Rule::function => parse_function(statement.into_inner()),
        Rule::plot => parse_plot(statement.into_inner()),
        r => Err(ParserError::InvalidStatement(format!(
            "Unexpected rule: {:?}",
            r
        ))),
    }
}

pub fn parse(cmd: &str) -> Result<Statement, ParserError> {
    match EquationParser::parse(Rule::statement, cmd) {
        Ok(rules) => parse_statement(rules),
        Err(e) => Err(ParserError::InvalidExpression(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_number() {
        let op = Operand::Number(12.2);
        assert_eq!(Ok(Statement::Expression { op }), parse("12.2"));
    }

    #[test]
    fn parse_symbol() {
        let op = Operand::Symbol("x".to_string());
        assert_eq!(Ok(Statement::Expression { op }), parse("x"));
    }

    #[test]
    fn parse_symbol_add() {
        let term = {
            let lhs = Operand::Symbol("x".to_string());
            let rhs = Operand::Number(1.0);
            let op = Operation::Add;
            Term { op, lhs, rhs }
        };
        let op = Operand::Term(Box::new(term));
        assert_eq!(Ok(Statement::Expression { op }), parse("x + 1"));
    }

    #[test]
    fn parse_term_add() {
        let lhs = Operand::Number(3.0);
        let rhs = Operand::Number(-4.0);
        let op = Operation::Mul;
        let op = Operand::Term(Box::new(Term { op, lhs, rhs }));
        assert_eq!(Ok(Statement::Expression { op }), parse("3 * -4"));
    }

    #[test]
    fn parse_term_mul() {
        let lhs = Operand::Number(1.0);
        let rhs = Operand::Number(2.0);
        let op = Operation::Add;
        let op = Operand::Term(Box::new(Term { op, lhs, rhs }));
        assert_eq!(Ok(Statement::Expression { op }), parse("1 + 2"));
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
        let op = Operand::Term(Box::new(Term { op, lhs, rhs }));
        assert_eq!(Ok(Statement::Expression { op }), parse("1 + 2 * val"));
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
        let op = Operand::Term(Box::new(Term { op, lhs, rhs }));
        assert_eq!(Ok(Statement::Expression { op }), parse("1 + 2 ^ exp * val"));
    }

    #[test]
    fn parse_a_is_1() {
        let statement = Statement::Assignment {
            sym: "a".to_string(),
            op: Operand::Number(1.0),
        };
        assert_eq!(Ok(statement), parse("a := 1"));
    }

    #[test]
    fn parse_solve_for() {
        let statement = Statement::SolveFor {
            lhs: Operand::Number(13.0),
            rhs: Operand::Symbol("x".to_string()),
            sym: "x".to_string(),
        };
        assert_eq!(Ok(statement), parse("solve 13 = x for x"));
    }

    #[test]
    fn parse_fun_no_args() {
        let fun = Function::Custom(CustomFunction {
            args: Vec::new(),
            body: Operand::Number(12.0),
        });
        let statement = Statement::Function {
            name: "ghs".to_string(),
            fun,
        };
        assert_eq!(Ok(statement), parse("ghs() := 12"));
    }

    #[test]
    fn parse_fun_x() {
        let fun = Function::Custom(CustomFunction {
            args: vec!["x".to_string()],
            body: {
                let lhs = Operand::Number(1.0);
                let rhs = Operand::Symbol("x".to_string());
                let op = Operation::Add;
                Operand::Term(Box::new(Term { lhs, rhs, op }))
            },
        });
        let statement = Statement::Function {
            name: "f".to_string(),
            fun,
        };
        assert_eq!(Ok(statement), parse("f(x) := 1 + x"));
    }

    #[test]
    fn parse_fun_call_without_params() {
        let fun_call = FunCall {
            name: "fun".to_string(),
            params: Vec::new(),
        };
        let op = Operand::FunCall(fun_call);
        let stat = Statement::Expression { op };
        assert_eq!(Ok(stat), parse("fun()"));
    }

    #[test]
    fn parse_fun_call_with_symbol() {
        let fun_call = FunCall {
            name: "fun".to_string(),
            params: vec![Operand::Symbol("x".to_string())],
        };
        let op = Operand::FunCall(fun_call);
        let stat = Statement::Expression { op };
        assert_eq!(Ok(stat), parse("fun(x)"));
    }

    #[test]
    fn parse_fun_call_with_number() {
        let fun_call = FunCall {
            name: "fun".to_string(),
            params: vec![Operand::Number(42.0)],
        };
        let op = Operand::FunCall(fun_call);
        let stat = Statement::Expression { op };
        assert_eq!(Ok(stat), parse("fun(42)"));
    }

    #[test]
    fn parse_plot() {
        let stat = Statement::Plot {
            name: "fun".to_string(),
        };
        assert_eq!(Ok(stat), parse("plot fun"));
    }
}
