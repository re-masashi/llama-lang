use crate::parser::{Parser, External, Function, ExprValue};
use crate::lexer::tokens::{Token, TokenType};
use crate::{unwrap_some, Result, Symbol};
use std::collections::HashMap;

impl Parser {
	fn parse_type_annot(&mut self) -> Result<(String,String)>{
		// Check if Identifier exists, else return Err
		loop{
			match unwrap_some!(self.tokens.peek()){
		 		Token{
					type_:TokenType::Identifier(_),
					pos:_,
					line_no:_
				} => {break;}
		 	_ => return Err("Syntax Error: expected Identifier".to_string())
			}
		}
		// Store identifier.
		let name = match unwrap_some!(self.tokens.next()).type_{ TokenType::Identifier(s)=>s, _=>unreachable!()};
		// Check if colon exists.
		loop{match unwrap_some!(self.tokens.peek()) { Token{
				type_:TokenType::Colon,
				pos:_,
				line_no:_
			} => {break;} //ugly but works..
			_=> return Err("expected ':' .".to_string()),
		}}
		self.tokens.next(); // Eat ':'
		// Check if type exists
		loop{
			match unwrap_some!(self.tokens.peek()) {
				Token{
					type_:TokenType::Identifier(_),
					pos:_,
					line_no:_} => {break;}
				_ => return Err("expected Identifier.".to_string()),
			}
		}
		// Store type
		let type_ = match unwrap_some!(self.tokens.next()).type_{ TokenType::Identifier(s)=>s, _=>unreachable!()};
		return Ok((name, type_))
	}

	pub fn parse_extern(&mut self) -> Result<External>{
		let name: String;
		let return_type: String;
		let mut args = HashMap::new(); // Map<NAME, TYPE>
		match self.tokens.peek() {

		 	Some(Token{type_, pos:_, line_no:_ }) if type_ == &TokenType::Extern => {

		 		self.tokens.next(); // Eat extern

		 		loop{match unwrap_some!(self.tokens.peek()){
		 			Token{
						type_:TokenType::Identifier(_),
						pos:_,
						line_no:_
					} => {break;}
		 			_ => return Err("Syntax Error: expected Identifier after keyword 'extern'".to_string()),
		 		}} // This is ugly, but works... loop just to use break in match
		 		
                // Eat and store name
		 		match unwrap_some!(self.tokens.next()).type_{ 
		 			TokenType::Identifier(n) => name = n, // Always matches
		 			_ => unreachable!(), // never happens
		 		}
		 		
                if unwrap_some!(self.tokens.peek()).type_ != TokenType::LParen {
		 			return Err("Syntax Error: expected '(' after Identifier".to_string())
		 		}
		 		
                self.tokens.next(); // Eat '('
		 		
                if unwrap_some!(self.tokens.peek()).type_ == TokenType::RParen{
		 			self.tokens.next(); // Eat ')'
		 		}

		 		else {
		 			loop {
		 				if unwrap_some!(self.tokens.peek()).type_ == TokenType::Comma {
		 					self.tokens.next(); // Eat ','
		 					continue;
		 				}
		 				if unwrap_some!(self.tokens.peek()).type_ == TokenType::RParen{
		 					self.tokens.next(); // Eat ')'
		 					break;
		 				}
		 				let type_annot = self.parse_type_annot();
		 				match type_annot {
		 					Ok((n, t)) => args.insert(n, t),
		 					Err(e) => {
		 						return Err(e);
		 					},
		 				};
		 			}
		 		}

                if unwrap_some!(self.tokens.peek()).type_ != TokenType::Arrow{
		 			return Err("expected '->'".to_string())
		 		}

		 		self.tokens.next(); // Eat '->'

		 		match &unwrap_some!(self.tokens.peek()).type_{ 
		 			TokenType::Identifier(n) => {
		 				return_type = n.to_string()
		 			}, 
		 			_ => return Err("expected return type after extern".to_string()),
		 		}
		 		self.tokens.next(); // Eat the identifier

		 		if unwrap_some!(self.tokens.peek()).type_ == TokenType::Semicolon{
		 			self.tokens.next(); //Eat semicolon
		 		}
		 		else {
		 			return Err("Semicolon after extern is mandatory.".to_string())
		 		}
		 		self.symtab.insert(
		 			name.clone(), 
		 			Symbol::new(return_type.clone(),self.current_scope.clone())
		 		);
		 		return Ok(External{name:name, args:args, return_type: return_type})
		 	},
		 	_ => Err("PASS".to_string()),
		 }
	} // end of parse_extern

