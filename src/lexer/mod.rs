pub mod tokens;

use crate::lexer::tokens::{Token, TokenType};

use std::iter::Peekable;
use std::vec::IntoIter;
use std::{fs, io};

type Result<T> = std::result::Result<T, String>;

/// A lexical analyzer that splits the program into [`Token`]s.
///
/// [`Token`]: tokens/enum.Token.html
pub struct Lexer {
    /// The raw program characters.
    raw_data: Peekable<IntoIter<char>>,
    pos: usize,
}

impl Lexer {
    /// Create a lexer from a program file given the path to the file.
    ///
    /// # Arguments
    /// * `file_path` - The path to the program file.
    pub fn from_file(file_path: &str) -> io::Result<Self> {
        Ok(Self::from_text(&fs::read_to_string(file_path)?))
    }

    /// Create a lexer with the program data in plain text.
    ///
    /// # Arguments
    /// * `text` - The raw program.
    pub fn from_text(text: &str) -> Self {
        Lexer {
            raw_data: text.chars().collect::<Vec<_>>().into_iter().peekable(),
            pos: 0,
        }
    }

    /// Create a token by eating characters while a condition is met.
    ///
    /// # Arguments
    /// * `raw_token` - The raw string token to append characters to.
    /// * `cond` - The condition that must be met.
    fn get_next_char_while(&mut self, raw_token: &mut String, cond: fn(char) -> bool) {
        loop {
            match self.raw_data.peek() {
                Some(c) if cond(*c) => {
                    self.pos += 1;
                    raw_token.push(*c);
                    self.raw_data.next();
                }
                _ => {
                    break;
                }
            }
        }
    }

    /// Check if a character is a part of an identifier.
    ///
    /// Identifiers must start with an alphabetic character or underscore, and then can have
    /// alphanumeric characters and underscores.
    ///

    fn is_in_identifier(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }
}

impl Iterator for Lexer {
    type Item = Result<Token>;

