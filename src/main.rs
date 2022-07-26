use std::process;
use llamac::{lexer::Lexer, parser::Parser, parser::AstNode, codegen::Compiler};
use inkwell::{context::Context};

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

#[no_mangle]
extern "C" fn printi32(x:i32) ->i32{
    println!("{:?}", x);
    x
}

pub fn main() {    
    let lexer = unwrap_or_exit!(Lexer::from_file("lexing.txt"), "IO");
    let tokens = lexer 
        .map(|t| unwrap_or_exit!(t, "Lexing"))
        .collect::<Vec<_>>();
    let mut parser = Parser::new(tokens.into_iter().peekable());
    let _ = parser.parse_program();

    let context = &Context::create();
    let builder = &context.create_builder();
    let module = &context.create_module("main_mod");

    let mut compiler = Compiler::new(context, builder, module);

    match parser.parse_program(){
        Ok(vec) => {
            for fun in vec.iter(){
                compiler.compile_fn(
                    match fun {
                        AstNode::FunctionDef(f) => f,
                        _ => unimplemented!(),
                    }
                );
            }
        }
        Err(_) => println!("{}","failed to compile"),
    };

    //println!("{:#?}", parser.parse_program());
    //println!("==========\n\n\n\n\n\n");
    //println!("{:#?}", parser.symtab);
}
