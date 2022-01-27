use crate::parser::{Parser, AstNode};
use crate::lexer::tokens::{TokenType};
use crate::{unwrap_some, Result};

impl Parser{
	pub fn parse_program(&mut self) -> Result<Vec<AstNode>>{
		let mut ast:Vec<AstNode> = Vec::new();
		loop{
			match unwrap_some!(self.tokens.peek()).type_ {
				TokenType::Extern => {
					match self.parse_extern(){
						Ok(result) => {
							ast.insert(ast.len(), AstNode::Extern(result));
							if unwrap_some!(self.tokens.peek()).type_ == TokenType::Semicolon{
								self.tokens.next();
							}
						},
						Err(e) if e == "EOF".to_string() => return Ok(ast),
						Err(e) => return Err(e),
					}
				}

				TokenType::Def => {
					match self.parse_function(){
						Ok(result) => ast.insert(ast.len(), AstNode::FunctionDef(result)),
						Err(e) if e == "EOF".to_string() => return Ok(ast),
						Err(e) => return Err(e),
					}
				}

				_ => {
					match self.parse_expression(){
						Ok(result) => {
							ast.insert(ast.len(), AstNode::Expression(Box::new(result)));
						},
						Err(e) if e == "EOF".to_string() => return Ok(ast),
						Err(e) => return Err(e),
					}
				}
			}
		};
	}
}
