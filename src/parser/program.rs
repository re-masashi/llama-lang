use crate::parser::{Parser, AstNode};
use crate::lexer::tokens::{TokenType};
use crate::{unwrap_some, Result};

impl Parser{
	pub fn parse_program(&mut self) -> Result<Vec<AstNode>>{
		let ast:Vec<AstNode> = Vec::new();
		loop{
			match unwrap_some!(self.tokens.peek()).type_ {
				TokenType::Extern => {
					match self.parse_extern(){
						Ok(result) => {
							ast.insert(args.len(), AstNode::result);
							if unwrap_some!(self.tokens.peek()) == TokenType::Semicolon{
								self.tokens.next();
							}
						},
						Err(e) if e == "EOF".to_string() => return ast,
						Err(e) => return e,
					}
				}

				TokenType::Def => {
					match self.parse_extern(){
						Ok(result) => ast.insert(args.len(), result),
						Err(e) if e == "EOF".to_string() => return ast,
						Err(e) => return e,
					}
				}

				_ => {
					match self.parse_expression(){
						Ok(result) => ast.insert(args.len(), result),
						Err(e) if e == "EOF".to_string() => return ast,
						Err(e) => return e,
					}
				}
			}
		}
	}
}
