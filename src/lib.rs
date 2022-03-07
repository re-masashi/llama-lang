pub mod lexer;
pub mod parser;
pub mod codegen;

#[macro_export]
macro_rules! unwrap_some {
    ($val:expr) => {
        match $val {
            Some(s) => s,
            None => return Err("EOF".to_string()),
        }
    };
}

pub type Result<T> = std::result::Result<T, String>;