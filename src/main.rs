use std::{process, path::Path};
use llamac::{lexer::Lexer, parser::Parser, parser::AstNode, codegen::Compiler};
use inkwell::{OptimizationLevel, context::Context, targets::{
    Target, TargetTriple, TargetMachine,InitializationConfig, RelocMode, CodeModel, FileType},
    execution_engine::JitFunction
};

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
#[no_mangle]
extern "C" fn printchar(x:i32)->i32{
    print!("{:?}", x as u8 as char);
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

    Target::initialize_x86(&InitializationConfig::default());
    let opt = OptimizationLevel::Default;
    let reloc = RelocMode::Default;
    let model = CodeModel::Default;
    let path = Path::new("out.a");
    let target = Target::from_name("x86-64").unwrap();
    let target_machine = target.create_target_machine(
        &TargetTriple::create("x86_64-pc-linux-gnu"),
        "x86-64",
        "+avx2",
        opt,
        reloc,
        model
    )
    .unwrap();
    target_machine.write_to_file(&module, FileType::Object, &path);

    

    //println!("{:#?}", parser.parse_program());
    //println!("==========\n\n\n\n\n\n");
    //println!("{:#?}", parser.symtab);
}
