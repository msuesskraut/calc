pub type Number = f64;

#[derive(Debug, PartialEq, Clone)]
pub struct FunCall {
    pub name: String,
    pub params: Vec<Operand>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operand {
    Number(Number),
    Symbol(String),
    Term(Box<Term>),
    FunCall(FunCall),
}

impl Operand {
    pub fn is_symbol(&self, sym: &str) -> bool {
        matches!(self, Operand::Symbol(s) if s == sym)
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
pub struct CustomFunction {
    pub args: Vec<String>,
    pub body: Operand,
}

#[derive(Clone)]
pub struct BuildInFunction {
    pub name: String,
    pub arg: String,
    pub body: &'static dyn Fn(Number) -> Number,
}

impl PartialEq for BuildInFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arg == other.arg
    }
}

impl std::fmt::Debug for BuildInFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuildInFunction")
            .field("name", &self.name)
            .field("arg", &self.arg)
            .finish()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Function {
    Custom(CustomFunction),
    BuildIn(BuildInFunction),
}

impl Default for Function {
    fn default() -> Self {
        Function::Custom(CustomFunction {
            args: Vec::new(),
            body: Operand::Number(1.0),
        })
    }
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
    },
    Function {
        name: String,
        fun: Function,
    },
    Plot {
        name: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    fn create_term() -> Term {
        let lhs = Operand::Number(1.0);
        let rhs = Operand::Number(1.0);
        let op = Operation::Add;
        Term { op, lhs, rhs }
    }

    #[test]
    fn operand_is_symbol() {
        assert!(Operand::Symbol("x".to_string()).is_symbol("x"));
    }

    #[test]
    fn operand_is_not_symbol() {
        assert!(!Operand::Symbol("y".to_string()).is_symbol("x"));
        assert!(!Operand::Number(1.0).is_symbol("x"));
        assert!(!Operand::Term(Box::new(create_term())).is_symbol("x"));
    }
}
