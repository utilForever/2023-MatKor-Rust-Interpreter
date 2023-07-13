extern crate monkey;
extern crate rustyline;

use monkey::lexer::lexer::Lexer;
use monkey::token::token::Token;
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

                let mut lexer = Lexer::new(&line);

                loop {
                    let tok = lexer.next_token();
                    if tok == Token::Eof {
                        break;
                    }

                    println!("{:?}", tok);
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
