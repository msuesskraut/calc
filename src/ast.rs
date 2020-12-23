pub type Number = f64;

#[derive(Debug, PartialEq, Clone)]
pub enum Operand {
    Number(Number),
    Symbol(String),
    Term(Box<Term>),
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
pub struct Expression {
    pub eq: Operand,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assignment {
    pub sym: String,
    pub eq: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Expression(Expression),
    Assignment(Assignment),
}
