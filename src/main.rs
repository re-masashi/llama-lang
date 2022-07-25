use std::process;
use llamac::{lexer::Lexer, parser::Parser, parser::AstNode, codegen::Compiler};

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
    let program = parser.parse_program();
    let compiler = Compiler::new();
    for program_ in program{
        match program {
            Ok(func) => match func{
                AstNode::FunctionDef(fun) => {
                    compiler.compile_fn(fun)
                }
            },
            Err(_) => println!("{}","failed to compile"),
        }
    }
    println!("{:#?}", parser.parse_program());
    println!("==========\n\n\n\n\n\n");
    println!("{:#?}", parser.symtab);
}