    /// Identifies the next token
    fn next(&mut self) -> Option<Self::Item> {
        let token: Result<TokenType>;
        let current_char: char;
        // Find first non-whitespace character
        loop {
            match self.raw_data.next() {
                Some(c) if (c == ' ' || c == '\t' || c == '\n') => {
                    self.pos += 1;
                    continue;
                }
                // Comment
                Some(c) if c == '#' => {
                    let mut dump = String::new();
                    self.get_next_char_while(&mut dump, |c| c != '\n');
                    // println!("Lexing comment");
                    continue;
                }
                // At symbol (@)
                Some(c) if c == '@' => {
                    current_char = c;
                    self.pos += 1;
                    break;
                }
                Some(c) => {
                    current_char = c;
                    self.pos += 1;
                    break;
                }
                None => return None,
            }
        }

        // println!("First char: {}", current_char);

        // At symbol (@)
        if current_char == '@' {
            token = Ok(TokenType::At);
        }
        // Identifier
        else if Self::is_in_identifier(current_char) && !current_char.is_numeric() {
            let mut name = current_char.to_string();
            self.get_next_char_while(&mut name, Self::is_in_identifier);
            match name.as_str() {
                "if" => token = Ok(TokenType::If),
                "else" => token = Ok(TokenType::Else),
                "let" => token = Ok(TokenType::Let),
                "fn" => token = Ok(TokenType::Fn),
                "extern" => token = Ok(TokenType::Extern),
                "return" => token = Ok(TokenType::Return),
                "true" => token = Ok(TokenType::True),
                "false" => token = Ok(TokenType::False),
                "const" => token = Ok(TokenType::Const),
                "struct" => token = Ok(TokenType::Struct),
                "enum" => token = Ok(TokenType::Enum),
                "trait" => token = Ok(TokenType::Trait),
                "impl" => token = Ok(TokenType::Impl),
                "match" => token = Ok(TokenType::Match),
                "for" => token = Ok(TokenType::For),
                "while" => token = Ok(TokenType::While),
                "end" => token = Ok(TokenType::End),
                "as" => token = Ok(TokenType::As),
                "in" => token = Ok(TokenType::In),
                "bool" => token = Ok(TokenType::Bool),
                "self" => token = Ok(TokenType::SelfKeyword),
                "Self" => token = Ok(TokenType::SelfType),
                "do" => token = Ok(TokenType::Do),
                "break" => token = Ok(TokenType::Break),
                "continue" => token = Ok(TokenType::Continue),
                "import" => token = Ok(TokenType::Import),
                _ => token = Ok(TokenType::Identifier(name)),
            };
        }
        // Integer or Float Literal
        else if current_char.is_numeric() {
            let mut value = current_char.to_string();
            let mut is_float = false;

            loop {
                let next_char = self.raw_data.peek().copied();
                match next_char {
                    Some(c) if c.is_numeric() => {
                        value.push(c);
                        self.pos += 1;
                        self.raw_data.next();
                    }
                    Some(c) if c == '.' => {
                        let mut peek_clone = self.raw_data.clone();
                        peek_clone.next();
                        if let Some(next_c) = peek_clone.peek() {
                            if next_c.is_numeric() && !is_float {
                                is_float = true;
                                value.push('.');
                                self.pos += 1;
                                self.raw_data.next();
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    _ => break,
                }
            }

            if is_float {
                let float_val: f64 = value.parse().unwrap_or(0.0);
                token = Ok(TokenType::Float(float_val));
            } else {
                token = match value.parse::<i64>() {
                    Ok(i) => Ok(TokenType::Integer(i)),
                    Err(_) => Err(format!("Integer literal {} is invalid", value)),
                }
            }
        }
        // String Literal
        else if current_char == '"' {
            let mut value = String::new();

            self.get_next_char_while(&mut value, |c| c != '"');
            self.raw_data.next(); // Eat trailing "

            token = Ok(TokenType::Str(value));
        }
        // Semicolon
        else if current_char == ';' {
            token = Ok(TokenType::Semicolon);
        }
        // Comma
        else if current_char == ',' {
            token = Ok(TokenType::Comma);
        }
        // LParen
        else if current_char == '(' {
            token = Ok(TokenType::LParen);
        }
        // RParen
        else if current_char == ')' {
            token = Ok(TokenType::RParen);
        }
        // LBrack
        else if current_char == '[' {
            token = Ok(TokenType::LBrack);
        }
        // RBrack
        else if current_char == ']' {
            token = Ok(TokenType::RBrack);
        }
        // LBrace
        else if current_char == '{' {
            token = Ok(TokenType::LBrace);
        }
        // RBrace
        else if current_char == '}' {
            token = Ok(TokenType::RBrace);
        }
        // Plus and PlusEq
        else if current_char == '+' {
            if self.raw_data.peek() == Some(&'=') {
                self.raw_data.next(); // Eat =
                token = Ok(TokenType::PlusEq);
            } else {
                token = Ok(TokenType::Plus);
            }
        }
        // Minus, Arrow and MinusEq
        else if current_char == '-' {
            if self.raw_data.peek() == Some(&'=') {
                self.raw_data.next(); // Eat =
                token = Ok(TokenType::MinusEq);
            } else if self.raw_data.peek() == Some(&'>') {
                self.raw_data.next();
                token = Ok(TokenType::Arrow);
            } else {
                token = Ok(TokenType::Minus);
            }
        }
        // Mul and MulEq
        else if current_char == '*' {
            if self.raw_data.peek() == Some(&'=') {
                self.raw_data.next(); // Eat =
                token = Ok(TokenType::MulEq);
            } else {
                token = Ok(TokenType::Mul);
            }
        }
        // Div and DivEq
        else if current_char == '/' {
            if self.raw_data.peek() == Some(&'=') {
                self.raw_data.next(); // Eat =
                token = Ok(TokenType::DivEq);
            } else {
                token = Ok(TokenType::Div);
            }
        }
        // Less and LessEq
        else if current_char == '<' {
            if self.raw_data.peek() == Some(&'=') {
                self.raw_data.next(); // Eat =
                token = Ok(TokenType::LessEq);
            } else {
                token = Ok(TokenType::Less);
            }
        }
        // Greater and GreaterEq
        else if current_char == '>' {
            if self.raw_data.peek() == Some(&'=') {
                self.raw_data.next(); // Eat =
                token = Ok(TokenType::GreaterEq);
            } else {
                token = Ok(TokenType::Greater);
            }
        }
        // Assign and Equal
        else if current_char == '=' {
            if self.raw_data.peek() == Some(&'=') {
                self.raw_data.next(); // Eat =
                token = Ok(TokenType::Equal);
            } else {
                token = Ok(TokenType::Assign);
            }
        }
        // Not and NotEq
        else if current_char == '!' {
            if self.raw_data.peek() == Some(&'=') {
                self.raw_data.next(); // Eat =
                token = Ok(TokenType::NotEq);
            } else {
                token = Ok(TokenType::Not);
            }
        }
        // Dot (.)
        else if current_char == '.' {
            if self.raw_data.peek() == Some(&'.') {
                self.raw_data.next(); // Eat .
                token = Ok(TokenType::Spread);
            } else {
                token = Ok(TokenType::Dot);
            }
        }
        // Double colon (::)
        else if current_char == ':' {
            if self.raw_data.peek() == Some(&':') {
                self.raw_data.next(); // Eat :
                token = Ok(TokenType::DoubleColon);
            } else {
                token = Ok(TokenType::Colon);
            }
        }
        // Pipeline operator (|>)
        else if current_char == '|' {
            if self.raw_data.peek() == Some(&'>') {
                self.raw_data.next(); // Eat >
                token = Ok(TokenType::Pipeline);
            } else {
                token = Ok(TokenType::Unknown);
            }
        } else {
            token = Ok(TokenType::Unknown)
        }

        return Some(Ok(Token {
            type_: token.unwrap(),
            range: self.pos..self.pos,
        }));
    }
}

#[cfg(test)]
mod tests {

    use super::Lexer;
    use crate::lexer::tokens::TokenType;

    fn tokenize(text: &str) -> Vec<TokenType> {
        Lexer::from_text(text)
            .filter_map(|t| t.ok())
            .map(|t| t.type_)
            .collect()
    }

    #[test]
    fn is_in_identifier() {
        for &i in &['a', 'z', '_', '0', '9'] {
            assert!(Lexer::is_in_identifier(i));
        }

        for &s in &['+', '*', '@', ';'] {
            assert!(!Lexer::is_in_identifier(s));
        }
    }

    #[test]
    fn test_keywords() {
        let tokens = tokenize(
            "if else let fn extern return true false const struct enum trait impl match for while end as in bool self Self do break continue import",
        );
        assert!(matches!(&tokens[0], TokenType::If));
        assert!(matches!(&tokens[1], TokenType::Else));
        assert!(matches!(&tokens[2], TokenType::Let));
        assert!(matches!(&tokens[3], TokenType::Fn));
        assert!(matches!(&tokens[4], TokenType::Extern));
        assert!(matches!(&tokens[5], TokenType::Return));
        assert!(matches!(&tokens[6], TokenType::True));
        assert!(matches!(&tokens[7], TokenType::False));
        assert!(matches!(&tokens[8], TokenType::Const));
        assert!(matches!(&tokens[9], TokenType::Struct));
        assert!(matches!(&tokens[10], TokenType::Enum));
        assert!(matches!(&tokens[11], TokenType::Trait));
        assert!(matches!(&tokens[12], TokenType::Impl));
        assert!(matches!(&tokens[13], TokenType::Match));
        assert!(matches!(&tokens[14], TokenType::For));
        assert!(matches!(&tokens[15], TokenType::While));
        assert!(matches!(&tokens[16], TokenType::End));
        assert!(matches!(&tokens[17], TokenType::As));
        assert!(matches!(&tokens[18], TokenType::In));
        assert!(matches!(&tokens[19], TokenType::Bool));
        assert!(matches!(&tokens[20], TokenType::SelfKeyword));
        assert!(matches!(&tokens[21], TokenType::SelfType));
        assert!(matches!(&tokens[22], TokenType::Do));
        assert!(matches!(&tokens[23], TokenType::Break));
        assert!(matches!(&tokens[24], TokenType::Continue));
        assert!(matches!(&tokens[25], TokenType::Import));
    }

    #[test]
    fn test_identifiers() {
        let tokens = tokenize("x y_z someVar123 _private");
        assert!(matches!(&tokens[0], TokenType::Identifier(s) if s == "x"));
        assert!(matches!(&tokens[1], TokenType::Identifier(s) if s == "y_z"));
        assert!(matches!(&tokens[2], TokenType::Identifier(s) if s == "someVar123"));
        assert!(matches!(&tokens[3], TokenType::Identifier(s) if s == "_private"));
    }

    #[test]
    fn test_integer_literals() {
        let tokens = tokenize("0 42 100");
        assert!(matches!(&tokens[0], TokenType::Integer(0)));
        assert!(matches!(&tokens[1], TokenType::Integer(42)));
        assert!(matches!(&tokens[2], TokenType::Integer(100)));
    }

    #[test]
    fn test_float_literals() {
        let tokens = tokenize("3.14 0.5 1.0 100.0");
        assert!(matches!(&tokens[0], TokenType::Float(f) if (f - 3.14).abs() < 0.001));
        assert!(matches!(&tokens[1], TokenType::Float(f) if (f - 0.5).abs() < 0.001));
        assert!(matches!(&tokens[2], TokenType::Float(f) if (f - 1.0).abs() < 0.001));
        assert!(matches!(&tokens[3], TokenType::Float(f) if (f - 100.0).abs() < 0.001));
    }

    #[test]
    fn test_string_literals() {
        let tokens = tokenize("\"hello\" \"world\" \"with spaces\"");
        assert!(matches!(&tokens[0], TokenType::Str(s) if s == "hello"));
        assert!(matches!(&tokens[1], TokenType::Str(s) if s == "world"));
        assert!(matches!(&tokens[2], TokenType::Str(s) if s == "with spaces"));
    }

    #[test]
    fn test_punctuators() {
        let tokens = tokenize("; : :: , ( ) [ ] { } -> . .. |> @");
        assert!(matches!(&tokens[0], TokenType::Semicolon));
        assert!(matches!(&tokens[1], TokenType::Colon));
        assert!(matches!(&tokens[2], TokenType::DoubleColon));
        assert!(matches!(&tokens[3], TokenType::Comma));
        assert!(matches!(&tokens[4], TokenType::LParen));
        assert!(matches!(&tokens[5], TokenType::RParen));
        assert!(matches!(&tokens[6], TokenType::LBrack));
        assert!(matches!(&tokens[7], TokenType::RBrack));
        assert!(matches!(&tokens[8], TokenType::LBrace));
        assert!(matches!(&tokens[9], TokenType::RBrace));
        assert!(matches!(&tokens[10], TokenType::Arrow));
        assert!(matches!(&tokens[11], TokenType::Dot));
        assert!(matches!(&tokens[12], TokenType::Spread));
        assert!(matches!(&tokens[13], TokenType::Pipeline));
        assert!(matches!(&tokens[14], TokenType::At));
    }

    #[test]
    fn test_operators() {
        let tokens = tokenize("+ - * / = < > <= >= == ! != += -= *= /=");
        assert!(matches!(&tokens[0], TokenType::Plus));
        assert!(matches!(&tokens[1], TokenType::Minus));
        assert!(matches!(&tokens[2], TokenType::Mul));
        assert!(matches!(&tokens[3], TokenType::Div));
        assert!(matches!(&tokens[4], TokenType::Assign));
        assert!(matches!(&tokens[5], TokenType::Less));
        assert!(matches!(&tokens[6], TokenType::Greater));
        assert!(matches!(&tokens[7], TokenType::LessEq));
        assert!(matches!(&tokens[8], TokenType::GreaterEq));
        assert!(matches!(&tokens[9], TokenType::Equal));
        assert!(matches!(&tokens[10], TokenType::Not));
        assert!(matches!(&tokens[11], TokenType::NotEq));
        assert!(matches!(&tokens[12], TokenType::PlusEq));
        assert!(matches!(&tokens[13], TokenType::MinusEq));
        assert!(matches!(&tokens[14], TokenType::MulEq));
        assert!(matches!(&tokens[15], TokenType::DivEq));
    }

    #[test]
    fn test_comments() {
        let tokens = tokenize("x # this is a comment\n y");
        assert!(matches!(&tokens[0], TokenType::Identifier(s) if s == "x"));
        assert!(matches!(&tokens[1], TokenType::Identifier(s) if s == "y"));
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn test_function_definition() {
        let tokens = tokenize("fn add(x: i32, y: i32) -> i32 end");
        assert!(matches!(&tokens[0], TokenType::Fn));
        assert!(matches!(&tokens[1], TokenType::Identifier(s) if s == "add"));
        assert!(matches!(&tokens[2], TokenType::LParen));
        assert!(matches!(&tokens[3], TokenType::Identifier(s) if s == "x"));
        assert!(matches!(&tokens[4], TokenType::Colon));
        assert!(matches!(&tokens[5], TokenType::Identifier(s) if s == "i32"));
        assert!(matches!(&tokens[6], TokenType::Comma));
        assert!(matches!(&tokens[7], TokenType::Identifier(s) if s == "y"));
        assert!(matches!(&tokens[8], TokenType::Colon));
        assert!(matches!(&tokens[9], TokenType::Identifier(s) if s == "i32"));
        assert!(matches!(&tokens[10], TokenType::RParen));
        assert!(matches!(&tokens[11], TokenType::Arrow));
        assert!(matches!(&tokens[12], TokenType::Identifier(s) if s == "i32"));
        assert!(matches!(&tokens[13], TokenType::End));
    }

    #[test]
    fn test_let_binding() {
        let tokens = tokenize("let x: i32 = 42");
        assert!(matches!(&tokens[0], TokenType::Let));
        assert!(matches!(&tokens[1], TokenType::Identifier(s) if s == "x"));
        assert!(matches!(&tokens[2], TokenType::Colon));
        assert!(matches!(&tokens[3], TokenType::Identifier(s) if s == "i32"));
        assert!(matches!(&tokens[4], TokenType::Assign));
        assert!(matches!(&tokens[5], TokenType::Integer(42)));
    }

    #[test]
    fn test_struct_definition() {
        let tokens = tokenize("struct Point<T: Walkable> a: i32 b: f32 end");
        assert!(matches!(&tokens[0], TokenType::Struct));
        assert!(matches!(&tokens[1], TokenType::Identifier(s) if s == "Point"));
        assert!(matches!(&tokens[2], TokenType::Less));
        assert!(matches!(&tokens[3], TokenType::Identifier(s) if s == "T"));
        assert!(matches!(&tokens[4], TokenType::Colon));
        assert!(matches!(&tokens[5], TokenType::Identifier(s) if s == "Walkable"));
        assert!(matches!(&tokens[6], TokenType::Greater));
        assert!(matches!(&tokens[7], TokenType::Identifier(s) if s == "a"));
        assert!(matches!(&tokens[8], TokenType::Colon));
        assert!(matches!(&tokens[9], TokenType::Identifier(s) if s == "i32"));
        assert!(matches!(&tokens[10], TokenType::Identifier(s) if s == "b"));
        assert!(matches!(&tokens[11], TokenType::Colon));
        assert!(matches!(&tokens[12], TokenType::Identifier(s) if s == "f32"));
        assert!(matches!(&tokens[13], TokenType::End));
    }

    #[test]
    fn test_match_expression() {
        let tokens = tokenize("match x Some(y) = > 1, _ = > 2 end");
        assert!(matches!(&tokens[0], TokenType::Match));
        assert!(matches!(&tokens[1], TokenType::Identifier(s) if s == "x"));
        assert!(matches!(&tokens[2], TokenType::Identifier(s) if s == "Some"));
        assert!(matches!(&tokens[3], TokenType::LParen));
        assert!(matches!(&tokens[4], TokenType::Identifier(s) if s == "y"));
        assert!(matches!(&tokens[5], TokenType::RParen));
        assert!(matches!(&tokens[6], TokenType::Assign));
        assert!(matches!(&tokens[7], TokenType::Greater));
        assert!(matches!(&tokens[8], TokenType::Integer(1)));
        assert!(matches!(&tokens[9], TokenType::Comma));
        assert!(matches!(&tokens[10], TokenType::Identifier(s) if s == "_"));
        assert!(matches!(&tokens[11], TokenType::Assign));
        assert!(matches!(&tokens[12], TokenType::Greater));
        assert!(matches!(&tokens[13], TokenType::Integer(2)));
        assert!(matches!(&tokens[14], TokenType::End));
    }

    #[test]
    fn test_enum_path() {
        let tokens = tokenize("MyEnum::VariantOne");
        assert!(matches!(&tokens[0], TokenType::Identifier(s) if s == "MyEnum"));
        assert!(matches!(&tokens[1], TokenType::DoubleColon));
        assert!(matches!(&tokens[2], TokenType::Identifier(s) if s == "VariantOne"));
    }

    #[test]
    fn test_array_and_spread() {
        let tokens = tokenize("[1, 2, ..arr, 5]");
        assert!(matches!(&tokens[0], TokenType::LBrack));
        assert!(matches!(&tokens[1], TokenType::Integer(1)));
        assert!(matches!(&tokens[2], TokenType::Comma));
        assert!(matches!(&tokens[3], TokenType::Integer(2)));
        assert!(matches!(&tokens[4], TokenType::Comma));
        assert!(matches!(&tokens[5], TokenType::Spread));
        assert!(matches!(&tokens[6], TokenType::Identifier(s) if s == "arr"));
        assert!(matches!(&tokens[7], TokenType::Comma));
        assert!(matches!(&tokens[8], TokenType::Integer(5)));
        assert!(matches!(&tokens[9], TokenType::RBrack));
    }

    #[test]
    fn test_pipeline_operator() {
        let tokens = tokenize("x |> f(y, z) |> g.h");
        assert!(matches!(&tokens[0], TokenType::Identifier(s) if s == "x"));
        assert!(matches!(&tokens[1], TokenType::Pipeline));
        assert!(matches!(&tokens[2], TokenType::Identifier(s) if s == "f"));
        assert!(matches!(&tokens[3], TokenType::LParen));
        assert!(matches!(&tokens[4], TokenType::Identifier(s) if s == "y"));
        assert!(matches!(&tokens[5], TokenType::Comma));
        assert!(matches!(&tokens[6], TokenType::Identifier(s) if s == "z"));
        assert!(matches!(&tokens[7], TokenType::RParen));
        assert!(matches!(&tokens[8], TokenType::Pipeline));
        assert!(matches!(&tokens[9], TokenType::Identifier(s) if s == "g"));
        assert!(matches!(&tokens[10], TokenType::Dot));
        assert!(matches!(&tokens[11], TokenType::Identifier(s) if s == "h"));
    }

    #[test]
    fn test_attributes() {
        let tokens = tokenize("@doc(\"test\") @derive x");
        assert!(matches!(&tokens[0], TokenType::At));
        assert!(matches!(&tokens[1], TokenType::Identifier(s) if s == "doc"));
        assert!(matches!(&tokens[2], TokenType::LParen));
        assert!(matches!(&tokens[3], TokenType::Str(s) if s == "test"));
        assert!(matches!(&tokens[4], TokenType::RParen));
        assert!(matches!(&tokens[5], TokenType::At));
        assert!(matches!(&tokens[6], TokenType::Identifier(s) if s == "derive"));
        assert!(matches!(&tokens[7], TokenType::Identifier(s) if s == "x"));
    }

    #[test]
    fn test_for_loop() {
        let tokens = tokenize("for i in 0..10 println(i) end");
        assert!(matches!(&tokens[0], TokenType::For));
        assert!(matches!(&tokens[1], TokenType::Identifier(s) if s == "i"));
        assert!(matches!(&tokens[2], TokenType::In));
        assert!(matches!(&tokens[3], TokenType::Integer(0)));
        assert!(matches!(&tokens[4], TokenType::Spread));
        assert!(matches!(&tokens[5], TokenType::Integer(10)));
        assert!(matches!(&tokens[6], TokenType::Identifier(s) if s == "println"));
        assert!(matches!(&tokens[7], TokenType::LParen));
        assert!(matches!(&tokens[8], TokenType::Identifier(s) if s == "i"));
        assert!(matches!(&tokens[9], TokenType::RParen));
        assert!(matches!(&tokens[10], TokenType::End));
    }

    #[test]
    fn test_while_loop() {
        let tokens = tokenize("while x < 10 x = x + 1 end");
        assert!(matches!(&tokens[0], TokenType::While));
        assert!(matches!(&tokens[1], TokenType::Identifier(s) if s == "x"));
        assert!(matches!(&tokens[2], TokenType::Less));
        assert!(matches!(&tokens[3], TokenType::Integer(10)));
        assert!(matches!(&tokens[4], TokenType::Identifier(s) if s == "x"));
        assert!(matches!(&tokens[5], TokenType::Assign));
        assert!(matches!(&tokens[6], TokenType::Identifier(s) if s == "x"));
        assert!(matches!(&tokens[7], TokenType::Plus));
        assert!(matches!(&tokens[8], TokenType::Integer(1)));
        assert!(matches!(&tokens[9], TokenType::End));
    }

    #[test]
    fn test_impl_block() {
        let tokens = tokenize("impl Trait for Type fn method(self) end end");
        assert!(matches!(&tokens[0], TokenType::Impl));
        assert!(matches!(&tokens[1], TokenType::Identifier(s) if s == "Trait"));
        assert!(matches!(&tokens[2], TokenType::For));
        assert!(matches!(&tokens[3], TokenType::Identifier(s) if s == "Type"));
        assert!(matches!(&tokens[4], TokenType::Fn));
        assert!(matches!(&tokens[5], TokenType::Identifier(s) if s == "method"));
        assert!(matches!(&tokens[6], TokenType::LParen));
        assert!(matches!(&tokens[7], TokenType::SelfKeyword));
        assert!(matches!(&tokens[8], TokenType::RParen));
        assert!(matches!(&tokens[9], TokenType::End));
        assert!(matches!(&tokens[10], TokenType::End));
    }

    #[test]
    fn test_extern_function() {
        let tokens = tokenize("extern fn external_func(x: i32) -> i32");
        assert!(matches!(&tokens[0], TokenType::Extern));
        assert!(matches!(&tokens[1], TokenType::Fn));
        assert!(matches!(&tokens[2], TokenType::Identifier(s) if s == "external_func"));
        assert!(matches!(&tokens[3], TokenType::LParen));
        assert!(matches!(&tokens[4], TokenType::Identifier(s) if s == "x"));
        assert!(matches!(&tokens[5], TokenType::Colon));
        assert!(matches!(&tokens[6], TokenType::Identifier(s) if s == "i32"));
        assert!(matches!(&tokens[7], TokenType::RParen));
        assert!(matches!(&tokens[8], TokenType::Arrow));
        assert!(matches!(&tokens[9], TokenType::Identifier(s) if s == "i32"));
    }

    #[test]
    fn test_if_expression() {
        let tokens = tokenize("if x > 5 true else false end");
        assert!(matches!(&tokens[0], TokenType::If));
        assert!(matches!(&tokens[1], TokenType::Identifier(s) if s == "x"));
        assert!(matches!(&tokens[2], TokenType::Greater));
        assert!(matches!(&tokens[3], TokenType::Integer(5)));
        assert!(matches!(&tokens[4], TokenType::True));
        assert!(matches!(&tokens[5], TokenType::Else));
        assert!(matches!(&tokens[6], TokenType::False));
        assert!(matches!(&tokens[7], TokenType::End));
    }

    #[test]
    fn test_type_cast() {
        let tokens = tokenize("x as i32 y as f32");
        assert!(matches!(&tokens[0], TokenType::Identifier(s) if s == "x"));
        assert!(matches!(&tokens[1], TokenType::As));
        assert!(matches!(&tokens[2], TokenType::Identifier(s) if s == "i32"));
        assert!(matches!(&tokens[3], TokenType::Identifier(s) if s == "y"));
        assert!(matches!(&tokens[4], TokenType::As));
        assert!(matches!(&tokens[5], TokenType::Identifier(s) if s == "f32"));
    }
}