	pub fn parse_function(&mut self) -> Result<Function>{
		let name: String;
		let return_type: String;
		let mut args = HashMap::new(); // Map<NAME, TYPE> of type <String, String>
		let mut expressions: Vec<ExprValue> = Vec::new(); 
		match self.tokens.peek() {

			Some(Token{type_, pos:_, line_no:_ }) if type_ == &TokenType::Def => {

			 		self.tokens.next(); // Eat Def

			 		loop{
			 			match unwrap_some!(self.tokens.peek()){
				 			Token{
								type_:TokenType::Identifier(_),
								pos:_,
								line_no:_
							} => {break;}
				 			_ => return Err("Syntax Error: expected Identifier after keyword 'extern'".to_string()),
				 		}
				 	} // This is ugly, but works. `loop` just to use break in match
				 		
		            // Eat and store
				 	match unwrap_some!(self.tokens.next()).type_{ 
				 		TokenType::Identifier(n) => name = n, // Always matches
				 			_ => unreachable!(), // never happens
				 	}
				 	self.current_scope = name.clone();
				 		
		            if unwrap_some!(self.tokens.peek()).type_ != TokenType::LParen {
				 		return Err("Syntax Error: expected '(' after Identifier".to_string())
				 	}
				 		
		            self.tokens.next(); // Eat '('
				 		
		            if unwrap_some!(self.tokens.peek()).type_ == TokenType::RParen{
				 		self.tokens.next(); // Eat ')'
				 	}

				 	else {
				 		loop {
				 			if unwrap_some!(self.tokens.peek()).type_ == TokenType::Comma {
				 				self.tokens.next(); // Eat ','
				 				continue;
				 			}
				 			if unwrap_some!(self.tokens.peek()).type_ == TokenType::RParen{
				 				self.tokens.next(); // Eat ')'
				 				break;
				 			}
				 			let type_annot = self.parse_type_annot();
				 			match type_annot {
				 				Ok((n, t)) => args.insert(n, t),
				 				Err(e) => {
				 					return Err(e);
				 				},
				 			};
				 		}
				 	}

		            if unwrap_some!(self.tokens.peek()).type_ != TokenType::Arrow{
				 		return Err("expected '->'".to_string())
				 	}

				 	self.tokens.next(); // Eat '->'

				 	match &unwrap_some!(self.tokens.peek()).type_{ 
				 		TokenType::Identifier(n) => {return_type = n.to_string()}, 
				 		_ => return Err("expected return type_".to_string()),
				 	}
				 	self.tokens.next(); // Eat the return_type

		            if unwrap_some!(self.tokens.peek()).type_ != TokenType::LBrace{
				 		return Err("expected '{' in fn def".to_string());
				 	}

				 	self.tokens.next(); // Eat '{'
				 	loop {
				 		match self.parse_expression() {
				 			Ok(expr) => expressions.insert(expressions.len(),expr),
				 			Err(e) if e == "Invalid expression" => {
				 				if unwrap_some!(self.tokens.peek()).type_ == TokenType::RBrace{
				 					break;
				 				}
				 				else {
				 					return Err(e)
				 				}
				 			},
				 			Err(e) => return Err(e),

				 		}
				 		// Eat the semicolons
				 		match unwrap_some!(self.tokens.peek()).type_ {
				 			TokenType::Semicolon => {self.tokens.next(); continue;}, 
				 			_ => {break}
				 		}
				 	}
				 	if unwrap_some!(self.tokens.peek()).type_ != TokenType::RBrace{
				 		print!("{:?}", self.tokens.peek());
				 		return Err("expected '}'".to_string());
				 	}
				 	self.tokens.next(); // Eat Rbrace

				 	match self.tokens.peek() {
				 		Some(t) if t.type_ == TokenType::Semicolon => {self.tokens.next();}, // Eat semicolon, if present
				 		_ => {},
				 	}
				 	println!("{}",self.current_scope.clone());
				 	self.current_scope = "global".to_string();
				 	self.symtab.insert(name.clone(), Symbol::new(return_type.clone(), self.current_scope.clone()));
				 	return Ok(Function{name: name, args:args, expressions: expressions, return_type: return_type})
			},
			_ => Err("PASS".to_string()), //never happens
		}
	}

}
