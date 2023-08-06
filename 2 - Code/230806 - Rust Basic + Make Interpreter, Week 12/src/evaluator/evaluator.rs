use crate::ast::ast::{Expression, Infix, Literal, Prefix, Program, Statement};
use crate::evaluator::object::Object;

pub struct Evaluator {}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {}
    }

    fn is_truthy(object: Object) -> bool {
        match object {
            Object::Null | Object::Bool(false) => false,
            _ => true,
        }
    }

    fn error(msg: String) -> Object {
        Object::Error(msg)
    }

    fn is_error(object: &Object) -> bool {
        match object {
            Object::Error(_) => true,
            _ => false,
        }
    }

    pub fn eval(&mut self, program: Program) -> Option<Object> {
        let mut result = None;

        for statement in program {
            match self.eval_statement(statement) {
                Some(Object::ReturnValue(value)) => return Some(*value),
                Some(Object::Error(msg)) => return Some(Object::Error(msg)),
                object => result = object,
            }
        }

        result
    }

    fn eval_block_statement(&mut self, statements: Vec<Statement>) -> Option<Object> {
        let mut result = None;

        for statement in statements {
            match self.eval_statement(statement) {
                Some(Object::ReturnValue(value)) => return Some(Object::ReturnValue(value)),
                Some(Object::Error(msg)) => return Some(Object::Error(msg)),
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
            Statement::Return(expression) => {
                let value = match self.eval_expression(expression) {
                    Some(value) => value,
                    None => return None,
                };

                if Self::is_error(&value) {
                    Some(value)
                } else {
                    Some(Object::ReturnValue(Box::new(value)))
                }
            }
            _ => None,
        }
    }

    fn eval_expression(&mut self, expression: Expression) -> Option<Object> {
        match expression {
            Expression::Literal(literal) => Some(self.eval_literal(literal)),
            Expression::Prefix(prefix, right_expression) => {
                if let Some(right) = self.eval_expression(*right_expression) {
                    Some(self.eval_prefix_expression(prefix, right))
                } else {
                    None
                }
            }
            Expression::Infix(infix, left_expression, right_expression) => {
                let left = self.eval_expression(*left_expression);
                let right = self.eval_expression(*right_expression);

                if left.is_some() && right.is_some() {
                    Some(self.eval_infix_expression(infix, left.unwrap(), right.unwrap()))
                } else {
                    None
                }
            }
            Expression::If {
                condition,
                consequence,
                alternative,
            } => self.eval_if_expression(*condition, consequence, alternative),
            _ => None,
        }
    }

    fn eval_literal(&mut self, literal: Literal) -> Object {
        match literal {
            Literal::Int(value) => Object::Int(value),
            Literal::Bool(value) => Object::Bool(value),
        }
    }

    fn eval_prefix_expression(&mut self, prefix: Prefix, right: Object) -> Object {
        match prefix {
            Prefix::Not => self.eval_not_operator_expression(right),
            Prefix::Minus => self.eval_minus_prefix_expression(right),
        }
    }

    fn eval_not_operator_expression(&mut self, right: Object) -> Object {
        match right {
            Object::Bool(true) => Object::Bool(false),
            Object::Bool(false) => Object::Bool(true),
            Object::Null => Object::Bool(true),
            _ => Object::Bool(false),
        }
    }

    fn eval_minus_prefix_expression(&mut self, right: Object) -> Object {
        match right {
            Object::Int(value) => Object::Int(-value),
            _ => Self::error(format!("unknown operator: -{right}")),
        }
    }

    fn eval_infix_expression(&mut self, infix: Infix, left: Object, right: Object) -> Object {
        match left {
            Object::Int(left_value) => {
                if let Object::Int(right_value) = right {
                    self.eval_infix_integer_expression(infix, left_value, right_value)
                } else {
                    Self::error(format!("type mismatch: {left} {infix} {right}"))
                }
            }
            Object::Bool(left_value) => {
                if let Object::Bool(right_value) = right {
                    self.eval_infix_boolean_expression(infix, left_value, right_value)
                } else {
                    Self::error(format!("type mismatch: {left} {infix} {right}"))
                }
            }
            _ => Self::error(format!("unknown operator: {left} {infix} {right}")),
        }
    }

    fn eval_infix_integer_expression(
        &mut self,
        infix: Infix,
        left_value: i64,
        right_value: i64,
    ) -> Object {
        match infix {
            Infix::Plus => Object::Int(left_value + right_value),
            Infix::Minus => Object::Int(left_value - right_value),
            Infix::Multiply => Object::Int(left_value * right_value),
            Infix::Divide => Object::Int(left_value / right_value),
            Infix::Equal => Object::Bool(left_value == right_value),
            Infix::NotEqual => Object::Bool(left_value != right_value),
            Infix::LessThan => Object::Bool(left_value < right_value),
            Infix::GreaterThan => Object::Bool(left_value > right_value),
        }
    }

    fn eval_infix_boolean_expression(
        &mut self,
        infix: Infix,
        left_value: bool,
        right_value: bool,
    ) -> Object {
        match infix {
            Infix::Equal => Object::Bool(left_value == right_value),
            Infix::NotEqual => Object::Bool(left_value != right_value),
            _ => Self::error(format!(
                "unknown operator: {left_value} {infix} {right_value}",
            )),
        }
    }

    fn eval_if_expression(
        &mut self,
        condition: Expression,
        consquence: Vec<Statement>,
        alternative: Option<Vec<Statement>>,
    ) -> Option<Object> {
        let condition = match self.eval_expression(condition) {
            Some(condition) => condition,
            None => return None,
        };

        if Self::is_truthy(condition) {
            self.eval_block_statement(consquence)
        } else if let Some(alternative) = alternative {
            self.eval_block_statement(alternative)
        } else {
            None
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
        let tests = vec![
            ("5", Some(Object::Int(5))),
            ("10", Some(Object::Int(10))),
            ("-5", Some(Object::Int(-5))),
            ("-10", Some(Object::Int(-10))),
            ("5 + 5 + 5 + 5 - 10", Some(Object::Int(10))),
            ("2 * 2 * 2 * 2 * 2", Some(Object::Int(32))),
            ("-50 + 100 + -50", Some(Object::Int(0))),
            ("5 * 2 + 10", Some(Object::Int(20))),
            ("5 + 2 * 10", Some(Object::Int(25))),
            ("20 + 2 * -10", Some(Object::Int(0))),
            ("50 / 2 * 2 + 10", Some(Object::Int(60))),
            ("2 * (5 + 10)", Some(Object::Int(30))),
            ("3 * 3 * 3 + 10", Some(Object::Int(37))),
            ("3 * (3 * 3) + 10", Some(Object::Int(37))),
            ("(5 + 10 * 2 + 15 / 3) * 2 + -10", Some(Object::Int(50))),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }

    #[test]
    fn test_boolean_expression() {
        let tests = vec![
            ("true", Some(Object::Bool(true))),
            ("false", Some(Object::Bool(false))),
            ("1 < 2", Some(Object::Bool(true))),
            ("1 > 2", Some(Object::Bool(false))),
            ("1 < 1", Some(Object::Bool(false))),
            ("1 > 1", Some(Object::Bool(false))),
            ("1 == 1", Some(Object::Bool(true))),
            ("1 != 1", Some(Object::Bool(false))),
            ("1 == 2", Some(Object::Bool(false))),
            ("1 != 2", Some(Object::Bool(true))),
            ("true == true", Some(Object::Bool(true))),
            ("false == false", Some(Object::Bool(true))),
            ("true == false", Some(Object::Bool(false))),
            ("true != false", Some(Object::Bool(true))),
            ("false != true", Some(Object::Bool(true))),
            ("(1 < 2) == true", Some(Object::Bool(true))),
            ("(1 < 2) == false", Some(Object::Bool(false))),
            ("(1 > 2) == true", Some(Object::Bool(false))),
            ("(1 > 2) == false", Some(Object::Bool(true))),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }

    #[test]
    fn test_not_operator() {
        let tests = vec![
            ("!true", Some(Object::Bool(false))),
            ("!false", Some(Object::Bool(true))),
            ("!!true", Some(Object::Bool(true))),
            ("!!false", Some(Object::Bool(false))),
            ("!!5", Some(Object::Bool(true))),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }

    #[test]
    fn test_if_else_expression() {
        let tests = vec![
            ("if (true) { 10 }", Some(Object::Int(10))),
            ("if (false) { 10 }", None),
            ("if (1) { 10 }", Some(Object::Int(10))),
            ("if (1 < 2) { 10 }", Some(Object::Int(10))),
            ("if (1 > 2) { 10 }", None),
            ("if (1 > 2) { 10 } else { 20 }", Some(Object::Int(20))),
            ("if (1 < 2) { 10 } else { 20 }", Some(Object::Int(10))),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }

    #[test]
    fn test_return_statement() {
        let tests = vec![
            ("return 10;", Some(Object::Int(10))),
            ("return 10; 9;", Some(Object::Int(10))),
            ("return 2 * 5; 9;", Some(Object::Int(10))),
            ("9; return 2 * 5; 9;", Some(Object::Int(10))),
            (
                r#"
if (10 > 1) {
    if (10 > 1) {
        return 10;
    }

    return 1;
}"#,
                Some(Object::Int(10)),
            ),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }

    #[test]
    fn test_error_handling() {
        let tests = vec![
            (
                "5 + true",
                Some(Object::Error(String::from("type mismatch: 5 + true"))),
            ),
            (
                "5 + true; 5;",
                Some(Object::Error(String::from("type mismatch: 5 + true"))),
            ),
            (
                "-true",
                Some(Object::Error(String::from("unknown operator: -true"))),
            ),
            (
                "5; true + false; 5;",
                Some(Object::Error(String::from(
                    "unknown operator: true + false",
                ))),
            ),
            (
                "if (10 > 1) { true + false; }",
                Some(Object::Error(String::from(
                    "unknown operator: true + false",
                ))),
            ),
            (
                r#"
if (10 > 1) {
    if (10 > 1) {
        return true + false;
    }

    return 1;
}"#,
                Some(Object::Error(String::from(
                    "unknown operator: true + false",
                ))),
            ),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }
}
