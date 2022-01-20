use crate::parser::{Parser, ExprValue};
use crate::lexer::tokens::{TokenType};
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

			TokenType::Identifier(_) => self.parse_identifier(), // Parses identifiers, assignments and function calls as well

			TokenType::Return => self.parse_return(),

			_ => unreachable!()
		};
		// The functions above will eat the value, then we can proceed to check for a bin op.
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

		let cond = Box::new(self.parse_expression().unwrap());

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

		let if_arm = Box::new(self.parse_expression().unwrap());

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

		if unwrap_some!(self.tokens.next()).type_ == TokenType::LBrace{
			self.tokens.next(); // Eat '{'
		}
		else {
			return Err("Expected '{'.".to_string());
		}

		let else_arm = Box::new(self.parse_expression().unwrap());

		if unwrap_some!(self.tokens.next()).type_ == TokenType::RBrace{
			self.tokens.next(); // Eat '}'
		}
		else {
			return Err("Missing closing '}'.".to_string());
		}
		return Ok(ExprValue::IfElse{cond:cond, if_:if_arm, else_:else_arm});
	}

	pub fn parse_declaration(&mut self) -> Result<ExprValue> {
		self.tokens.next(); // Eat `let`
		let name: String;
		let type_:String;

		match unwrap_some!(self.tokens.next()).type_ {
			TokenType::Identifier(n) => name = n,
			_ => return Err("Expected an identifier".to_string()),
		}

		if unwrap_some!(self.tokens.peek()).type_ == TokenType::Colon{
			self.tokens.next(); // Eat ':'
		}
		else {
			return Err("Missing ':'.".to_string());
		}

		match unwrap_some!(self.tokens.next()).type_ {
			TokenType::Identifier(t) => type_ = t,
			_ => return Err("Expected an identifier".to_string()),
		}
		return Ok(ExprValue::VarDecl{name:name,type_:type_})
	}

	pub fn parse_true(&mut self) -> Result<ExprValue> {
		self.tokens.next(); // Eat `true`
		return Ok(ExprValue::Boolean(true));
	}

	pub fn parse_false(&mut self) -> Result<ExprValue>{
		self.tokens.next(); // Eat `false`
		return Ok(ExprValue::Boolean(false));
	}

	pub fn parse_identifier(&mut self) -> Result<ExprValue> {
		// Eat the identifier and work.
		let name = match unwrap_some!(self.tokens.next()).type_ {
			TokenType::Identifier(n) => n,
			_ => unreachable!()
		};
		// Check for assignment
		match unwrap_some!(self.tokens.peek()).type_{
			TokenType::Assign => {
				self.tokens.next(); // Eat '='
				let value = Box::new(self.parse_expression().unwrap());
				return Ok(ExprValue::Assign{name:name, value:value})
			}
			TokenType::PlusEq => {
				let op = Box::new(unwrap_some!(self.tokens.next()).type_); // Eat '+='
				let value = Box::new(self.parse_expression().unwrap());
				return Ok(ExprValue::AugAssign{name:name, op:op, value:value})
			}
			TokenType::MinusEq => {
				let op = Box::new(unwrap_some!(self.tokens.next()).type_); // Eat '-='
				let value = Box::new(self.parse_expression().unwrap());
				return Ok(ExprValue::AugAssign{name:name, op:op, value:value})
			}
			TokenType::DivEq => {
				let op = Box::new(unwrap_some!(self.tokens.next()).type_); // Eat '/='
				let value = Box::new(self.parse_expression().unwrap());
				return Ok(ExprValue::AugAssign{name:name, op:op, value:value})
			}
			TokenType::MulEq => {
				let op = Box::new(unwrap_some!(self.tokens.next()).type_); // Eat '*='
				let value = Box::new(self.parse_expression().unwrap());
				return Ok(ExprValue::AugAssign{name:name, op:op, value:value})
			}
			_ => {}
		}
		// Check for function call
		if unwrap_some!(self.tokens.peek()).type_ == TokenType::LParen{
			self.tokens.next(); // Eat '('
			let mut values = Vec::new();
			let arg1; 
			match self.parse_expression(){
				Ok(expr) => arg1 = expr,
				Err(_) => {
					if unwrap_some!(self.tokens.peek()).type_ == TokenType::RParen{
						self.tokens.next();
						return Ok(ExprValue::FnCall(name, values));
					}
					return Err("Invalid Function call".to_string())
				}
			};
			values.insert(values.len(), arg1);
			loop {
				if unwrap_some!(self.tokens.peek()).type_ == TokenType::Comma{
					self.tokens.next(); // Eat ','
					values.insert(values.len(),self.parse_expression().unwrap());
				} 
				else if unwrap_some!(self.tokens.peek()).type_ == TokenType::RParen{
					self.tokens.next(); // Eat ')'
				} 
				else {
					return Err("Invalid function call.".to_string())
				}

				return Ok(ExprValue::FnCall(name, values))
			}
		}
		return Err("Expected ',' or '('".to_string() )
	}

	pub fn parse_return(&mut self) -> Result<ExprValue>{
		self.tokens.next(); // Eat `return`
		let expr = Box::new(self.parse_expression().unwrap());
		return Ok(ExprValue::Return(expr));
	}

}