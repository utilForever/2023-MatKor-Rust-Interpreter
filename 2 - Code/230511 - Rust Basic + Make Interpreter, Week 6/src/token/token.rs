#[derive(Debug, PartialEq)]
pub enum Token {
    Illegal,
    Eof,

    // Identifiers + Literals
    Ident(String),
    Int(i64),
    Bool(bool),

    // Operators
    Assign,
    Plus,
    Minus,
    Bang,
    Asterisk,
    Slash,

    Equal,
    NotEqual,
    LessThan,
    GreaterThan,

    // Delimiters
    Comma,
    Semicolon,
    Lparen,
    Rparen,
    Lbrace,
    Rbrace,

    // Reserved Keywords
    Function,
    Let,
    If,
    Else,
    Return,
}
