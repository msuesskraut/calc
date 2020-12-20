use crate::ast::*;

#[derive(Debug, PartialEq, Eq)]
pub enum CalcError {
    UnknownSymbol(String),
}

fn calc_term(term: Term, env: &Env) -> Result<Number, CalcError> {
    use self::Operation::*;
    let lhs = calc_operand(term.lhs, env)?;
    let rhs = calc_operand(term.rhs, env)?;
    Ok(match term.op {
        Add => lhs + rhs,
        Sub => lhs - rhs,
        Mul => lhs * rhs,
        Div => lhs / rhs,
        Rem => lhs % rhs,
        Pow => lhs.powf(rhs),
    })
}

fn calc_operand(op: Operand, env: &Env) -> Result<Number, CalcError> {
    use self::Operand::*;
    match op {
        Number(num) => Ok(num),
        Term(term) => calc_term(*term, env),
        Symbol(sym) => match env.get(&sym) {
            Some(num) => Ok(*num),
            None => Err(CalcError::UnknownSymbol(sym.clone())),
        },
    }
}

pub fn calc_equation(eq: Equation, env: &Env) -> Result<Number, CalcError> {
    calc_operand(eq.eq, env)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_number_atom() {
        assert_eq!(Ok(12.0), calc_operand(Operand::Number(12.0), &Env::new()));
    }

    #[test]
    fn calc_sym_unknown() {
        assert_eq!(
            Err(CalcError::UnknownSymbol("x".to_string())),
            calc_operand(Operand::Symbol("x".to_string()), &Env::new())
        );
    }

    #[test]
    fn calc_sym_known() {
        let mut env = Env::new();
        env.put("x".to_string(), 12.0);
        assert_eq!(
            Ok(12.0),
            calc_operand(Operand::Symbol("x".to_string()), &env)
        );
    }

    #[test]
    fn calc_term_add() {
        let lhs = Operand::Number(3.0);
        let rhs = Operand::Number(4.0);
        let op = Operation::Add;
        assert_eq!(Ok(7.0), calc_term(Term { op, lhs, rhs }, &Env::new()));
    }

    #[test]
    fn calc_term_sub() {
        let lhs = Operand::Number(3.0);
        let rhs = Operand::Number(4.0);
        let op = Operation::Sub;
        assert_eq!(Ok(-1.0), calc_term(Term { op, lhs, rhs }, &Env::new()));
    }

    #[test]
    fn calc_term_mul() {
        let lhs = Operand::Number(3.0);
        let rhs = Operand::Number(4.0);
        let op = Operation::Mul;
        assert_eq!(Ok(12.0), calc_term(Term { op, lhs, rhs }, &Env::new()));
    }

    #[test]
    fn calc_term_div() {
        let lhs = Operand::Number(12.0);
        let rhs = Operand::Number(4.0);
        let op = Operation::Div;
        assert_eq!(Ok(3.0), calc_term(Term { op, lhs, rhs }, &Env::new()));
    }

    #[test]
    fn calc_term_rem() {
        let lhs = Operand::Number(14.0);
        let rhs = Operand::Number(4.0);
        let op = Operation::Rem;
        assert_eq!(Ok(2.0), calc_term(Term { op, lhs, rhs }, &Env::new()));
    }

    #[test]
    fn calc_term_pow() {
        let lhs = Operand::Number(3.0);
        let rhs = Operand::Number(4.0);
        let op = Operation::Pow;
        assert_eq!(Ok(81.0), calc_term(Term { op, lhs, rhs }, &Env::new()));
    }

    #[test]
    fn calc_equation_simple() {
        let eq = Equation {
            eq: Operand::Number(3.0),
        };
        assert_eq!(Ok(3.0), calc_equation(eq, &Env::new()));
    }
}
