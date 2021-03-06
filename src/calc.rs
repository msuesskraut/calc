use crate::ast::*;

use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum CalcError {
    #[error("Unknown symbol `{0}`")]
    UnknownSymbol(String),
    #[error(
        "Unexpected number of parameters for call to `{name}` - expected {exp}, but got {act}"
    )]
    UnexpectedNumberOfParameters {
        name: String,
        act: usize,
        exp: usize,
    },
    #[error("Unknown function `{0}`")]
    UnknownFunction(String),
    #[error("Cannot change value of constant `{0}`")]
    CannotChangeConstant(String),
}

pub trait Env {
    fn get(&self, sym: &str) -> Option<&Number>;

    fn get_fun(&self, fun: &str) -> Option<&Function>;
}

#[derive(Debug, Clone, PartialEq)]
struct EnvVariable {
    value: Number,
    is_const: bool,
}

impl EnvVariable {
    fn new_const(value: Number) -> EnvVariable {
        EnvVariable {
            value,
            is_const: true,
        }
    }

    fn new(value: Number) -> EnvVariable {
        EnvVariable {
            value,
            is_const: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TopLevelEnv {
    vars: HashMap<String, EnvVariable>,
    funs: HashMap<String, Function>,
}

impl TopLevelEnv {
    pub fn put(&mut self, sym: String, num: Number) -> Result<(), CalcError> {
        if let Some(var) = self.vars.get_mut(&sym) {
            if var.is_const {
                return Err(CalcError::CannotChangeConstant(sym));
            } else {
                var.value = num;
            }
        } else {
            self.vars.insert(sym, EnvVariable::new(num));
        }
        Ok(())
    }

    pub fn put_fun(&mut self, name: String, fun: Function) {
        self.funs.insert(name, fun);
    }
}

impl Env for TopLevelEnv {
    fn get(&self, sym: &str) -> Option<&Number> {
        self.vars.get(sym).map(|var| &var.value)
    }

    fn get_fun(&self, fun: &str) -> Option<&Function> {
        self.funs.get(fun)
    }
}

impl Default for TopLevelEnv {
    fn default() -> Self {
        let funs = {
            let mut funs = HashMap::new();

            macro_rules! buildin {
                ($($id:ident) +) => {
                    $(
                        fn $id(x: Number) -> Number { x.$id() }
                        funs.insert(stringify!($id).to_string(), Function::BuildIn(BuildInFunction {
                            name: stringify!($id).to_string(),
                            arg: "x".to_string(),
                            body: &$id,
                        }));
                    )+
                }
            }

            buildin!(abs sqrt sin sinh cos cosh tan tanh exp ln log2 log10 atan atanh asin asinh acos acosh);

            funs
        };

        let vars = {
            let mut vars = HashMap::new();

            macro_rules! buildin {
                ($($id:ident) +) => {
                    $(
                        vars.insert(
                            stringify!($id).to_string().to_lowercase(),
                            EnvVariable::new_const(std::f64::consts::$id));
                    )+
                }
            }

            buildin!(
                E
                FRAC_1_PI FRAC_1_SQRT_2 FRAC_2_PI FRAC_2_SQRT_PI FRAC_PI_2
                FRAC_PI_3 FRAC_PI_4 FRAC_PI_6 FRAC_PI_8
                LN_2 LN_10 LOG2_10 LOG2_E LOG10_2 LOG10_E
                PI SQRT_2 TAU);

            vars
        };

        Self { vars, funs }
    }
}

struct ScopedEnv<'a> {
    parent: &'a dyn Env,
    env: HashMap<&'a str, &'a Number>,
}

impl<'a> Env for ScopedEnv<'a> {
    fn get(&self, sym: &str) -> Option<&Number> {
        self.env.get(sym).copied().or_else(|| self.parent.get(sym))
    }

    fn get_fun(&self, fun: &str) -> Option<&Function> {
        self.parent.get_fun(fun)
    }
}

pub fn calc_term(term: &Term, env: &dyn Env) -> Result<Number, CalcError> {
    use self::Operation::*;
    let lhs = calc_operand(&term.lhs, env)?;
    let rhs = calc_operand(&term.rhs, env)?;
    Ok(match term.op {
        Add => lhs + rhs,
        Sub => lhs - rhs,
        Mul => lhs * rhs,
        Div => lhs / rhs,
        Rem => lhs % rhs,
        Pow => lhs.powf(rhs),
    })
}

fn calc_custom_function_call(
    function: &CustomFunction,
    fun_call: &FunCall,
    env: &dyn Env,
) -> Result<Number, CalcError> {
    if fun_call.params.len() != function.args.len() {
        return Err(CalcError::UnexpectedNumberOfParameters {
            name: fun_call.name.clone(),
            act: fun_call.params.len(),
            exp: function.args.len(),
        });
    }
    let params = fun_call
        .params
        .iter()
        .try_fold(Vec::new(), |mut params, op| {
            params.push(calc_operand(op, env)?);
            Ok(params)
        })?;
    let fun_env: HashMap<&str, &Number> = function
        .args
        .iter()
        .zip(params.iter())
        .map(|(arg, num)| (arg.as_str(), num))
        .collect();
    calc_operand(
        &function.body,
        &ScopedEnv {
            parent: env,
            env: fun_env,
        },
    )
}

pub fn calc_function_call(fun_call: &FunCall, env: &dyn Env) -> Result<Number, CalcError> {
    let function = env
        .get_fun(&fun_call.name)
        .ok_or_else(|| CalcError::UnknownFunction(fun_call.name.to_string()))?;
    match function {
        Function::Custom(function) => calc_custom_function_call(function, fun_call, env),
        Function::BuildIn(function) => {
            if fun_call.params.len() != 1 {
                return Err(CalcError::UnexpectedNumberOfParameters {
                    name: fun_call.name.clone(),
                    act: fun_call.params.len(),
                    exp: 1,
                });
            }
            let x = calc_operand(&fun_call.params[0], env)?;
            Ok((function.body)(x))
        }
    }
}

pub fn calc_operand(op: &Operand, env: &dyn Env) -> Result<Number, CalcError> {
    use self::Operand::*;
    match op {
        Number(num) => Ok(*num),
        Term(term) => calc_term(term, env),
        Symbol(sym) => match env.get(sym) {
            Some(num) => Ok(*num),
            None => Err(CalcError::UnknownSymbol(sym.clone())),
        },
        FunCall(fun_call) => calc_function_call(fun_call, env),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_env_empty() {
        let env = TopLevelEnv::default();

        assert_eq!(None, env.get("x"));
    }

    #[test]
    fn read_env_var() {
        let mut env = TopLevelEnv::default();
        env.put("x".to_string(), 12.0).unwrap();

        assert_eq!(Some(&12.0), env.get("x"));
    }

    #[test]
    fn calc_number_atom() {
        assert_eq!(
            Ok(12.0),
            calc_operand(&Operand::Number(12.0), &TopLevelEnv::default())
        );
    }

    #[test]
    fn calc_sym_unknown() {
        assert_eq!(
            Err(CalcError::UnknownSymbol("x".to_string())),
            calc_operand(&Operand::Symbol("x".to_string()), &TopLevelEnv::default())
        );
    }

    #[test]
    fn calc_sym_known() {
        let mut env = TopLevelEnv::default();
        env.put("x".to_string(), 12.0).unwrap();
        assert_eq!(
            Ok(12.0),
            calc_operand(&Operand::Symbol("x".to_string()), &env)
        );
    }

    #[test]
    fn calc_sym_constant() {
        let mut env = TopLevelEnv::default();
        assert!(
            matches!(env.put("e".to_string(), 2.0), Err(CalcError::CannotChangeConstant(sym)) if sym == "e")
        );
    }

    #[test]
    fn calc_term_add() {
        let lhs = Operand::Number(3.0);
        let rhs = Operand::Number(4.0);
        let op = Operation::Add;
        assert_eq!(
            Ok(7.0),
            calc_term(&Term { op, lhs, rhs }, &TopLevelEnv::default())
        );
    }

    #[test]
    fn calc_term_sub() {
        let lhs = Operand::Number(3.0);
        let rhs = Operand::Number(4.0);
        let op = Operation::Sub;
        assert_eq!(
            Ok(-1.0),
            calc_term(&Term { op, lhs, rhs }, &TopLevelEnv::default())
        );
    }

    #[test]
    fn calc_term_mul() {
        let lhs = Operand::Number(3.0);
        let rhs = Operand::Number(4.0);
        let op = Operation::Mul;
        assert_eq!(
            Ok(12.0),
            calc_term(&Term { op, lhs, rhs }, &TopLevelEnv::default())
        );
    }

    #[test]
    fn calc_term_div() {
        let lhs = Operand::Number(12.0);
        let rhs = Operand::Number(4.0);
        let op = Operation::Div;
        assert_eq!(
            Ok(3.0),
            calc_term(&Term { op, lhs, rhs }, &TopLevelEnv::default())
        );
    }

    #[test]
    fn calc_term_rem() {
        let lhs = Operand::Number(14.0);
        let rhs = Operand::Number(4.0);
        let op = Operation::Rem;
        assert_eq!(
            Ok(2.0),
            calc_term(&Term { op, lhs, rhs }, &TopLevelEnv::default())
        );
    }

    #[test]
    fn calc_term_pow() {
        let lhs = Operand::Number(3.0);
        let rhs = Operand::Number(4.0);
        let op = Operation::Pow;
        assert_eq!(
            Ok(81.0),
            calc_term(&Term { op, lhs, rhs }, &TopLevelEnv::default())
        );
    }

    #[test]
    fn calc_equation_simple() {
        let op = Operand::Number(3.0);
        assert_eq!(Ok(3.0), calc_operand(&op, &TopLevelEnv::default()));
    }

    #[test]
    fn calc_simple_function_call() {
        let lhs = Operand::Symbol("x".to_string());
        let rhs = Operand::Symbol("y".to_string());
        let op = Operation::Add;
        let function = Function::Custom(CustomFunction {
            args: vec!["x".to_string(), "y".to_string()],
            body: Operand::Term(Box::new(Term { lhs, rhs, op })),
        });
        let mut funs = HashMap::new();
        funs.insert("fun".to_string(), function);
        let env = TopLevelEnv {
            vars: HashMap::new(),
            funs,
        };
        let expr = Operand::FunCall(FunCall {
            name: "fun".to_string(),
            params: vec![Operand::Number(4.0), Operand::Number(3.0)],
        });
        assert_eq!(Ok(7.0), calc_operand(&expr, &env));
    }

    #[test]
    fn calc_buildinfunction_call() {
        fn my_cos(x: Number) -> Number {
            x.cos()
        }
        let function = Function::BuildIn(BuildInFunction {
            name: "cos".to_string(),
            arg: "x".to_string(),
            body: &my_cos,
        });
        let mut funs = HashMap::new();
        funs.insert("cos".to_string(), function);
        let env = TopLevelEnv {
            vars: HashMap::new(),
            funs,
        };
        let expr = Operand::FunCall(FunCall {
            name: "cos".to_string(),
            params: vec![Operand::Number(0.)],
        });
        assert_eq!(Ok(1.0), calc_operand(&expr, &env));
    }

    #[test]
    fn top_level_env_build_ins() {
        let env = TopLevelEnv::default();
        assert!(env.get_fun("sin").is_some());
        assert!(env.get_fun("cos").is_some());
    }
}
