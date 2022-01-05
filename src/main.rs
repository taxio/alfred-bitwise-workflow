use std::env;

pub mod error;
pub mod lexer;

fn main() {
    let args: Vec<String> = env::args().collect();
    let query = &args[1];

    println!("query: {}", query);

    let mut lex = lexer::Lexer::new(query);
    match lex.tokenize() {
        Ok(tokens) => {
            println!("token size: {}", tokens.len());
            for t in tokens {
                println!("{:?}", t);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
