use crate::parser::{Parser, AstNode};
use crate::lexer::tokens::{TokenType};
use crate::{unwrap_some, Result};

impl Parser{
	pub fn parse_program(&mut self) -> Result<Vec<AstNode>>{
		let mut ast:Vec<AstNode> = Vec::new();
		loop{
			match self.tokens.peek() {
				Some(s) => match s.type_{
					TokenType::Extern => {
						match self.parse_extern(){
							Ok(result) => {
								ast.insert(ast.len(), AstNode::Extern(result));
							},
							Err(e) if e == "EOF".to_string() => return Ok(ast),
							Err(e) => return Err(e),
						}
					}

					TokenType::Def => {
						match self.parse_function(){
							Ok(result) => {
								ast.insert(ast.len(), AstNode::FunctionDef(result));
							},
							Err(e) if e == "EOF".to_string() => return Ok(ast),
							Err(e) => return Err(e),
						}
					}

					_ => /*{
						match self.parse_expression(){
							Ok(result) => {
								match self.tokens.peek(){
									Some(t) if t.type_ == TokenType::Semicolon 
										=> self.tokens.next(), // eat ';'
									Some(_) => {
										let pos = unwrap_some!(self.tokens.peek()).pos;
										let line = unwrap_some!(self.tokens.peek()).line_no;
										return Err(format!(
											"Expected semicolon after expression. Before line {}:{}", line, pos).to_string());
									},
									None => return Err("EOF".to_string())
								};
								ast.insert(ast.len(), AstNode::Expression(Box::new(result)));
							},
							Err(e) if e == "EOF".to_string() => return Ok(ast),
							Err(e) => return Err(e),
						}
					}*/
					{
						println!("{:?}", self.tokens.peek());
						return Err("Only functions or expressions allowed at top-level.".to_string())
					}
				}
				None => return Ok(ast)				
			}
		};
	}
}
