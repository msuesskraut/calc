pub type Number = f64;

#[derive(Debug, PartialEq, Clone)]
pub enum Operand {
    Number(Number),
    Symbol(String),
    Term(Box<Term>),
}

impl Operand {
    pub fn is_num(&self) -> bool {
        match self {
            Operand::Number(_) => true,
            _ => false,
        }
    }

    pub fn is_sym(&self) -> bool {
        match self {
            Operand::Symbol(_) => true,
            _ => false,
        }
    }

    pub fn is_symbol(&self, sym: &str) -> bool {
        match self {
            Operand::Symbol(s) if s == sym => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Term {
    pub op: Operation,
    pub lhs: Operand,
    pub rhs: Operand,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Expression {
        op: Operand,
    },
    Assignment {
        sym: String,
        op: Operand,
    },
    SolveFor {
        lhs: Operand,
        rhs: Operand,
        sym: String,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operand_is_num() {
        assert!(Operand::Number(1.0).is_num());
    }

    fn create_term() -> Term {
        let lhs = Operand::Number(1.0);
        let rhs = Operand::Number(1.0);
        let op = Operation::Add;
        Term { op, lhs, rhs }
    }

    #[test]
    fn operand_is_not_num() {
        assert!(!Operand::Symbol("x".to_string()).is_num());
        assert!(!Operand::Term(Box::new(create_term())).is_num());
    }

    #[test]
    fn operand_is_sym() {
        assert!(Operand::Symbol("x".to_string()).is_sym());
    }

    #[test]
    fn operand_is_not_sym() {
        assert!(!Operand::Number(1.0).is_sym());
        assert!(!Operand::Term(Box::new(create_term())).is_sym());
    }

    #[test]
    fn operand_is_symbol() {
        assert!(Operand::Symbol("x".to_string()).is_symbol("x"));
    }

    #[test]
    fn operand_is_not_symbol() {
        assert!(!Operand::Symbol("y".to_string()).is_symbol("x"));
        assert!(!Operand::Number(1.0).is_sym());
        assert!(!Operand::Term(Box::new(create_term())).is_sym());
    }
}
