use std::fmt;

use crate::ast::ast::{Expression, Identifier, Literal, Program, Statement};
use crate::lexer::lexer::Lexer;
use crate::token::token::Token;

#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    UnexpectedToken,
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseErrorKind::UnexpectedToken => write!(f, "Unexpected Token"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseError {
    kind: ParseErrorKind,
    msg: String,
}

impl ParseError {
    fn new(kind: ParseErrorKind, msg: String) -> Self {
        ParseError { kind, msg }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.msg)
    }
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    cur_token: Token,
    peek_token: Token,
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        let mut parser = Parser {
            lexer,
            cur_token: Token::Eof,
            peek_token: Token::Eof,
            errors: Vec::new(),
        };

        parser.next_token();
        parser.next_token();

        parser
    }

    #[allow(dead_code)]
    fn get_errors(&mut self) -> Vec<ParseError> {
        self.errors.clone()
    }

    fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn cur_token_is(&mut self, token: Token) -> bool {
        self.cur_token == token
    }

    fn peek_token_is(&mut self, token: Token) -> bool {
        self.peek_token == token
    }

    fn expect_peek(&mut self, token: Token) -> bool {
        if self.peek_token_is(token.clone()) {
            self.next_token();
            true
        } else {
            self.error_next_token(token);
            false
        }
    }

    fn error_next_token(&mut self, token: Token) {
        self.errors.push(ParseError::new(
            ParseErrorKind::UnexpectedToken,
            format!(
                "expected next token to be {:?}, got {:?} instead",
                token, self.peek_token
            ),
        ));
    }

    pub fn parse_program(&mut self) -> Program {
        let mut program = Vec::new();

        while self.cur_token != Token::Eof {
            match self.parse_statement() {
                Some(statement) => program.push(statement),
                None => {}
            }

            self.next_token();
        }

        program
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.cur_token {
            Token::Let => self.parse_let_statement(),
            _ => None,
        }
    }

    fn parse_let_statement(&mut self) -> Option<Statement> {
        match &self.peek_token {
            Token::Ident(_) => self.next_token(),
            _ => return None,
        };

        let identifier = match self.parse_identifier() {
            Some(identifier) => identifier,
            None => return None,
        };

        if !self.expect_peek(Token::Assign) {
            return None;
        }

        self.next_token();

        let expression = match self.parse_expression() {
            Some(expression) => expression,
            None => return None,
        };

        while !self.cur_token_is(Token::Semicolon) {
            self.next_token();
        }

        Some(Statement::Let(identifier, expression))
    }

    fn parse_expression(&mut self) -> Option<Expression> {
        match self.cur_token {
            Token::Int(_) => self.parse_int_expression(),
            _ => None,
        }
    }

    fn parse_identifier(&mut self) -> Option<Identifier> {
        match &self.cur_token {
            Token::Ident(ident) => Some(Identifier(ident.clone())),
            _ => None,
        }
    }

    fn parse_int_expression(&mut self) -> Option<Expression> {
        match &self.cur_token {
            Token::Int(int) => Some(Expression::Literal(Literal::Int(int.clone()))),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::ast::{Expression, Identifier, Literal, Statement};
    use crate::lexer::lexer::Lexer;
    use crate::parser::parser::Parser;

    fn check_parse_errors(parser: &mut Parser) {
        let errors = parser.get_errors();

        if errors.is_empty() {
            return;
        }

        println!("\n");
        println!("parser has {} errors", errors.len());

        for error in errors {
            println!("parse error: {:?}", error);
        }

        println!("\n");
        panic!("failed");
    }

    #[test]
    fn test_let_statement() {
        let input = r#"
let x = 5;
let y = 10;
let foobar = 838383;
"#;
        let l = Lexer::new(input);
        let mut p = Parser::new(l);

        let program = p.parse_program();
        check_parse_errors(&mut p);

        assert_eq!(
            vec![
                Statement::Let(
                    Identifier(String::from("x")),
                    Expression::Literal(Literal::Int(5))
                ),
                Statement::Let(
                    Identifier(String::from("y")),
                    Expression::Literal(Literal::Int(10))
                ),
                Statement::Let(
                    Identifier(String::from("foobar")),
                    Expression::Literal(Literal::Int(838383)),
                ),
            ],
            program,
        );
    }
}
