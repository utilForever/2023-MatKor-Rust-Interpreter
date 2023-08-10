use std::fmt;

use crate::ast::ast::{
    Expression, Identifier, Infix, Literal, Precedence, Prefix, Program, Statement,
};
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

    fn token_to_precedence(token: &Token) -> Precedence {
        match token {
            Token::Equal | Token::NotEqual => Precedence::Equals,
            Token::LessThan => Precedence::LessGreater,
            Token::GreaterThan => Precedence::LessGreater,
            Token::Plus | Token::Minus => Precedence::Sum,
            Token::Asterisk | Token::Slash => Precedence::Product,
            Token::Lparen => Precedence::Call,
            _ => Precedence::Lowest,
        }
    }

    pub fn get_errors(&mut self) -> Vec<ParseError> {
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

    fn cur_token_precedence(&mut self) -> Precedence {
        Self::token_to_precedence(&self.cur_token)
    }

    fn peek_token_precedence(&mut self) -> Precedence {
        Self::token_to_precedence(&self.peek_token)
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
            Token::Return => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
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

        let expression = match self.parse_expression(Precedence::Lowest) {
            Some(expression) => expression,
            None => return None,
        };

        if self.cur_token_is(Token::Semicolon) {
            self.next_token();
        }

        Some(Statement::Let(identifier, expression))
    }

    fn parse_return_statement(&mut self) -> Option<Statement> {
        self.next_token();

        let expression = match self.parse_expression(Precedence::Lowest) {
            Some(expression) => expression,
            None => return None,
        };

        if self.cur_token_is(Token::Semicolon) {
            self.next_token();
        }

        Some(Statement::Return(expression))
    }

    fn parse_expression_statement(&mut self) -> Option<Statement> {
        match self.parse_expression(Precedence::Lowest) {
            Some(expression) => {
                if self.peek_token_is(Token::Semicolon) {
                    self.next_token();
                }
                Some(Statement::Expression(expression))
            }
            None => None,
        }
    }

    fn parse_block_statement(&mut self) -> Vec<Statement> {
        self.next_token();

        let mut block = Vec::new();

        while !self.cur_token_is(Token::Rbrace) && !self.cur_token_is(Token::Eof) {
            match self.parse_statement() {
                Some(statement) => block.push(statement),
                None => {}
            }

            self.next_token();
        }

        block
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        // Prefix
        let mut left = match self.cur_token {
            Token::Ident(_) => self.parse_identifier_expression(),
            Token::Int(_) => self.parse_int_expression(),
            Token::Bool(_) => self.parse_bool_expression(),
            Token::Bang | Token::Minus => self.parse_prefix_expression(),
            Token::Lparen => self.parse_grouped_expression(),
            Token::If => self.parse_if_expression(),
            Token::Function => self.parse_function_expression(),
            _ => None,
        };

        // Infix
        while !self.peek_token_is(Token::Semicolon) && precedence < self.peek_token_precedence() {
            match self.peek_token {
                Token::Plus
                | Token::Minus
                | Token::Asterisk
                | Token::Slash
                | Token::Equal
                | Token::NotEqual
                | Token::LessThan
                | Token::GreaterThan => {
                    self.next_token();
                    left = self.parse_infix_expression(left.unwrap());
                }
                Token::Lparen => {
                    self.next_token();
                    left = self.parse_call_expression(left.unwrap());
                }
                _ => return left,
            }
        }

        left
    }

    fn parse_identifier(&mut self) -> Option<Identifier> {
        match &self.cur_token {
            Token::Ident(ident) => Some(Identifier(ident.clone())),
            _ => None,
        }
    }

    fn parse_identifier_expression(&mut self) -> Option<Expression> {
        match self.parse_identifier() {
            Some(ident) => Some(Expression::Identifier(ident)),
            None => None,
        }
    }

    fn parse_int_expression(&mut self) -> Option<Expression> {
        match &self.cur_token {
            Token::Int(int) => Some(Expression::Literal(Literal::Int(int.clone()))),
            _ => None,
        }
    }

    fn parse_bool_expression(&mut self) -> Option<Expression> {
        match self.cur_token {
            Token::Bool(value) => Some(Expression::Literal(Literal::Bool(value == true))),
            _ => None,
        }
    }

    fn parse_expression_list(&mut self, end: Token) -> Option<Vec<Expression>> {
        let mut list = Vec::new();

        if self.peek_token_is(end.clone()) {
            self.next_token();
            return Some(list);
        }

        self.next_token();

        match self.parse_expression(Precedence::Lowest) {
            Some(expression) => list.push(expression),
            None => return None,
        };

        while self.peek_token_is(Token::Comma) {
            self.next_token();
            self.next_token();

            match self.parse_expression(Precedence::Lowest) {
                Some(expression) => list.push(expression),
                None => return None,
            };
        }

        if !self.expect_peek(end) {
            return None;
        }

        Some(list)
    }

    fn parse_prefix_expression(&mut self) -> Option<Expression> {
        let prefix = match self.cur_token {
            Token::Bang => Prefix::Not,
            Token::Minus => Prefix::Minus,
            _ => return None,
        };

        self.next_token();

        match self.parse_expression(Precedence::Prefix) {
            Some(expr) => Some(Expression::Prefix(prefix, Box::new(expr))),
            None => None,
        }
    }

    fn parse_infix_expression(&mut self, left: Expression) -> Option<Expression> {
        let infix = match self.cur_token {
            Token::Plus => Infix::Plus,
            Token::Minus => Infix::Minus,
            Token::Asterisk => Infix::Multiply,
            Token::Slash => Infix::Divide,
            Token::Equal => Infix::Equal,
            Token::NotEqual => Infix::NotEqual,
            Token::LessThan => Infix::LessThan,
            Token::GreaterThan => Infix::GreaterThan,
            _ => return None,
        };

        let precedence = self.cur_token_precedence();

        self.next_token();

        match self.parse_expression(precedence) {
            Some(expr) => Some(Expression::Infix(infix, Box::new(left), Box::new(expr))),
            None => None,
        }
    }

    fn parse_call_expression(&mut self, func: Expression) -> Option<Expression> {
        let arguments = match self.parse_expression_list(Token::Rparen) {
            Some(arguments) => arguments,
            None => return None,
        };

        Some(Expression::Call {
            function: Box::new(func),
            arguments,
        })
    }

    fn parse_grouped_expression(&mut self) -> Option<Expression> {
        self.next_token();

        let expr = self.parse_expression(Precedence::Lowest);

        if !self.expect_peek(Token::Rparen) {
            None
        } else {
            expr
        }
    }

    fn parse_if_expression(&mut self) -> Option<Expression> {
        if !self.expect_peek(Token::Lparen) {
            return None;
        }

        self.next_token();

        let condition = match self.parse_expression(Precedence::Lowest) {
            Some(expr) => expr,
            None => return None,
        };

        if !self.expect_peek(Token::Rparen) || !self.expect_peek(Token::Lbrace) {
            return None;
        }

        let consequence = self.parse_block_statement();
        let mut alternative = None;

        if self.peek_token_is(Token::Else) {
            self.next_token();

            if !self.expect_peek(Token::Lbrace) {
                return None;
            }

            alternative = Some(self.parse_block_statement());
        }

        Some(Expression::If {
            condition: Box::new(condition),
            consequence,
            alternative,
        })
    }

    fn parse_function_expression(&mut self) -> Option<Expression> {
        if !self.expect_peek(Token::Lparen) {
            return None;
        }

        let parameters = match self.parse_function_parameters() {
            Some(parameters) => parameters,
            None => return None,
        };

        if !self.expect_peek(Token::Lbrace) {
            return None;
        }

        Some(Expression::Function {
            parameters,
            body: self.parse_block_statement(),
        })
    }

    fn parse_function_parameters(&mut self) -> Option<Vec<Identifier>> {
        let mut parameters = Vec::new();

        if self.peek_token_is(Token::Rparen) {
            self.next_token();
            return Some(parameters);
        }

        self.next_token();

        match self.parse_identifier() {
            Some(ident) => parameters.push(ident),
            None => return None,
        };

        while self.peek_token_is(Token::Comma) {
            self.next_token();
            self.next_token();

            match self.parse_identifier() {
                Some(ident) => parameters.push(ident),
                None => return None,
            }
        }

        if !self.expect_peek(Token::Rparen) {
            return None;
        }

        Some(parameters)
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::ast::{Expression, Identifier, Infix, Literal, Prefix, Statement};
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

    #[test]
    fn test_return_statement() {
        let input = r#"
return 5;
return 10;
return 993322;
"#;
        let l = Lexer::new(input);
        let mut p = Parser::new(l);

        let program = p.parse_program();
        check_parse_errors(&mut p);

        assert_eq!(
            vec![
                Statement::Return(Expression::Literal(Literal::Int(5))),
                Statement::Return(Expression::Literal(Literal::Int(10))),
                Statement::Return(Expression::Literal(Literal::Int(993322))),
            ],
            program,
        );
    }

    #[test]
    fn test_identifier_expression() {
        let input = "foobar;";

        let l = Lexer::new(input);
        let mut p = Parser::new(l);

        let program = p.parse_program();
        check_parse_errors(&mut p);

        assert_eq!(
            vec![Statement::Expression(Expression::Identifier(Identifier(
                String::from("foobar")
            )))],
            program,
        );
    }

    #[test]
    fn test_integer_literal_expression() {
        let input = "5;";

        let l = Lexer::new(input);
        let mut p = Parser::new(l);

        let program = p.parse_program();
        check_parse_errors(&mut p);

        assert_eq!(
            vec![Statement::Expression(Expression::Literal(Literal::Int(5)))],
            program,
        );
    }

    #[test]
    fn test_boolean_literal_expression() {
        let tests = vec![
            (
                "true;",
                Statement::Expression(Expression::Literal(Literal::Bool(true))),
            ),
            (
                "false;",
                Statement::Expression(Expression::Literal(Literal::Bool(false))),
            ),
        ];

        for (input, expect) in tests {
            let l = Lexer::new(input);
            let mut p = Parser::new(l);

            let program = p.parse_program();
            check_parse_errors(&mut p);

            assert_eq!(vec![expect], program);
        }
    }

    #[test]
    fn test_prefix_expression() {
        let tests = vec![
            (
                "!5;",
                Statement::Expression(Expression::Prefix(
                    Prefix::Not,
                    Box::new(Expression::Literal(Literal::Int(5))),
                )),
            ),
            (
                "-15;",
                Statement::Expression(Expression::Prefix(
                    Prefix::Minus,
                    Box::new(Expression::Literal(Literal::Int(15))),
                )),
            ),
        ];

        for (input, expect) in tests {
            let l = Lexer::new(input);
            let mut p = Parser::new(l);

            let program = p.parse_program();
            check_parse_errors(&mut p);

            assert_eq!(vec![expect], program);
        }
    }

    #[test]
    fn test_infix_operator() {
        let tests = vec![
            (
                "5 + 5;",
                Statement::Expression(Expression::Infix(
                    Infix::Plus,
                    Box::new(Expression::Literal(Literal::Int(5))),
                    Box::new(Expression::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 - 5;",
                Statement::Expression(Expression::Infix(
                    Infix::Minus,
                    Box::new(Expression::Literal(Literal::Int(5))),
                    Box::new(Expression::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 * 5;",
                Statement::Expression(Expression::Infix(
                    Infix::Multiply,
                    Box::new(Expression::Literal(Literal::Int(5))),
                    Box::new(Expression::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 / 5;",
                Statement::Expression(Expression::Infix(
                    Infix::Divide,
                    Box::new(Expression::Literal(Literal::Int(5))),
                    Box::new(Expression::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 > 5;",
                Statement::Expression(Expression::Infix(
                    Infix::GreaterThan,
                    Box::new(Expression::Literal(Literal::Int(5))),
                    Box::new(Expression::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 < 5;",
                Statement::Expression(Expression::Infix(
                    Infix::LessThan,
                    Box::new(Expression::Literal(Literal::Int(5))),
                    Box::new(Expression::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 == 5;",
                Statement::Expression(Expression::Infix(
                    Infix::Equal,
                    Box::new(Expression::Literal(Literal::Int(5))),
                    Box::new(Expression::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 != 5;",
                Statement::Expression(Expression::Infix(
                    Infix::NotEqual,
                    Box::new(Expression::Literal(Literal::Int(5))),
                    Box::new(Expression::Literal(Literal::Int(5))),
                )),
            ),
        ];

        for (input, expect) in tests {
            let l = Lexer::new(input);
            let mut p = Parser::new(l);

            let program = p.parse_program();
            check_parse_errors(&mut p);

            assert_eq!(vec![expect], program);
        }
    }

    #[test]
    fn test_if_expr() {
        let input = "if (x < y) { x }";

        let l = Lexer::new(input);
        let mut p = Parser::new(l);

        let program = p.parse_program();
        check_parse_errors(&mut p);

        assert_eq!(
            vec![Statement::Expression(Expression::If {
                condition: Box::new(Expression::Infix(
                    Infix::LessThan,
                    Box::new(Expression::Identifier(Identifier(String::from("x")))),
                    Box::new(Expression::Identifier(Identifier(String::from("y"))))
                )),
                consequence: vec![Statement::Expression(Expression::Identifier(Identifier(
                    String::from("x")
                )))],
                alternative: None,
            })],
            program,
        );
    }

    #[test]
    fn test_if_else_expr() {
        let input = "if (x < y) { x } else { y }";

        let l = Lexer::new(input);
        let mut p = Parser::new(l);

        let program = p.parse_program();
        check_parse_errors(&mut p);

        assert_eq!(
            vec![Statement::Expression(Expression::If {
                condition: Box::new(Expression::Infix(
                    Infix::LessThan,
                    Box::new(Expression::Identifier(Identifier(String::from("x")))),
                    Box::new(Expression::Identifier(Identifier(String::from("y"))))
                )),
                consequence: vec![Statement::Expression(Expression::Identifier(Identifier(
                    String::from("x")
                )))],
                alternative: Some(vec![Statement::Expression(Expression::Identifier(
                    Identifier(String::from("y"))
                ))]),
            })],
            program,
        );
    }

    #[test]
    fn test_func_expression() {
        let input = "fn(x, y) { x + y; }";

        let l = Lexer::new(input);
        let mut p = Parser::new(l);

        let program = p.parse_program();
        check_parse_errors(&mut p);

        assert_eq!(
            vec![Statement::Expression(Expression::Function {
                parameters: vec![Identifier(String::from("x")), Identifier(String::from("y")),],
                body: vec![Statement::Expression(Expression::Infix(
                    Infix::Plus,
                    Box::new(Expression::Identifier(Identifier(String::from("x")))),
                    Box::new(Expression::Identifier(Identifier(String::from("y")))),
                ))],
            })],
            program,
        );
    }

    #[test]
    fn test_func_params() {
        let tests = vec![
            ("fn() {};", vec![]),
            ("fn(x) {};", vec![Identifier(String::from("x"))]),
            (
                "fn(x, y, z) {};",
                vec![
                    Identifier(String::from("x")),
                    Identifier(String::from("y")),
                    Identifier(String::from("z")),
                ],
            ),
        ];

        for (input, expect) in tests {
            let l = Lexer::new(input);
            let mut p = Parser::new(l);

            let program = p.parse_program();
            check_parse_errors(&mut p);

            assert_eq!(
                vec![Statement::Expression(Expression::Function {
                    parameters: expect,
                    body: vec![],
                })],
                program,
            );
        }
    }

    #[test]
    fn test_call_expression() {
        let input = "add(1, 2 * 3, 4 + 5);";

        let l = Lexer::new(input);
        let mut p = Parser::new(l);

        let program = p.parse_program();
        check_parse_errors(&mut p);

        assert_eq!(
            vec![Statement::Expression(Expression::Call {
                function: Box::new(Expression::Identifier(Identifier(String::from("add")))),
                arguments: vec![
                    Expression::Literal(Literal::Int(1)),
                    Expression::Infix(
                        Infix::Multiply,
                        Box::new(Expression::Literal(Literal::Int(2))),
                        Box::new(Expression::Literal(Literal::Int(3)))
                    ),
                    Expression::Infix(
                        Infix::Plus,
                        Box::new(Expression::Literal(Literal::Int(4))),
                        Box::new(Expression::Literal(Literal::Int(5)))
                    ),
                ],
            })],
            program,
        );
    }

    #[test]
    fn test_operator_precedence_parsing() {
        let tests = vec![
            (
                "-a * b",
                Statement::Expression(Expression::Infix(
                    Infix::Multiply,
                    Box::new(Expression::Prefix(
                        Prefix::Minus,
                        Box::new(Expression::Identifier(Identifier(String::from("a")))),
                    )),
                    Box::new(Expression::Identifier(Identifier(String::from("b")))),
                )),
            ),
            (
                "!-a",
                Statement::Expression(Expression::Prefix(
                    Prefix::Not,
                    Box::new(Expression::Prefix(
                        Prefix::Minus,
                        Box::new(Expression::Identifier(Identifier(String::from("a")))),
                    )),
                )),
            ),
            (
                "a + b + c",
                Statement::Expression(Expression::Infix(
                    Infix::Plus,
                    Box::new(Expression::Infix(
                        Infix::Plus,
                        Box::new(Expression::Identifier(Identifier(String::from("a")))),
                        Box::new(Expression::Identifier(Identifier(String::from("b")))),
                    )),
                    Box::new(Expression::Identifier(Identifier(String::from("c")))),
                )),
            ),
            (
                "a + b - c",
                Statement::Expression(Expression::Infix(
                    Infix::Minus,
                    Box::new(Expression::Infix(
                        Infix::Plus,
                        Box::new(Expression::Identifier(Identifier(String::from("a")))),
                        Box::new(Expression::Identifier(Identifier(String::from("b")))),
                    )),
                    Box::new(Expression::Identifier(Identifier(String::from("c")))),
                )),
            ),
            (
                "a * b * c",
                Statement::Expression(Expression::Infix(
                    Infix::Multiply,
                    Box::new(Expression::Infix(
                        Infix::Multiply,
                        Box::new(Expression::Identifier(Identifier(String::from("a")))),
                        Box::new(Expression::Identifier(Identifier(String::from("b")))),
                    )),
                    Box::new(Expression::Identifier(Identifier(String::from("c")))),
                )),
            ),
            (
                "a * b / c",
                Statement::Expression(Expression::Infix(
                    Infix::Divide,
                    Box::new(Expression::Infix(
                        Infix::Multiply,
                        Box::new(Expression::Identifier(Identifier(String::from("a")))),
                        Box::new(Expression::Identifier(Identifier(String::from("b")))),
                    )),
                    Box::new(Expression::Identifier(Identifier(String::from("c")))),
                )),
            ),
            (
                "a + b / c",
                Statement::Expression(Expression::Infix(
                    Infix::Plus,
                    Box::new(Expression::Identifier(Identifier(String::from("a")))),
                    Box::new(Expression::Infix(
                        Infix::Divide,
                        Box::new(Expression::Identifier(Identifier(String::from("b")))),
                        Box::new(Expression::Identifier(Identifier(String::from("c")))),
                    )),
                )),
            ),
            (
                "a + b * c + d / e - f",
                Statement::Expression(Expression::Infix(
                    Infix::Minus,
                    Box::new(Expression::Infix(
                        Infix::Plus,
                        Box::new(Expression::Infix(
                            Infix::Plus,
                            Box::new(Expression::Identifier(Identifier(String::from("a")))),
                            Box::new(Expression::Infix(
                                Infix::Multiply,
                                Box::new(Expression::Identifier(Identifier(String::from("b")))),
                                Box::new(Expression::Identifier(Identifier(String::from("c")))),
                            )),
                        )),
                        Box::new(Expression::Infix(
                            Infix::Divide,
                            Box::new(Expression::Identifier(Identifier(String::from("d")))),
                            Box::new(Expression::Identifier(Identifier(String::from("e")))),
                        )),
                    )),
                    Box::new(Expression::Identifier(Identifier(String::from("f")))),
                )),
            ),
            (
                "5 > 4 == 3 < 4",
                Statement::Expression(Expression::Infix(
                    Infix::Equal,
                    Box::new(Expression::Infix(
                        Infix::GreaterThan,
                        Box::new(Expression::Literal(Literal::Int(5))),
                        Box::new(Expression::Literal(Literal::Int(4))),
                    )),
                    Box::new(Expression::Infix(
                        Infix::LessThan,
                        Box::new(Expression::Literal(Literal::Int(3))),
                        Box::new(Expression::Literal(Literal::Int(4))),
                    )),
                )),
            ),
            (
                "5 < 4 != 3 > 4",
                Statement::Expression(Expression::Infix(
                    Infix::NotEqual,
                    Box::new(Expression::Infix(
                        Infix::LessThan,
                        Box::new(Expression::Literal(Literal::Int(5))),
                        Box::new(Expression::Literal(Literal::Int(4))),
                    )),
                    Box::new(Expression::Infix(
                        Infix::GreaterThan,
                        Box::new(Expression::Literal(Literal::Int(3))),
                        Box::new(Expression::Literal(Literal::Int(4))),
                    )),
                )),
            ),
            (
                "3 + 4 * 5 == 3 * 1 + 4 * 5",
                Statement::Expression(Expression::Infix(
                    Infix::Equal,
                    Box::new(Expression::Infix(
                        Infix::Plus,
                        Box::new(Expression::Literal(Literal::Int(3))),
                        Box::new(Expression::Infix(
                            Infix::Multiply,
                            Box::new(Expression::Literal(Literal::Int(4))),
                            Box::new(Expression::Literal(Literal::Int(5))),
                        )),
                    )),
                    Box::new(Expression::Infix(
                        Infix::Plus,
                        Box::new(Expression::Infix(
                            Infix::Multiply,
                            Box::new(Expression::Literal(Literal::Int(3))),
                            Box::new(Expression::Literal(Literal::Int(1))),
                        )),
                        Box::new(Expression::Infix(
                            Infix::Multiply,
                            Box::new(Expression::Literal(Literal::Int(4))),
                            Box::new(Expression::Literal(Literal::Int(5))),
                        )),
                    )),
                )),
            ),
            (
                "true",
                Statement::Expression(Expression::Literal(Literal::Bool(true))),
            ),
            (
                "false",
                Statement::Expression(Expression::Literal(Literal::Bool(false))),
            ),
            (
                "3 > 5 == false",
                Statement::Expression(Expression::Infix(
                    Infix::Equal,
                    Box::new(Expression::Infix(
                        Infix::GreaterThan,
                        Box::new(Expression::Literal(Literal::Int(3))),
                        Box::new(Expression::Literal(Literal::Int(5))),
                    )),
                    Box::new(Expression::Literal(Literal::Bool(false))),
                )),
            ),
            (
                "3 < 5 == true",
                Statement::Expression(Expression::Infix(
                    Infix::Equal,
                    Box::new(Expression::Infix(
                        Infix::LessThan,
                        Box::new(Expression::Literal(Literal::Int(3))),
                        Box::new(Expression::Literal(Literal::Int(5))),
                    )),
                    Box::new(Expression::Literal(Literal::Bool(true))),
                )),
            ),
            (
                "1 + (2 + 3) + 4",
                Statement::Expression(Expression::Infix(
                    Infix::Plus,
                    Box::new(Expression::Infix(
                        Infix::Plus,
                        Box::new(Expression::Literal(Literal::Int(1))),
                        Box::new(Expression::Infix(
                            Infix::Plus,
                            Box::new(Expression::Literal(Literal::Int(2))),
                            Box::new(Expression::Literal(Literal::Int(3))),
                        )),
                    )),
                    Box::new(Expression::Literal(Literal::Int(4))),
                )),
            ),
            (
                "(5 + 5) * 2",
                Statement::Expression(Expression::Infix(
                    Infix::Multiply,
                    Box::new(Expression::Infix(
                        Infix::Plus,
                        Box::new(Expression::Literal(Literal::Int(5))),
                        Box::new(Expression::Literal(Literal::Int(5))),
                    )),
                    Box::new(Expression::Literal(Literal::Int(2))),
                )),
            ),
            (
                "2 / (5 + 5)",
                Statement::Expression(Expression::Infix(
                    Infix::Divide,
                    Box::new(Expression::Literal(Literal::Int(2))),
                    Box::new(Expression::Infix(
                        Infix::Plus,
                        Box::new(Expression::Literal(Literal::Int(5))),
                        Box::new(Expression::Literal(Literal::Int(5))),
                    )),
                )),
            ),
            (
                "-(5 + 5)",
                Statement::Expression(Expression::Prefix(
                    Prefix::Minus,
                    Box::new(Expression::Infix(
                        Infix::Plus,
                        Box::new(Expression::Literal(Literal::Int(5))),
                        Box::new(Expression::Literal(Literal::Int(5))),
                    )),
                )),
            ),
            (
                "!(true == true)",
                Statement::Expression(Expression::Prefix(
                    Prefix::Not,
                    Box::new(Expression::Infix(
                        Infix::Equal,
                        Box::new(Expression::Literal(Literal::Bool(true))),
                        Box::new(Expression::Literal(Literal::Bool(true))),
                    )),
                )),
            ),
        ];

        for (input, expect) in tests {
            let l = Lexer::new(input);
            let mut p = Parser::new(l);

            let program = p.parse_program();
            check_parse_errors(&mut p);

            assert_eq!(vec![expect], program);
        }
    }
}
