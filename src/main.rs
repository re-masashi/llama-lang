use std::process;
use llamac::{lexer::Lexer, parser::Parser};

macro_rules! unwrap_or_exit {
    ($f:expr, $origin:tt) => {
        match $f {
            Ok(a) => a,
            Err(e) => {
                eprintln!("{}: {}", $origin, e);
                process::exit(1);
            }
        }
    };
}

pub fn main() {    
    let lexer = unwrap_or_exit!(Lexer::from_file("lexing.txt"), "IO");
    let tokens = lexer 
        .map(|t| unwrap_or_exit!(t, "Lexing"))
        .collect::<Vec<_>>();
    let mut parser = Parser::new(tokens.into_iter().peekable());
    println!("{:#?}", parser.parse_program());
}
