/// A token that is parsed by the [`Lexer`].
///
/// [`Lexer`]: ../struct.Lexer.html
#[derive(Debug, PartialEq)]
pub enum TokenType {
    /// An identifier of a variable or function with its name.
    Identifier(String),
    /// Keywords
    If, // if
    Else,   // else
    Let,    // let
    Def,    // def
    Extern, // extern
    Return, // return
    True,   // true
    False,  // false

    /// Literals
    Integer(i32),
    Str(String),

    /// Punctuators
    Semicolon,
    Colon,
    Comma,
    LParen, // (
    RParen, // )
    LBrack, // [
    RBrack, // ]
    LBrace, // {
    RBrace, // }
    Arrow,  // ->

    /// Operators
    Minus,
    Plus,
    Div,
    Mul,
    Assign,    // =
    Less,      // <
    Greater,   // >
    LessEq,    // <=
    GreaterEq, // >=
    Equal,     // ==
    Not,       // !
    NotEq,     // !=
    /// AugAssign operators
    PlusEq, // +=
    MinusEq,   // -=
    MulEq,     // *=
    DivEq,     // /=

    Eof,

    Unknown,
}

#[derive(Debug)]
pub struct Token {
    pub type_: TokenType,
    pub pos: i32,
    pub line_no: i32,
}
