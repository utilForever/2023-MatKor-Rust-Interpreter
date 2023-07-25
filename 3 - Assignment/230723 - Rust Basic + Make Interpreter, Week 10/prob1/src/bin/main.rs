extern crate monkey;
extern crate rustyline;

use monkey::lexer::lexer::Lexer;
use monkey::parser::parser::Parser;
use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() {
    let mut rl = Editor::<()>::new();

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

                println!("{:?}", program);
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
