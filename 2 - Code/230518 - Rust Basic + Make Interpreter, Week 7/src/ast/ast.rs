#[derive(Debug, PartialEq)]
pub struct Identifier(pub String);

#[derive(Debug, PartialEq)]
pub enum Statement {
    Let(Identifier, Expression),
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Identifier(Identifier),
    Literal(Literal),
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    Int(i64),
}

pub type Program = Vec<Statement>;
