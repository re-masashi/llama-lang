use crate::lexer::tokens::{Token,TokenType};
use std::iter::Peekable;
use std::vec::IntoIter;
use std::collections::HashMap;

pub mod function;
pub mod expression;
pub mod program;

type TokenIter = Peekable<IntoIter<Token>>;

//the top-level
#[derive(Debug)]
pub enum AstNode{
    Extern(External),
    FunctionDef(Function),
    Expression(Box<ExprValue>) // Valid in top-level and in functions
}

#[derive(Debug)]
pub enum ExprValue{
    FnCall(String, Vec<ExprValue>),
    UnOp(Box<TokenType>, Box<ExprValue>),
    BinOp(Box<ExprValue>,Box<TokenType>,Box<ExprValue>),
    Boolean(bool),
    Integer(i32),
    Str(String),
    Identifier(String),
    VarDecl{name:String, type_:String},
    IfElse{cond:Box<ExprValue>, if_:Vec<ExprValue>, else_:Vec<ExprValue>},
    Assign{name:String, value:Box<ExprValue>},
    AugAssign{name:String, op: Box<TokenType>, value:Box<ExprValue>},
    Return(Box<ExprValue>)
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

    pub fn get_tok_precedence(&mut self, tok: TokenType) -> i32{
        match tok {
            TokenType::Equal
            |TokenType::NotEq
            |TokenType::Greater
            |TokenType::GreaterEq
            |TokenType::Less
            |TokenType::LessEq => 0,
            TokenType::Minus
            |TokenType::Plus => 1,
            TokenType::DivEq
            |TokenType::Mul => 2,
            any => panic!("Bad operator! Unknown {:?}", any),
        }
    }

}
