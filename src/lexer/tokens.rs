/// A token that is parsed by the [`Lexer`].
///
/// [`Lexer`]: ../struct.Lexer.html
#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    /// An identifier of a variable or function with its name.
    Identifier(String),
    /// Keywords
    If, // if
    Else,        // else
    Let,         // let
    Fn,          // fn
    Extern,      // extern
    Return,      // return
    True,        // true
    False,       // false
    Const,       // const
    Struct,      // struct
    Enum,        // enum
    Trait,       // trait
    Impl,        // impl
    Match,       // match
    For,         // for
    While,       // while
    End,         // end
    As,          // as
    In,          // in
    Bool,        // bool
    SelfKeyword, // self
    SelfType,    // Self
    Do,          // do
    Break,       // break
    Continue,    // continue
    Import,      // import

    /// Literals
    Integer(i64),
    Float(f64),
    Str(String),

    /// Punctuators
    At, // @
    Semicolon,
    Colon,
    DoubleColon, // ::
    Comma,
    LParen,   // (
    RParen,   // )
    LBrack,   // [
    RBrack,   // ]
    LBrace,   // {
    RBrace,   // }
    Arrow,    // ->
    Dot,      // .
    Spread,   // ..
    Pipeline, // |>

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
    MinusEq, // -=
    MulEq,   // *=
    DivEq,   // /=

    Unknown,
}

#[derive(Debug)]
pub struct Token {
    pub type_: TokenType,
    pub range: std::ops::Range<usize>,
}
