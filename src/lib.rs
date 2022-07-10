pub mod lexer;
pub mod parser;
pub mod codegen;

use std::collections::HashMap;

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

#[derive(Debug)]
pub struct SymbolTable{
    symbols: HashMap<String,Symbol>,
}


impl SymbolTable{

    fn new()->Self{
        SymbolTable{
            symbols: HashMap::new(),
        }
    }

    fn insert(&mut self, name:String, symbol: Symbol)->i32{
            self.symbols.insert(name, symbol);
            0
    }

    fn lookup(&mut self, name:String)->Option<Symbol>{
        match self.symbols.get(&name){
            Some(s) => Some(s.clone()),
            None   => None
        }
    }



}

#[derive(Clone,Debug)]
pub struct Symbol{
    type_: String,
    scope: String
}

impl Symbol{
    fn new(type_: String, scope:String) -> Self{
        Symbol{
            type_,
            scope
        }
    }
}