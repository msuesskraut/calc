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
}

pub trait Env {
    fn get(&self, sym: &str) -> Option<&Number>;

    fn get_fun(&self, fun: &str) -> Option<&Function>;
}

#[derive(Debug, Default, Clone)]
pub struct TopLevelEnv {
    vars: HashMap<String, Number>,
    funs: HashMap<String, Function>,
}

impl TopLevelEnv {
    pub fn put(&mut self, sym: String, num: Number) {
        self.vars.insert(sym, num);
    }

    pub fn put_fun(&mut self, name: String, fun: Function) {
        self.funs.insert(name, fun);
    }
}

impl Env for TopLevelEnv {
    fn get(&self, sym: &str) -> Option<&Number> {
        self.vars.get(sym)
    }

    fn get_fun(&self, fun: &str) -> Option<&Function> {
        self.funs.get(fun)
    }
}

struct ScopedEnv<'a> {
    parent: &'a dyn Env,
    env: HashMap<String, Number>,
}

impl<'a> Env for ScopedEnv<'a> {
    fn get(&self, sym: &str) -> Option<&Number> {
        self.env.get(sym).or_else(|| self.parent.get(sym))
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

pub fn calc_function_call(fun_call: &FunCall, env: &dyn Env) -> Result<Number, CalcError> {
    let function = env
        .get_fun(&fun_call.name)
        .ok_or_else(|| CalcError::UnknownFunction(fun_call.name.to_string()))?;
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
    let fun_env: HashMap<String, Number> = function
        .args
        .iter()
        .zip(params.iter())
        .map(|(arg, num)| (arg.clone(), *num))
        .collect();
    //let env: HashMap<String, Number> = function.args.iter().zip(params.iter()).cloned().collect();
    calc_operand(
        &function.body,
        &ScopedEnv {
            parent: env,
            env: fun_env,
        },
    )
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
        env.put("x".to_string(), 12.0);

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
        env.put("x".to_string(), 12.0);
        assert_eq!(
            Ok(12.0),
            calc_operand(&Operand::Symbol("x".to_string()), &env)
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
        let function = Function {
            args: vec!["x".to_string(), "y".to_string()],
            body: Operand::Term(Box::new(Term { lhs, rhs, op })),
        };
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
}
