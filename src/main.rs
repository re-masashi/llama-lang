use llamac::lexer::Lexer;
use llamac::parser::Parser;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = if args.len() > 1 {
        &args[1]
    } else {
        "lexing.lam"
    };

    let source = std::fs::read_to_string(filename).expect("Failed to read file");
    let tokens: Vec<_> = Lexer::from_text(&source).filter_map(|t| t.ok()).collect();

    println!("Tokens: {:?}", tokens);

    let mut parser = Parser::new(tokens, filename.to_string());
    match parser.parse() {
        Ok(program) => {
            println!("Parsed program successfully!");
            for node in &program {
                println!("{:?}", node);
            }
        }
        Err(e) => {
            println!("Parse error: {}", e);
        }
    }
}
