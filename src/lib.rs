pub mod lexer;
pub mod parser;

#[macro_export]
macro_rules! unwrap_some {
    ($val:expr) => {
        match $val {
            Some(s) => s,
            _ => panic!(),
        }
    };
}

pub type Result<T> = std::result::Result<T, String>;