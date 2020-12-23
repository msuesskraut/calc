use crate::ast::{ Expression, Operand, Term };
use crate::calc::{ calc_term, Env };

use lazy_static::lazy_static;

lazy_static! {
    static ref EMPTY_ENV: Env = Env::default();
}

fn simplify_term(term: &Term) -> Operand {
    let lhs = simplify_operand(&term.lhs);
    let rhs = simplify_operand(&term.rhs);
    if lhs.is_num() && rhs.is_num() {
        return Operand::Number(calc_term(term, &EMPTY_ENV).unwrap());
    }
    unimplemented!()
    // pattern:
    // - 3 * x              -> correct
    // - x * 3              -> swap
    // - 3 / x              -> not supported
    // - x / 3              -> x * (1/3)
    // - (3 * x) + 2        -> swap
    // - 2 + (3 * x)        -> correct
    // - 2 * (3 * x)        -> calc(2 * 3) * x
    // - (2 * x) * 3        -> calc(2 * 3) * x
    // - 2 / (3 * x)        -> calc(2 / 3) * x
    // - (2 * x) / 3        -> calc(2 / 3) * x
    // - (2 * x) * (3 * x)  -> not supported (2 * 3) * x^2
    // - (2 * x) / (3 * x)  -> not supported (2 / 3) & x != 0
}

fn simplify_operand(op: &Operand) -> Operand {
    match op {
        Operand::Number(_) => op.clone(),
        Operand::Symbol(_) => op.clone(),
        Operand::Term(term) => simplify_term(&*term),
    }
}

fn simplify(eq: &Expression) -> Expression {
    Expression { eq : simplify_operand(&eq.eq) }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ast::{ Statement, Operation };
    use crate::parser::parse;

    fn parse_expression(s: &str) -> Expression {
        let statement = parse(s).unwrap();
        if let Statement::Expression(expression) = statement {
            expression
        } else {
            panic!("string is not a valid expression")
        }
    }

    #[test]
    fn simplify_number() {
        let eq = Operand::Number(12.0);
        let eq = Expression { eq };
        assert_eq!(eq, simplify(&parse_expression("12")));
    }

    #[test]
    fn simplify_symbol() {
        let eq = Operand::Symbol("x".to_string());
        let eq = Expression { eq };
        assert_eq!(eq, simplify(&parse_expression("x")));
    }

    #[test]
    fn simplify_number_addition() {
        let eq = Operand::Number(3.0);
        let eq = Expression { eq };
        assert_eq!(eq, simplify(&parse_expression("1 + 2")));
    }

    #[test]
    fn simplify_number_subtraction() {
        let eq = Operand::Number(-1.0);
        let eq = Expression { eq };
        assert_eq!(eq, simplify(&parse_expression("1 - 2")));
    }

    #[test]
    #[should_panic(expected = "not implemented")]
    fn simplify_factor_x() {
        let lhs = Operand::Number(3.0);
        let rhs = Operand::Symbol("x".to_string());
        let op = Operation::Mul;
        let eq = Expression { eq: Operand::Term(Box::new(Term { op, lhs, rhs })) };
        assert_eq!(eq, simplify(&parse_expression("3 * x")));
    }
}