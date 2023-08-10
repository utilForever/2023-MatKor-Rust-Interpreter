extern crate monkey;
extern crate rustyline;

use std::cell::RefCell;
use std::rc::Rc;

use monkey::evaluator::environment::Environment;
use monkey::evaluator::evaluator::Evaluator;
use monkey::lexer::lexer::Lexer;
use monkey::parser::parser::Parser;
use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() {
    let mut rl = Editor::<()>::new();
    let environment = Environment::new();
    let mut evaluator = Evaluator::new(Rc::new(RefCell::new(environment)));

    println!("Hello! This is the Monkey programming language!");
    println!("Feel free to type in commands\n");

    loop {
        match rl.readline(">> ") {
            Ok(line) => {
                rl.add_history_entry(&line);

                let mut parser = Parser::new(Lexer::new(&line));
                let program = parser.parse_program();
                let errors = parser.get_errors();

                if errors.len() > 0 {
                    for err in errors {
                        println!("{err}");
                    }

                    continue;
                }

                if let Some(evaluated) = evaluator.eval(program) {
                    println!("{evaluated}\n");
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("\nBye :)");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!();
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }
    }
}
