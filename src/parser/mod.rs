use crate::lexer::tokens::{Token,TokenType};
use std::iter::Peekable;
use std::vec::IntoIter;
use std::collections::HashMap;

pub mod function;
pub mod expression;

type TokenIter = Peekable<IntoIter<Token>>;

//the top-level
#[derive(Debug)]
pub enum Program{
    Extern(External),
    FunctionDef(Function),
    Expression(Box<ExprValue>) // Valid in top-level and in functions
}

#[derive(Debug)]
pub enum ExprValue{
    FnCall(String, HashMap<String,String>),
    UnOp(Box<TokenType>, Box<ExprValue>),
    Boolean(bool),
    Integer(i32),
    Str(String),
    Identifier(String),
    VarDecl{name:String, type_:String},
    IfElse{cond:Box<ExprValue>, if_:Box<ExprValue>, else_:Box<ExprValue>},
    Assign{name:String, value:Box<ExprValue>}
}

// 'extern' name (args) '->' return_type
#[derive(Debug)]
pub struct External {
    pub name: String,
    pub args: HashMap<String,String>,
    pub return_type: String,
}

// 'def' name (args) '->' return_type { statements}
#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub args: HashMap<String,String>,
    pub expressions: Vec<ExprValue>,
    pub return_type: String,
}

/// A parser that generates an abstract syntax tree.
pub struct Parser {
    tokens: TokenIter,
}

impl Parser {

    pub fn new(tokens: TokenIter) -> Self {
        Parser { tokens }
    }

}
