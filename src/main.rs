use std::process;
use llamac::{lexer::Lexer};

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
    for token in tokens{
        println!("{:?}",token );
    }
}
