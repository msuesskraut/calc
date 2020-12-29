use crate::ast::*;
use crate::calc::{CalcError, Env, calc_function_call};

use thiserror::Error;

/// Normalized form of a any operand
/// `factor * x + summand`#
#[derive(Debug, PartialEq)]
struct NormForm {
    a1: Number,
    a0: Number,
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum SolverError {
    #[error("Unknown variable `{0}` in `solve ... for ...`")]
    UnknownVariable(String),
    #[error("Unsupported `^2` of variable to solve for in `solve ... for ...`")]
    UnsupportedXSquare,
    #[error("Unsupported variable in denominator in `solve ... for ...`")]
    UnsupportedXDenominator,
    #[error("Unsupported % with solve for variable in `solve ... for ...`")]
    UnsupportedRemainder,
    #[error("Unsupported power in `solve ... for ...`")]
    UnsupportedPower,
    #[error("`solve ... for ...` contains no variable (after simplification)")]
    NoVariable,
    #[error(transparent)]
    FunctionCallError(#[from] CalcError),
}

fn normalize_term(term: &Term, sym: &str, env: &dyn Env) -> Result<NormForm, SolverError> {
    let lhs = normalize(&term.lhs, sym, env)?;
    let rhs = normalize(&term.rhs, sym, env)?;
    match term.op {
        Operation::Add => Ok({
            let factor = lhs.a1 + rhs.a1;
            let summand = lhs.a0 + rhs.a0;
            NormForm {
                a1: factor,
                a0: summand,
            }
        }),
        Operation::Sub => Ok({
            let factor = lhs.a1 - rhs.a1;
            let summand = lhs.a0 - rhs.a0;
            NormForm {
                a1: factor,
                a0: summand,
            }
        }),
        Operation::Mul => {
            let a2 = lhs.a1 * rhs.a1;
            let a1 = lhs.a1 * rhs.a0 + rhs.a1 * lhs.a0;
            let a0 = lhs.a0 * rhs.a0;
            if a2 != 0.0 {
                Err(SolverError::UnsupportedXSquare)
            } else {
                Ok(NormForm { a1, a0 })
            }
        }
        Operation::Div => {
            if rhs.a1 != 0.0 {
                Err(SolverError::UnsupportedXDenominator)
            } else {
                let a1 = lhs.a1 / rhs.a0;
                let a0 = lhs.a0 / rhs.a0;
                Ok(NormForm { a1, a0 })
            }
        }
        Operation::Rem => {
            if (lhs.a1 != 0.0) || (rhs.a1 != 0.0) {
                Err(SolverError::UnsupportedRemainder)
            } else {
                Ok(NormForm {
                    a1: 0.0,
                    a0: (lhs.a0 % rhs.a0),
                })
            }
        }
        Operation::Pow => {
            if (lhs.a1 != 0.0) || (rhs.a1 != 0.0) {
                Err(SolverError::UnsupportedPower)
            } else {
                Ok(NormForm {
                    a1: 0.0,
                    a0: (lhs.a0.powf(rhs.a0)),
                })
            }
        }
    }
}

fn normalize(op: &Operand, sym: &str, env: &dyn Env) -> Result<NormForm, SolverError> {
    match op {
        Operand::Number(num) => Ok(NormForm { a1: 0.0, a0: *num }),
        Operand::Symbol(s) => {
            if op.is_symbol(sym) {
                Ok(NormForm { a1: 1.0, a0: 0.0 })
            } else {
                let num = env.get(s).ok_or_else(|| SolverError::UnknownVariable(s.clone()))?;
                Ok(NormForm { a1: 0.0, a0: *num })
            }
        }
        Operand::Term(term) => normalize_term(&*term, sym, env),
        Operand::FunCall(fun_call) => {
            let num = calc_function_call(fun_call, env)?;
            Ok(NormForm { a1: 0.0, a0: num })
        },
    }
}

pub fn solve_for(lhs: &Operand, rhs: &Operand, sym: &str, env: &dyn Env) -> Result<Number, SolverError> {
    let norm_form_lhs = normalize(lhs, sym, env)?;
    let norm_form_rhs = normalize(rhs, sym, env)?;
    let denominator = norm_form_lhs.a1 - norm_form_rhs.a1;
    if 0.0 == denominator {
        Err(SolverError::NoVariable)
    } else {
        let nominator = norm_form_rhs.a0 - norm_form_lhs.a0;
        Ok(nominator / denominator)
    }
}

#[cfg(test)]
mod tests {
    mod helpers {
        use crate::ast::{Operand, Statement};
        use crate::parser::parse;

        pub fn parse_expression(s: &str) -> Operand {
            let statement = parse(s).unwrap();
            if let Statement::Expression { op } = statement {
                op
            } else {
                panic!("string is not a valid expression")
            }
        }

        #[test]
        fn parse_expression_success() {
            assert_eq!(Operand::Number(1.0), parse_expression("1"));
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
    use self::helpers::parse_expression;
    use super::*;
    use crate::parse;
    use crate::calc::TopLevelEnv;

    #[test]
    fn normalize_operand_number() {
        let exp = NormForm { a1: 0f64, a0: 1.2 };
        assert_eq!(exp, normalize(&parse_expression("1.2"), "x", &TopLevelEnv::default()).unwrap());
    }

    #[test]
    fn normalize_operand_symbol_x() {
        let exp = NormForm { a1: 1f64, a0: 0f64 };
        assert_eq!(exp, normalize(&parse_expression("x"), "x", &TopLevelEnv::default()).unwrap());
    }

    #[test]
    fn normalize_operand_symbol_y_unknown() {
        let act = normalize(&parse_expression("y"), "x", &TopLevelEnv::default());
        assert!(matches!(act, Err(SolverError::UnknownVariable(s)) if s == "y"));
    }

    #[test]
    fn normalize_operand_symbol_y() {
        let mut env = TopLevelEnv::default();
        env.put("y".to_string(), 12.0);
        let act = normalize(&parse_expression("y"), "x", &env);
        assert_eq!(Ok(NormForm { a1: 0.0, a0: 12.0 }), act);
    }

    #[test]
    fn normalize_operand_simple_add() {
        let exp = NormForm { a1: 1f64, a0: 1f64 };
        assert_eq!(exp, normalize(&parse_expression("x + 1"), "x", &TopLevelEnv::default()).unwrap());
    }

    #[test]
    fn normalize_operand_simple_sub() {
        let exp = NormForm {
            a1: 1f64,
            a0: -12f64,
        };
        assert_eq!(exp, normalize(&parse_expression("x - 12"), "x", &TopLevelEnv::default()).unwrap());
    }

    #[test]
    fn normalize_operand_simple_mul() {
        let exp = NormForm { a1: 2f64, a0: 0f64 };
        assert_eq!(exp, normalize(&parse_expression("x * 2"), "x", &TopLevelEnv::default()).unwrap());
    }

    #[test]
    fn normalize_operand_simple_rem() {
        let exp = NormForm { a1: 0f64, a0: 1f64 };
        assert_eq!(exp, normalize(&parse_expression("7 % 3"), "x", &TopLevelEnv::default()).unwrap());
    }

    #[test]
    fn normalize_operand_simple_pow() {
        let exp = NormForm {
            a1: 0f64,
            a0: 27f64,
        };
        assert_eq!(exp, normalize(&parse_expression("3 ^ 3"), "x", &TopLevelEnv::default()).unwrap());
    }

    #[test]
    fn normalize_operand_simple_norm_form() {
        let exp = NormForm { a1: 3f64, a0: 2f64 };
        assert_eq!(exp, normalize(&parse_expression("3 * x + 2"), "x", &TopLevelEnv::default()).unwrap());
    }

    #[test]
    fn normalize_operand_simple_norm_sub() {
        let exp = NormForm {
            a1: 3f64,
            a0: -2f64,
        };
        assert_eq!(exp, normalize(&parse_expression("3 * x - 2"), "x", &TopLevelEnv::default()).unwrap());
    }

    #[test]
    fn normalize_operand_div() {
        let exp = NormForm {
            a1: 4f64,
            a0: -5f64,
        };
        assert_eq!(
            exp,
            normalize(&parse_expression("(12 * x - 15) / 3"), "x", &TopLevelEnv::default()).unwrap()
        );
    }

    #[test]
    fn solve_for_simple() {
        assert!(
            if let Statement::SolveFor { lhs, rhs, sym } = parse("solve x = 10 for x").unwrap() {
                assert_eq!(Ok(10.0), solve_for(&lhs, &rhs, &sym, &TopLevelEnv::default()));
                true
            } else {
                false
            }
        );
    }

    #[test]
    fn solve_for_complex() {
        assert!(if let Statement::SolveFor { lhs, rhs, sym } =
            parse("solve 5 + 2 * x + 12 = 22 - 6 * x + 7 for x").unwrap()
        {
            assert_eq!(Ok(1.5), solve_for(&lhs, &rhs, &sym, &TopLevelEnv::default()));
            true
        } else {
            false
        });
    }

    #[test]
    fn solve_for_with_function_call() {
        let mut env = TopLevelEnv::default();
        env.put_fun("add".to_string(), Function {
            args: vec!["x".to_string(), "y".to_string()],
            body: Operand::Term(Box::new(Term {
                lhs: Operand::Symbol("x".to_string()),
                rhs: Operand::Symbol("y".to_string()),
                op: Operation::Add,
            }))
        });
        assert!(if let Statement::SolveFor { lhs, rhs, sym } =
            parse("solve 2 * x + add(5, 12) = 22 - 6 * x + 7 for x").unwrap()
        {
            assert_eq!(Ok(1.5), solve_for(&lhs, &rhs, &sym, &env));
            true
        } else {
            false
        });
    }
}
