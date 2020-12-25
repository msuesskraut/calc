use crate::ast::{ Expression, Operand, Term, Number };
use crate::calc::{ calc_term, Env };

use lazy_static::lazy_static;

lazy_static! {
    static ref EMPTY_ENV: Env = Env::default();
}

/// Normalized form of a any operand
/// `factor * x + summand`#
#[derive(Debug, PartialEq)]
struct NormForm {
    factor: Number,
    summand: Number,
}

#[derive(Debug, PartialEq)]
enum SolverError {
    UnknownSymbol(String),
}

fn normalize_term(term: &Term) -> Result<NormForm, SolverError> {
    unimplemented!()
}

fn normalize_operand(op: &Operand, sym: &str) -> Result<NormForm, SolverError> {
    match op {
        Operand::Number(num) => Ok(NormForm { factor: 0.0, summand: *num }),
        Operand::Symbol(s) => if s == sym {
            Ok(NormForm { factor: 1.0, summand: 0.0 })
        } else {
            Err(SolverError::UnknownSymbol(s.clone()))
        },
        Operand::Term(term) => normalize_term(&*term)
    }
}

fn normalize(eq: &Expression, sym: &str) -> Result<NormForm, SolverError> {
    normalize_operand(&eq.eq, sym)
}

#[cfg(test)]
mod tests {
    mod helpers {
        use crate::ast::{ Statement, Expression, Operand };
        use crate::parser::parse;

        pub fn parse_expression(s: &str) -> Expression {
            let statement = parse(s).unwrap();
            if let Statement::Expression(expression) = statement {
                expression
            } else {
                panic!("string is not a valid expression")
            }
        }

        #[test]
        fn parse_expression_success() {
            let eq = Operand::Number(1.0);
            assert_eq!(Expression { eq }, parse_expression("1"))
        }

        #[test]
        #[should_panic(expected = "string is not a valid expression")]
        fn parse_expression_failed_assignment() {
            parse_expression("x:=1");
        }

        #[test]
        #[should_panic(expected = "InvalidExpression")]
        fn parse_expression_failed_equation() {
            parse_expression("1 @");
        }
    }
    use super::*;
    use self::helpers::parse_expression;

    #[test]
    fn normalize_operand_number() {
        let exp = NormForm { factor: 0f64, summand: 1.2 };
        assert_eq!(exp, normalize(&parse_expression("1.2"), "x").unwrap());   
    }

    #[test]
    fn normalize_operand_symbol_x() {
        let exp = NormForm { factor: 1f64, summand: 0f64 };
        assert_eq!(exp, normalize(&parse_expression("x"), "x").unwrap());   
    }

    #[test]
    fn normalize_operand_symbol_y() {
        let act = normalize(&parse_expression("y"), "x");
        assert!(matches!(act, Err(SolverError::UnknownSymbol(s)) if s == "y"));
    }

    #[test]
    fn normalize_operand_simple_add() {
        let exp = NormForm { factor: 1f64, summand: 1f64 };
        assert_eq!(exp, normalize(&parse_expression("x + 1"), "x").unwrap());   
    }

    #[test]
    fn normalize_operand_simple_mul() {
        let exp = NormForm { factor: 2f64, summand: 0f64 };
        assert_eq!(exp, normalize(&parse_expression("x * 2"), "x").unwrap());   
    }

    #[test]
    fn normalize_operand_simple_norm_form() {
        let exp = NormForm { factor: 3f64, summand: 2f64 };
        assert_eq!(exp, normalize(&parse_expression("3 * x + 2"), "x").unwrap());   
    }

    #[test]
    fn normalize_operand_simple_norm_sub() {
        let exp = NormForm { factor: 3f64, summand: -2f64 };
        assert_eq!(exp, normalize(&parse_expression("3 * x - 2"), "x").unwrap());   
    }
}