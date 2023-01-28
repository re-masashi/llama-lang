use crate::lexer::tokens::{Token,TokenType};
use crate::SymbolTable;
use crate::unwrap_some;
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
}

#[derive(Debug)]
pub enum ExprValue{
    FnCall(String, Vec<ExprValue>), // Done codegen
    UnOp(Box<TokenType>, Box<ExprValue>), // Done codegen
    BinOp(Box<ExprValue>,Box<TokenType>,Box<ExprValue>), // Done codegen
    Boolean(bool), // Done codegen
    Integer(i32), // Done codegen 
    Str(String), // Done codegen
    Identifier(String), // Done codegen
    VarDecl{name:String, type_:String}, // Done codegen
    IfElse{cond:Box<ExprValue>, if_:Vec<ExprValue>, else_:Vec<ExprValue>}, // Done codegen
    Assign{name:String, value:Box<ExprValue>}, // Done codegen
    AugAssign{name:String, op: Box<TokenType>, value:Box<ExprValue>},
    Return(Box<ExprValue>) 
}

// 'extern' name (args) '->' return_type
#[derive(Debug)]
pub struct External {
    pub name: String,
    pub args: Args,
    pub return_type: String,
}

// 'def' name (args) '->' return_type { expressions}
#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub args: Args,
    pub expressions: Vec<ExprValue>,
    pub return_type: String,
}

/// A parser that generates an abstract syntax tree.
pub struct Parser {
    tokens: TokenIter,
    pub symtab: SymbolTable,
    current_scope: String,
    pos: i32,
    line_no: i32,
}

#[derive(Debug)]
pub struct Args {
    pub name: Vec<String>,
    pub type_: Vec<String>
} // I will this improvise later.

impl Parser {

    pub fn new(tokens: TokenIter) -> Self {
        Parser { tokens, symtab: SymbolTable::new(), current_scope:"global".to_string(),             pos: -1,
            line_no: 1,}
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

    fn advance(&mut self){
        self.pos = match self.tokens.peek(){
                Some(t)=>t,
                None => panic!("Dunno"),
            }.pos;
        self.line_no = match self.tokens.peek(){
                Some(t)=>t,
                None => panic!("Dunno"),
            }.line_no;
    }

}
