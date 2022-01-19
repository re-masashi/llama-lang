use crate::parser::{Parser, ExprValue};
use crate::lexer::tokens::{Token, TokenType};
use crate::{unwrap_some,Result};

impl Parser {

	pub fn parse_expression(&mut self)->Result<ExprValue>{
		unimplemented!();
		let l_value: Result<ExprValue> = match unwrap_some!(self.tokens.peek()).type_ {
			TokenType::LParen => {
				self.tokens.next();
				self.parse_paren_expression()
			},
			// Unary
			TokenType::Plus 
			| TokenType::Minus 
			| TokenType::Not => self.parse_unop(),

			TokenType::If => self.parse_if_else(),
			TokenType::Let => self.parse_declaration(),
			TokenType::True => self.parse_true(),
			TokenType::False => self.parse_false(),
			TokenType::Identifier(_) => self.parse_identifier(), // Parses identifiers and function calls as well
			// TokenType::Return => self.parse_return(),
			// Need to figure out what to do with Eof
		};
		// The functions above will eat the value, then we can proceed to check for a bin op.
		// match unwrap_some!(self.tokens.peek()){ }

	}

	pub fn parse_unop(&mut self) -> Result<ExprValue>{
		// Eat the operator while working.
		let op = match unwrap_some!(self.tokens.next()).type_{
			t => Box::new(t),
		};
		let expr = Box::new(self.parse_expression().unwrap());
		return Ok(ExprValue::UnOp(op, expr));
	}

	pub fn parse_paren_expression(&mut self) -> Result<ExprValue> {
		self.tokens.next(); // Eat '('
		let expr = self.parse_expression()
					.unwrap();
		if unwrap_some!(self.tokens.peek()).type_ == TokenType::RParen{
			self.tokens.next(); // Eat ')'
			return Ok(expr);
		}
		else {
			return Err("Missing closing parenthesis".to_string())
		}
	}

	pub fn parse_if_else(&mut self) -> Result<ExprValue> {
		self.tokens.next(); // Eat 'if'
		if unwrap_some!(self.tokens.next()).type_ == TokenType::LParen{
			self.tokens.next(); // Eat '('
		}
		else {
			return Err("Expected parenthesis".to_string());
		}

		let cond = self.parse_expression();

		if unwrap_some!(self.tokens.next()).type_ == TokenType::RParen{
			self.tokens.next(); // Eat ')'
		}
		else {
			return Err("Missing closing parenthesis".to_string());
		}

		if unwrap_some!(self.tokens.next()).type_ == TokenType::LBrace{
			self.tokens.next(); // Eat '{'
		}
		else {
			return Err("Expected '{' .".to_string());
		}

		let if_arm = self.parse_expression();

		if unwrap_some!(self.tokens.next()).type_ == TokenType::RBrace{
			self.tokens.next(); // Eat '}'
		}
		else {
			return Err("Missing closing '{'".to_string());
		}

		if unwrap_some!(self.tokens.next()).type_ == TokenType::Else{
			self.tokens.next(); // Eat 'else'
		}
		else {
			return Err("`if` without `else` not allowed.".to_string());
		}

	}

}