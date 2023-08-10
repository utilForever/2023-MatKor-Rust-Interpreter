use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::ast::{Expression, Identifier, Infix, Literal, Prefix, Program, Statement};
use crate::evaluator::environment::Environment;
use crate::evaluator::object::Object;

pub struct Evaluator {
    environment: Rc<RefCell<Environment>>,
}

impl Evaluator {
    pub fn new(environment: Rc<RefCell<Environment>>) -> Self {
        Evaluator { environment }
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
            Statement::Let(identifier, expression) => {
                let value = match self.eval_expression(expression) {
                    Some(value) => value,
                    None => return None,
                };

                if Self::is_error(&value) {
                    Some(value)
                } else {
                    let Identifier(name) = identifier;
                    self.environment.borrow_mut().set(name, &value);

                    None
                }
            }
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
        }
    }

    fn eval_expression(&mut self, expression: Expression) -> Option<Object> {
        match expression {
            Expression::Identifier(identifier) => Some(self.eval_identifier(identifier)),
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
            Expression::Function { parameters, body } => Some(Object::Function(
                parameters,
                body,
                Rc::clone(&self.environment),
            )),
            Expression::Call {
                function,
                arguments,
            } => Some(self.eval_call_expression(function, arguments)),
        }
    }

    fn eval_identifier(&mut self, identifier: Identifier) -> Object {
        let Identifier(name) = identifier;

        match self.environment.borrow_mut().get(name.clone()) {
            Some(value) => value,
            None => Object::Error(String::from(format!("identifier not found: {name}"))),
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

    fn eval_call_expression(
        &mut self,
        function: Box<Expression>,
        arguments: Vec<Expression>,
    ) -> Object {
        let arguments = arguments
            .iter()
            .map(|expression| {
                self.eval_expression(expression.clone())
                    .unwrap_or(Object::Null)
            })
            .collect::<Vec<_>>();

        let (parameters, body, environment) = match self.eval_expression(*function) {
            Some(Object::Function(parameters, body, environment)) => {
                (parameters, body, environment)
            }
            Some(object) => return Self::error(format!("{object} is not valid function")),
            None => return Object::Null,
        };

        if parameters.len() != arguments.len() {
            return Self::error(format!(
                "wrong number of arguments: {} expected but {} given",
                parameters.len(),
                arguments.len(),
            ));
        }

        let current_env = Rc::clone(&self.environment);
        let mut scoped_env = Environment::new_with_outer(Rc::clone(&environment));
        let list = parameters.iter().zip(arguments.iter());

        for (_, (identifier, object)) in list.enumerate() {
            let Identifier(name) = identifier.clone();
            scoped_env.set(name, object);
        }

        self.environment = Rc::new(RefCell::new(scoped_env));

        let object = self.eval_block_statement(body);

        self.environment = current_env;

        match object {
            Some(object) => object,
            None => Object::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::ast::ast::{Expression, Identifier, Infix, Literal, Statement};
    use crate::evaluator::environment::Environment;
    use crate::evaluator::evaluator::Evaluator;
    use crate::evaluator::object::Object;
    use crate::lexer::lexer::Lexer;
    use crate::parser::parser::Parser;

    fn eval(input: &str) -> Option<Object> {
        let mut e = Evaluator::new(Rc::new(RefCell::new(Environment::new())));
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
    fn test_let_statement() {
        let tests = vec![
            ("let a = 5; a;", Some(Object::Int(5))),
            ("let a = 5 * 5; a;", Some(Object::Int(25))),
            ("let a = 5; let b = a; b;", Some(Object::Int(5))),
            (
                "let a = 5; let b = a; let c = a + b + 5; c;",
                Some(Object::Int(15)),
            ),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }

    #[test]
    fn test_function_object() {
        let input = "fn(x) { x + 2; };";

        assert_eq!(
            Some(Object::Function(
                vec![Identifier(String::from("x"))],
                vec![Statement::Expression(Expression::Infix(
                    Infix::Plus,
                    Box::new(Expression::Identifier(Identifier(String::from("x")))),
                    Box::new(Expression::Literal(Literal::Int(2))),
                ))],
                Rc::new(RefCell::new(Environment::new())),
            )),
            eval(input),
        )
    }

    #[test]
    fn test_function_application() {
        let tests = vec![
            (
                "let identity = fn(x) { x; }; identity(5);",
                Some(Object::Int(5)),
            ),
            (
                "let identity = fn(x) { return x; }; identity(5);",
                Some(Object::Int(5)),
            ),
            (
                "let double = fn(x) { x * 2; }; double(5);",
                Some(Object::Int(10)),
            ),
            (
                "let add = fn(x, y) { x + y; }; add(5, 5);",
                Some(Object::Int(10)),
            ),
            (
                "let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));",
                Some(Object::Int(20)),
            ),
            ("fn(x) { x; }(5)", Some(Object::Int(5))),
            (
                "fn(a) { let f = fn(b) { a + b }; f(a); }(5);",
                Some(Object::Int(10)),
            ),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }

    #[test]
    fn test_closures() {
        let input = r#"
let newAdder = fn(x) {
    fn(y) { x + y };
}

let addTwo = newAdder(2);
addTwo(2);
"#;

        assert_eq!(Some(Object::Int(4)), eval(input));
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

// let x = 5 * 5;
