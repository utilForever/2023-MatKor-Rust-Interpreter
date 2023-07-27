use crate::ast::ast::{Expression, Literal, Program, Statement};
use crate::evaluator::object::Object;

pub struct Evaluator {}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {}
    }

    pub fn eval(&mut self, program: Program) -> Option<Object> {
        let mut result = None;

        for statement in program {
            match self.eval_statement(statement) {
                object => result = object,
            }
        }

        result
    }

    fn eval_statement(&mut self, statement: Statement) -> Option<Object> {
        match statement {
            Statement::Expression(expression) => {
                let value = match self.eval_expression(expression) {
                    Some(value) => value,
                    None => return None,
                };

                Some(value)
            }
            _ => None,
        }
    }

    fn eval_expression(&mut self, expression: Expression) -> Option<Object> {
        match expression {
            Expression::Literal(literal) => Some(self.eval_literal(literal)),
            _ => None,
        }
    }

    fn eval_literal(&mut self, literal: Literal) -> Object {
        match literal {
            Literal::Int(value) => Object::Int(value),
            Literal::Bool(value) => Object::Bool(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::evaluator::evaluator::Evaluator;
    use crate::evaluator::object::Object;
    use crate::lexer::lexer::Lexer;
    use crate::parser::parser::Parser;

    fn eval(input: &str) -> Option<Object> {
        let mut e = Evaluator::new();
        e.eval(Parser::new(Lexer::new(input)).parse_program())
    }

    #[test]
    fn test_integer_expression() {
        let tests = vec![("5", Some(Object::Int(5))), ("10", Some(Object::Int(10)))];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }

    #[test]
    fn test_boolean_expression() {
        let tests = vec![
            ("true", Some(Object::Bool(true))),
            ("false", Some(Object::Bool(false))),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }
}
