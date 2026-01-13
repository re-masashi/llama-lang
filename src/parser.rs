use crate::ast::*;
use crate::lexer::tokens::{Token, TokenType};
use crate::meta::{Attribute, AttributeArg, Meta};

use std::iter::Peekable;
use std::vec::IntoIter;

pub type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub range: Option<std::ops::Range<usize>>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ParseError {}

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
    filename: String,
    errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, filename: String) -> Self {
        Parser {
            tokens: tokens.into_iter().peekable(),
            filename,
            errors: Vec::new(),
        }
    }

    pub fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    pub fn peek_type(&mut self) -> Option<TokenType> {
        self.peek().map(|t| t.type_.clone())
    }

    pub fn advance(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    pub fn consume(&mut self) -> Result<Token> {
        self.advance().ok_or_else(|| ParseError {
            message: "Unexpected end of input".to_string(),
            range: None,
        })
    }

    pub fn consume_type(&mut self, expected: TokenType) -> Result<TokenType> {
        let token = self.consume()?;
        if token.type_ == expected {
            Ok(token.type_)
        } else {
            Err(ParseError {
                message: format!("Expected {:?}, found {:?}", expected, token.type_),
                range: Some(token.range.clone()),
            })
        }
    }

    pub fn consume_identifier(&mut self) -> Result<String> {
        let token = self.consume()?;
        match token.type_ {
            TokenType::Identifier(name) => Ok(name),
            _ => Err(ParseError {
                message: format!("Expected identifier, found {:?}", token.type_),
                range: Some(token.range.clone()),
            }),
        }
    }

    pub fn expect_type(&mut self, expected: TokenType) -> Result<&Token> {
        match self.peek() {
            Some(token) if token.type_ == expected => Ok(token),
            Some(token) => Err(ParseError {
                message: format!("Expected {:?}, found {:?}", expected, token.type_),
                range: Some(token.range.clone()),
            }),
            None => Err(ParseError {
                message: format!("Expected {:?}, found end of input", expected),
                range: None,
            }),
        }
    }

    pub fn check_type(&mut self, expected: TokenType) -> bool {
        match self.peek_type() {
            Some(token_type) => token_type == expected,
            None => false,
        }
    }

    pub fn check_identifier(&mut self) -> bool {
        matches!(self.peek_type(), Some(TokenType::Identifier(_)))
    }

    pub fn match_type(&mut self, expected: TokenType) -> bool {
        if self.check_type(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn match_any(&mut self, types: &[TokenType]) -> Option<TokenType> {
        for token_type in types {
            if self.check_type(token_type.clone()) {
                self.advance();
                return Some(token_type.clone());
            }
        }
        None
    }

    pub fn sync(&mut self) {
        loop {
            match self.peek_type() {
                Some(TokenType::Semicolon) => {
                    self.advance();
                    break;
                }
                Some(TokenType::End) => {
                    self.advance();
                    break;
                }
                Some(TokenType::Fn)
                | Some(TokenType::Struct)
                | Some(TokenType::Enum)
                | Some(TokenType::Trait)
                | Some(TokenType::Impl)
                | Some(TokenType::Const)
                | Some(TokenType::Import)
                | Some(TokenType::Let) => break,
                None => break,
                _ => {
                    self.advance();
                }
            }
        }
    }

    pub fn create_meta(&self, range: std::ops::Range<usize>) -> Meta {
        Meta {
            filename: self.filename.clone(),
            range,
            attributes: Vec::new(),
        }
    }

    pub fn parse_attributes(&mut self) -> Result<Vec<Attribute>> {
        let mut attributes = Vec::new();

        while self.match_type(TokenType::At) {
            let name = self.consume_identifier()?;

            let args = if self.match_type(TokenType::LParen) {
                let mut attribute_args = Vec::new();
                if !self.check_type(TokenType::RParen) {
                    loop {
                        if self.check_identifier()
                            || matches!(self.peek_type(), Some(TokenType::Str(_)))
                        {
                            let token = self.consume()?;
                            match token.type_ {
                                TokenType::Str(s) => {
                                    attribute_args.push(AttributeArg::Literal(s));
                                }
                                TokenType::Identifier(s) => {
                                    if self.match_type(TokenType::Assign) {
                                        let value_token = self.consume()?;
                                        match value_token.type_ {
                                            TokenType::Str(v) => {
                                                attribute_args.push(AttributeArg::KeyValue(s, v));
                                            }
                                            TokenType::Identifier(v) => {
                                                attribute_args.push(AttributeArg::KeyValue(s, v));
                                            }
                                            _ => {
                                                return Err(ParseError {
                                                    message: format!(
                                                        "Invalid attribute value: {:?}",
                                                        value_token.type_
                                                    ),
                                                    range: Some(value_token.range.clone()),
                                                });
                                            }
                                        }
                                    } else {
                                        attribute_args.push(AttributeArg::Variable(s));
                                    }
                                }
                                _ => {}
                            }
                        }

                        if !self.match_type(TokenType::Comma) {
                            break;
                        }
                    }
                }
                self.consume_type(TokenType::RParen)?;
                attribute_args
            } else {
                Vec::new()
            };

            attributes.push(Attribute { name, args });
        }

        Ok(attributes)
    }

    pub fn add_error(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn get_errors(&self) -> &[ParseError] {
        &self.errors
    }

    pub fn is_at_end(&mut self) -> bool {
        self.peek().is_none()
    }

    pub fn parse(&mut self) -> Result<Program> {
        let mut program = Vec::new();

        while !self.is_at_end() {
            let attributes = self.parse_attributes()?;
            let start_token = self.peek().ok_or_else(|| ParseError {
                message: "Expected a top-level declaration".to_string(),
                range: None,
            })?;
            let start = start_token.range.start;

            let kind = match &start_token.type_ {
                TokenType::Fn => {
                    self.advance();
                    self.parse_function()?
                }
                TokenType::Struct => {
                    self.advance();
                    self.parse_struct()?
                }
                TokenType::Enum => {
                    self.advance();
                    self.parse_enum()?
                }
                TokenType::Trait => {
                    self.advance();
                    self.parse_trait()?
                }
                TokenType::Impl => {
                    self.advance();
                    self.parse_impl()?
                }
                TokenType::Const => {
                    self.advance();
                    self.parse_const()?
                }
                TokenType::Extern => {
                    self.advance();
                    self.parse_extern()?
                }
                TokenType::Import => {
                    self.advance();
                    self.parse_import()?
                }
                token_type => {
                    return Err(ParseError {
                        message: format!("Expected top-level declaration, found {:?}", token_type),
                        range: Some(start_token.range.clone()),
                    });
                }
            };

            let end = match self.peek() {
                Some(token) => token.range.end,
                None => start,
            };

            let node = AstNode {
                meta: Meta {
                    filename: self.filename.clone(),
                    range: start..end,
                    attributes,
                },
                kind,
            };

            program.push(node);
        }

        Ok(program)
    }

    fn parse_function(&mut self) -> Result<AstNodeKind> {
        let name = self.consume_identifier()?;

        let generic_params = if self.match_type(TokenType::Less) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        self.consume_type(TokenType::LParen)?;
        let mut args = Vec::new();
        if !self.check_type(TokenType::RParen) {
            loop {
                args.push(self.parse_function_param()?);
                if !self.match_type(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume_type(TokenType::RParen)?;

        let return_type = if self.match_type(TokenType::Arrow) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        let body = self.parse_block(false)?;

        Ok(AstNodeKind::Function {
            name,
            args,
            return_type,
            generic_params,
            body,
        })
    }

    fn parse_struct(&mut self) -> Result<AstNodeKind> {
        let name = self.consume_identifier()?;

        let generic_params = if self.match_type(TokenType::Less) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        let mut fields = Vec::new();
        while !self.check_type(TokenType::End) && !self.is_at_end() {
            let attributes = self.parse_attributes()?;
            let field_name = self.consume_identifier()?;
            self.consume_type(TokenType::Colon)?;
            let type_annotation = self.parse_type_annotation()?;
            let field_token = self.peek();
            let start = field_name.len().saturating_sub(field_name.len());
            let end = field_token.map(|t| t.range.end).unwrap_or(start);

            fields.push(StructField {
                name: field_name,
                type_annotation,
                meta: Meta {
                    filename: self.filename.clone(),
                    range: start..end,
                    attributes,
                },
            });
        }

        self.consume_type(TokenType::End)?;

        Ok(AstNodeKind::Struct {
            name,
            generic_params,
            fields,
        })
    }

    fn parse_enum(&mut self) -> Result<AstNodeKind> {
        let name = self.consume_identifier()?;

        let generic_params = if self.match_type(TokenType::Less) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        let mut variants = Vec::new();
        while !self.check_type(TokenType::End) && !self.is_at_end() {
            let variant_name = self.consume_identifier()?;
            let start_token = self.peek();
            let start = start_token.map(|t| t.range.start).unwrap_or(0);

            if self.match_type(TokenType::LParen) {
                let mut fields = Vec::new();
                if !self.check_type(TokenType::RParen) {
                    loop {
                        fields.push(self.parse_type_annotation()?);
                        if !self.match_type(TokenType::Comma) {
                            break;
                        }
                    }
                }
                self.consume_type(TokenType::RParen)?;
                let end_token = self.peek();
                let end = end_token.map(|t| t.range.end).unwrap_or(start);

                variants.push(EnumVariant::Tuple {
                    name: variant_name,
                    fields,
                    meta: Meta {
                        filename: self.filename.clone(),
                        range: start..end,
                        attributes: Vec::new(),
                    },
                });
            } else if self.match_type(TokenType::LBrace) {
                let mut fields = Vec::new();
                if !self.check_type(TokenType::RBrace) {
                    loop {
                        let attributes = self.parse_attributes()?;
                        let field_name = self.consume_identifier()?;
                        self.consume_type(TokenType::Colon)?;
                        let type_annotation = self.parse_type_annotation()?;
                        let field_start = self.peek().map(|t| t.range.start).unwrap_or(start);
                        let field_end = self.peek().map(|t| t.range.end).unwrap_or(field_start);

                        fields.push(StructField {
                            name: field_name,
                            type_annotation,
                            meta: Meta {
                                filename: self.filename.clone(),
                                range: field_start..field_end,
                                attributes,
                            },
                        });

                        if !self.match_type(TokenType::Comma) {
                            break;
                        }
                    }
                }
                self.consume_type(TokenType::RBrace)?;
                let end_token = self.peek();
                let end = end_token.map(|t| t.range.end).unwrap_or(start);

                variants.push(EnumVariant::Struct {
                    name: variant_name,
                    fields,
                    meta: Meta {
                        filename: self.filename.clone(),
                        range: start..end,
                        attributes: Vec::new(),
                    },
                });
            } else {
                let end_token = self.peek();
                let end = end_token.map(|t| t.range.end).unwrap_or(start);

                variants.push(EnumVariant::Tuple {
                    name: variant_name,
                    fields: Vec::new(),
                    meta: Meta {
                        filename: self.filename.clone(),
                        range: start..end,
                        attributes: Vec::new(),
                    },
                });
            }
        }

        self.consume_type(TokenType::End)?;

        Ok(AstNodeKind::Enum {
            name,
            generic_params,
            variants,
        })
    }

    fn parse_trait(&mut self) -> Result<AstNodeKind> {
        let name = self.consume_identifier()?;

        let generic_params = if self.match_type(TokenType::Less) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        let mut methods = Vec::new();
        while !self.check_type(TokenType::End) && !self.is_at_end() {
            let start_token = self.peek().ok_or_else(|| ParseError {
                message: "Expected method signature".to_string(),
                range: None,
            })?;
            let start = start_token.range.start;

            self.consume_type(TokenType::Fn)?;
            let method_name = self.consume_identifier()?;

            let method_generic_params = if self.match_type(TokenType::Less) {
                self.parse_generic_params()?
            } else {
                Vec::new()
            };

            self.consume_type(TokenType::LParen)?;
            let mut args = Vec::new();
            if !self.check_type(TokenType::RParen) {
                loop {
                    if self.match_type(TokenType::SelfKeyword) {
                        args.push(MethodParam::SelfParam);
                    } else {
                        let name = self.consume_identifier()?;
                        self.consume_type(TokenType::Colon)?;
                        let type_annotation = self.parse_type_annotation()?;
                        args.push(MethodParam::TypedParam {
                            name,
                            type_annotation,
                        });
                    }
                    if !self.match_type(TokenType::Comma) {
                        break;
                    }
                }
            }
            self.consume_type(TokenType::RParen)?;

            let return_type = if self.match_type(TokenType::Arrow) {
                Some(self.parse_type_annotation()?)
            } else {
                None
            };

            let end_token = self.peek();
            let end = end_token.map(|t| t.range.end).unwrap_or(start);

            methods.push(MethodSignature {
                name: method_name,
                args,
                return_type,
                generic_params: method_generic_params,
                meta: Meta {
                    filename: self.filename.clone(),
                    range: start..end,
                    attributes: Vec::new(),
                },
            });
        }

        self.consume_type(TokenType::End)?;

        Ok(AstNodeKind::Trait {
            name,
            generic_params,
            methods,
        })
    }

    fn parse_impl(&mut self) -> Result<AstNodeKind> {
        let trait_name = if self.check_identifier() {
            Some(self.consume_identifier()?)
        } else {
            None
        };

        let for_type = if self.match_type(TokenType::For) {
            self.parse_type_annotation()?
        } else if trait_name.is_some() {
            return Err(ParseError {
                message: "Expected 'for' after trait name in impl".to_string(),
                range: None,
            });
        } else {
            return Err(ParseError {
                message: "Expected trait name or type in impl".to_string(),
                range: None,
            });
        };

        let mut methods = Vec::new();
        while !self.check_type(TokenType::End) && !self.is_at_end() {
            let start_token = self.peek().ok_or_else(|| ParseError {
                message: "Expected method in impl".to_string(),
                range: None,
            })?;
            let start = start_token.range.start;
            let attributes = self.parse_attributes()?;
            let kind = self.parse_function()?;
            let end_token = self.peek();
            let end = end_token.map(|t| t.range.end).unwrap_or(start);

            methods.push(AstNode {
                meta: Meta {
                    filename: self.filename.clone(),
                    range: start..end,
                    attributes,
                },
                kind,
            });
        }

        self.consume_type(TokenType::End)?;

        Ok(AstNodeKind::Impl {
            trait_name,
            for_type,
            methods,
        })
    }

    fn parse_const(&mut self) -> Result<AstNodeKind> {
        let name = self.consume_identifier()?;

        let type_annotation = if self.match_type(TokenType::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        self.consume_type(TokenType::Assign)?;
        let value = self.parse_expression()?;

        Ok(AstNodeKind::Const {
            name,
            type_annotation,
            value,
        })
    }

    fn parse_extern(&mut self) -> Result<AstNodeKind> {
        self.consume_type(TokenType::Fn)?;
        let name = self.consume_identifier()?;

        self.consume_type(TokenType::LParen)?;
        let mut args = Vec::new();
        if !self.check_type(TokenType::RParen) {
            loop {
                args.push(self.parse_function_param()?);
                if !self.match_type(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume_type(TokenType::RParen)?;

        let return_type = if self.match_type(TokenType::Arrow) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        Ok(AstNodeKind::Extern {
            name,
            args,
            return_type,
        })
    }

    fn parse_import(&mut self) -> Result<AstNodeKind> {
        let path = match self.consume()?.type_ {
            TokenType::Str(s) => s,
            token_type => {
                return Err(ParseError {
                    message: format!(
                        "Expected string literal for import path, found {:?}",
                        token_type
                    ),
                    range: None,
                });
            }
        };

        let items = if self.match_type(TokenType::Dot) {
            self.consume_type(TokenType::LBrace)?;
            let mut import_items = Vec::new();
            if !self.check_type(TokenType::RBrace) {
                loop {
                    let name = self.consume_identifier()?;
                    let alias = if self.match_type(TokenType::As) {
                        Some(self.consume_identifier()?)
                    } else {
                        None
                    };
                    import_items.push(ImportItem { name, alias });

                    if !self.match_type(TokenType::Comma) {
                        break;
                    }
                }
            }
            self.consume_type(TokenType::RBrace)?;
            import_items
        } else {
            Vec::new()
        };

        Ok(AstNodeKind::Import { path, items })
    }

    fn parse_function_param(&mut self) -> Result<FunctionParam> {
        let name = self.consume_identifier()?;
        self.consume_type(TokenType::Colon)?;
        let type_annotation = self.parse_type_annotation()?;
        Ok(FunctionParam {
            name,
            type_annotation,
        })
    }

    fn parse_generic_params(&mut self) -> Result<Vec<GenericParam>> {
        let mut params = Vec::new();
        loop {
            let name = self.consume_identifier()?;
            let mut constraints = Vec::new();
            if self.match_type(TokenType::Colon) {
                loop {
                    constraints.push(self.parse_type_annotation()?);
                    if !self.match_type(TokenType::Plus) {
                        break;
                    }
                }
            }
            params.push(GenericParam { name, constraints });

            if !self.match_type(TokenType::Comma) {
                break;
            }
        }
        self.consume_type(TokenType::Greater)?;
        Ok(params)
    }

    fn parse_type_arguments(&mut self) -> Result<Vec<TypeAnnotation>> {
        let mut args = Vec::new();
        if !self.check_type(TokenType::Greater) {
            loop {
                args.push(self.parse_type_annotation()?);
                if !self.match_type(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume_type(TokenType::Greater)?;
        Ok(args)
    }

    fn parse_type_annotation(&mut self) -> Result<TypeAnnotation> {
        let start_token = self.peek().ok_or_else(|| ParseError {
            message: "Expected type annotation".to_string(),
            range: None,
        })?;
        let start = start_token.range.start;

        let name = if self.match_type(TokenType::SelfType) {
            "Self".to_string()
        } else {
            self.consume_identifier()?
        };

        let kind = if name == "Self" {
            TypeAnnotationKind::Constructor {
                name,
                generic_args: vec![],
            }
        } else if self.match_type(TokenType::Less) {
            let generic_args = self.parse_type_arguments()?;
            TypeAnnotationKind::Constructor { name, generic_args }
        } else if self.match_type(TokenType::LParen) {
            let mut types = Vec::new();
            if !self.check_type(TokenType::RParen) {
                loop {
                    types.push(self.parse_type_annotation()?);
                    if !self.match_type(TokenType::Comma) {
                        break;
                    }
                }
            }
            self.consume_type(TokenType::RParen)?;
            TypeAnnotationKind::Tuple(types)
        } else if self.match_type(TokenType::LBrace) {
            let mut fields = Vec::new();
            if !self.check_type(TokenType::RBrace) {
                loop {
                    let field_name = self.consume_identifier()?;
                    self.consume_type(TokenType::Colon)?;
                    let field_type = self.parse_type_annotation()?;
                    fields.push(RowTypeField {
                        name: field_name,
                        type_annotation: field_type,
                    });
                    if !self.match_type(TokenType::Comma) {
                        break;
                    }
                }
            }
            self.consume_type(TokenType::RBrace)?;
            TypeAnnotationKind::Row(fields)
        } else if self.match_type(TokenType::LBrack) {
            let element_type = Box::new(self.parse_type_annotation()?);
            let size = if let Some(TokenType::Integer(i)) = self.peek_type() {
                self.advance();
                Some(i as usize)
            } else {
                None
            };
            self.consume_type(TokenType::RBrack)?;
            TypeAnnotationKind::Array { element_type, size }
        } else {
            TypeAnnotationKind::Constructor {
                name,
                generic_args: Vec::new(),
            }
        };

        let end_token = self.peek();
        let end = end_token.map(|t| t.range.end).unwrap_or(start);

        Ok(TypeAnnotation {
            meta: Meta {
                filename: self.filename.clone(),
                range: start..end,
                attributes: Vec::new(),
            },
            kind,
        })
    }

    fn parse_block(&mut self, explicit: bool) -> Result<Vec<Expr>> {
        if explicit {
            self.consume_type(TokenType::Do)?;
        }

        let mut expressions = Vec::new();
        while !self.check_type(TokenType::End) && !self.is_at_end() {
            expressions.push(self.parse_expression()?);

            if self.check_type(TokenType::Semicolon) {
                self.advance();
            }
        }

        self.consume_type(TokenType::End)?;
        Ok(expressions)
    }

    fn parse_expression(&mut self) -> Result<Expr> {
        let attributes = self.parse_attributes()?;
        let mut expr = self.parse_assignment()?;

        if !attributes.is_empty() {
            expr.meta.attributes = attributes;
        }

        Ok(expr)
    }

    fn parse_assignment(&mut self) -> Result<Expr> {
        let left = self.parse_pipe()?;

        if self.match_type(TokenType::Assign) {
            let right = self.parse_assignment()?;
            let start = left.meta.range.start;
            let end = right.meta.range.end;

            return Ok(Expr {
                meta: Meta {
                    filename: self.filename.clone(),
                    range: start..end,
                    attributes: Vec::new(),
                },
                kind: ExprKind::Assign {
                    target: Box::new(left),
                    value: Box::new(right),
                },
            });
        } else if let Some(op) = self.match_any(&[
            TokenType::PlusEq,
            TokenType::MinusEq,
            TokenType::MulEq,
            TokenType::DivEq,
        ]) {
            let op_str = match op {
                TokenType::PlusEq => "+=",
                TokenType::MinusEq => "-=",
                TokenType::MulEq => "*=",
                TokenType::DivEq => "/=",
                _ => unreachable!(),
            };
            let right = self.parse_assignment()?;
            let start = left.meta.range.start;
            let end = right.meta.range.end;

            return Ok(Expr {
                meta: Meta {
                    filename: self.filename.clone(),
                    range: start..end,
                    attributes: Vec::new(),
                },
                kind: ExprKind::BinaryOp {
                    left: Box::new(left),
                    operator: op_str.to_string(),
                    right: Box::new(right),
                },
            });
        }

        Ok(left)
    }

    fn parse_pipe(&mut self) -> Result<Expr> {
        let mut left = self.parse_logical_or()?;

        while self.match_type(TokenType::Pipeline) {
            let right = self.parse_logical_or()?;
            let start = left.meta.range.start;
            let end = right.meta.range.end;

            left = Expr {
                meta: Meta {
                    filename: self.filename.clone(),
                    range: start..end,
                    attributes: Vec::new(),
                },
                kind: ExprKind::BinaryOp {
                    left: Box::new(left),
                    operator: "|>".to_string(),
                    right: Box::new(right),
                },
            };
        }

        Ok(left)
    }

    fn parse_logical_or(&mut self) -> Result<Expr> {
        self.parse_logical_and()
    }

    fn parse_logical_and(&mut self) -> Result<Expr> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut left = self.parse_comparison()?;

        while let Some(op) = self.match_any(&[TokenType::Equal, TokenType::NotEq]) {
            let op_str = match op {
                TokenType::Equal => "==",
                TokenType::NotEq => "!=",
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?;
            let start = left.meta.range.start;
            let end = right.meta.range.end;

            left = Expr {
                meta: Meta {
                    filename: self.filename.clone(),
                    range: start..end,
                    attributes: Vec::new(),
                },
                kind: ExprKind::BinaryOp {
                    left: Box::new(left),
                    operator: op_str.to_string(),
                    right: Box::new(right),
                },
            };
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut left = self.parse_range()?;

        while let Some(op) = self.match_any(&[
            TokenType::Less,
            TokenType::LessEq,
            TokenType::Greater,
            TokenType::GreaterEq,
        ]) {
            let op_str = match op {
                TokenType::Less => "<",
                TokenType::LessEq => "<=",
                TokenType::Greater => ">",
                TokenType::GreaterEq => ">=",
                _ => unreachable!(),
            };
            let right = self.parse_range()?;
            let start = left.meta.range.start;
            let end = right.meta.range.end;

            left = Expr {
                meta: Meta {
                    filename: self.filename.clone(),
                    range: start..end,
                    attributes: Vec::new(),
                },
                kind: ExprKind::BinaryOp {
                    left: Box::new(left),
                    operator: op_str.to_string(),
                    right: Box::new(right),
                },
            };
        }

        Ok(left)
    }

    fn parse_range(&mut self) -> Result<Expr> {
        let left = self.parse_term()?;

        if self.match_type(TokenType::Spread) {
            let right = self.parse_term()?;
            let start = left.meta.range.start;
            let end = right.meta.range.end;

            return Ok(Expr {
                meta: Meta {
                    filename: self.filename.clone(),
                    range: start..end,
                    attributes: Vec::new(),
                },
                kind: ExprKind::Range {
                    start: Box::new(left),
                    end: Box::new(right),
                    inclusive: false,
                },
            });
        }

        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr> {
        let mut left = self.parse_factor()?;

        while let Some(op) = self.match_any(&[TokenType::Plus, TokenType::Minus]) {
            let op_str = match op {
                TokenType::Plus => "+",
                TokenType::Minus => "-",
                _ => unreachable!(),
            };
            let right = self.parse_factor()?;
            let start = left.meta.range.start;
            let end = right.meta.range.end;

            left = Expr {
                meta: Meta {
                    filename: self.filename.clone(),
                    range: start..end,
                    attributes: Vec::new(),
                },
                kind: ExprKind::BinaryOp {
                    left: Box::new(left),
                    operator: op_str.to_string(),
                    right: Box::new(right),
                },
            };
        }

        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expr> {
        let mut left = self.parse_unary()?;

        while let Some(op) = self.match_any(&[TokenType::Mul, TokenType::Div]) {
            let op_str = match op {
                TokenType::Mul => "*",
                TokenType::Div => "/",
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            let start = left.meta.range.start;
            let end = right.meta.range.end;

            left = Expr {
                meta: Meta {
                    filename: self.filename.clone(),
                    range: start..end,
                    attributes: Vec::new(),
                },
                kind: ExprKind::BinaryOp {
                    left: Box::new(left),
                    operator: op_str.to_string(),
                    right: Box::new(right),
                },
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        let start_token = self.peek().ok_or_else(|| ParseError {
            message: "Expected expression".to_string(),
            range: None,
        })?;
        let start = start_token.range.start;

        if let Some(op) = self.match_any(&[TokenType::Not, TokenType::Minus]) {
            let op_str = match op {
                TokenType::Not => "!",
                TokenType::Minus => "-",
                _ => unreachable!(),
            };
            let operand = self.parse_unary()?;
            let end = operand.meta.range.end;

            return Ok(Expr {
                meta: Meta {
                    filename: self.filename.clone(),
                    range: start..end,
                    attributes: Vec::new(),
                },
                kind: ExprKind::UnaryOp {
                    operator: op_str.to_string(),
                    operand: Box::new(operand),
                },
            });
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_type(TokenType::Dot) {
                let field = self.consume_identifier()?;
                let end = self
                    .peek()
                    .map(|t| t.range.end)
                    .unwrap_or(expr.meta.range.end);

                expr = Expr {
                    meta: Meta {
                        filename: self.filename.clone(),
                        range: expr.meta.range.start..end,
                        attributes: Vec::new(),
                    },
                    kind: ExprKind::DotAccess {
                        value: Box::new(expr),
                        field,
                    },
                };
            } else if self.match_type(TokenType::LParen) {
                let mut args = Vec::new();
                if !self.check_type(TokenType::RParen) {
                    loop {
                        args.push(self.parse_expression()?);
                        if !self.match_type(TokenType::Comma) {
                            break;
                        }
                    }
                }
                self.consume_type(TokenType::RParen)?;
                let end = self
                    .peek()
                    .map(|t| t.range.end)
                    .unwrap_or(expr.meta.range.end);

                expr = Expr {
                    meta: Meta {
                        filename: self.filename.clone(),
                        range: expr.meta.range.start..end,
                        attributes: Vec::new(),
                    },
                    kind: ExprKind::FunctionCall {
                        function: Box::new(expr),
                        args,
                    },
                };
            } else if self.match_type(TokenType::LBrack) {
                let index = self.parse_expression()?;
                self.consume_type(TokenType::RBrack)?;
                let end = self
                    .peek()
                    .map(|t| t.range.end)
                    .unwrap_or(expr.meta.range.end);

                expr = Expr {
                    meta: Meta {
                        filename: self.filename.clone(),
                        range: expr.meta.range.start..end,
                        attributes: Vec::new(),
                    },
                    kind: ExprKind::Index {
                        value: Box::new(expr),
                        index: Box::new(index),
                    },
                };
            } else if self.match_type(TokenType::As) {
                let type_annotation = self.parse_type_annotation()?;
                let end = type_annotation.meta.range.end;

                expr = Expr {
                    meta: Meta {
                        filename: self.filename.clone(),
                        range: expr.meta.range.start..end,
                        attributes: Vec::new(),
                    },
                    kind: ExprKind::Cast(Box::new(expr), type_annotation),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        let start_token = self.peek().ok_or_else(|| ParseError {
            message: "Expected expression".to_string(),
            range: None,
        })?;
        let start = start_token.range.start;
        let token_type = start_token.type_.clone();

        let kind = match token_type {
            TokenType::Integer(i) => {
                self.advance();
                ExprKind::Literal(Literal::Integer(i))
            }
            TokenType::Float(f) => {
                self.advance();
                ExprKind::Literal(Literal::Float(f))
            }
            TokenType::Str(s) => {
                self.advance();
                ExprKind::Literal(Literal::String(s))
            }
            TokenType::True => {
                self.advance();
                ExprKind::Literal(Literal::Boolean(true))
            }
            TokenType::False => {
                self.advance();
                ExprKind::Literal(Literal::Boolean(false))
            }
            TokenType::Identifier(name) => {
                self.advance();

                if self.check_type(TokenType::DoubleColon) {
                    self.consume_type(TokenType::DoubleColon)?;
                    let variant_name = self.consume_identifier()?;
                    let end = self.peek().map(|t| t.range.end).unwrap_or(start);

                    // Check for enum initialization
                    if self.check_type(TokenType::LBrace) {
                        self.advance();
                        let mut fields = Vec::new();
                        if !self.check_type(TokenType::RBrace) {
                            loop {
                                if self.match_type(TokenType::Spread) {
                                    let expr = self.parse_expression()?;
                                    fields.push(crate::ast::StructInitField::Spread(expr));
                                } else {
                                    let field_name = self.consume_identifier()?;
                                    self.consume_type(TokenType::Colon)?;
                                    let value = self.parse_expression()?;
                                    fields.push(crate::ast::StructInitField::Field {
                                        name: field_name,
                                        value,
                                    });
                                }
                                if !self.match_type(TokenType::Comma) {
                                    break;
                                }
                            }
                        }
                        self.consume_type(TokenType::RBrace)?;
                        let end = self.peek().map(|t| t.range.end).unwrap_or(start);

                        return Ok(Expr {
                            meta: Meta {
                                filename: self.filename.clone(),
                                range: start..end,
                                attributes: Vec::new(),
                            },
                            kind: ExprKind::EnumInit {
                                enum_name: name,
                                variant_name,
                                fields: crate::ast::EnumInitFields::Struct(fields),
                            },
                        });
                    } else if self.check_type(TokenType::LParen) {
                        self.advance();
                        let mut fields = Vec::new();
                        if !self.check_type(TokenType::RParen) {
                            loop {
                                fields.push(self.parse_expression()?);
                                if !self.match_type(TokenType::Comma) {
                                    break;
                                }
                            }
                        }
                        self.consume_type(TokenType::RParen)?;
                        let end = self.peek().map(|t| t.range.end).unwrap_or(start);

                        return Ok(Expr {
                            meta: Meta {
                                filename: self.filename.clone(),
                                range: start..end,
                                attributes: Vec::new(),
                            },
                            kind: ExprKind::EnumInit {
                                enum_name: name,
                                variant_name,
                                fields: crate::ast::EnumInitFields::Tuple(fields),
                            },
                        });
                    } else {
                        return Ok(Expr {
                            meta: Meta {
                                filename: self.filename.clone(),
                                range: start..end,
                                attributes: Vec::new(),
                            },
                            kind: ExprKind::Variable(format!("{}::{}", name, variant_name)),
                        });
                    }
                } else if self.check_type(TokenType::LBrace) {
                    // Struct initialization
                    self.advance();
                    let mut fields = Vec::new();
                    if !self.check_type(TokenType::RBrace) {
                        loop {
                            if self.match_type(TokenType::Spread) {
                                let expr = self.parse_expression()?;
                                fields.push(crate::ast::StructInitField::Spread(expr));
                            } else {
                                let field_name = self.consume_identifier()?;
                                self.consume_type(TokenType::Colon)?;
                                let value = self.parse_expression()?;
                                fields.push(crate::ast::StructInitField::Field {
                                    name: field_name,
                                    value,
                                });
                            }
                            if !self.match_type(TokenType::Comma) {
                                break;
                            }
                        }
                    }
                    self.consume_type(TokenType::RBrace)?;
                    let end = self.peek().map(|t| t.range.end).unwrap_or(start);

                    return Ok(Expr {
                        meta: Meta {
                            filename: self.filename.clone(),
                            range: start..end,
                            attributes: Vec::new(),
                        },
                        kind: ExprKind::StructInit { name, fields },
                    });
                }

                ExprKind::Variable(name)
            }
            TokenType::LParen => {
                self.advance();
                let first_expr = self.parse_expression()?;
                if self.match_type(TokenType::Comma) {
                    // This is a tuple
                    let mut elements = vec![first_expr];
                    loop {
                        elements.push(self.parse_expression()?);
                        if !self.match_type(TokenType::Comma) {
                            break;
                        }
                    }
                    self.consume_type(TokenType::RParen)?;
                    ExprKind::Tuple(elements)
                } else {
                    self.consume_type(TokenType::RParen)?;
                    first_expr.kind
                }
            }
            TokenType::LBrack => {
                self.advance();
                let mut elements = Vec::new();
                if !self.check_type(TokenType::RBrack) {
                    loop {
                        if self.match_type(TokenType::Spread) {
                            let expr = self.parse_expression()?;
                            elements.push(ArrayElement::Spread(expr));
                        } else {
                            let expr = self.parse_expression()?;
                            elements.push(ArrayElement::Value(expr));
                        }
                        if !self.match_type(TokenType::Comma) {
                            break;
                        }
                    }
                }
                self.consume_type(TokenType::RBrack)?;
                ExprKind::Array(elements)
            }
            TokenType::LBrace => {
                self.advance();
                let mut fields = Vec::new();
                if !self.check_type(TokenType::RBrace) {
                    loop {
                        if self.match_type(TokenType::Spread) {
                            let expr = self.parse_expression()?;
                            fields.push(RowField::Spread(expr));
                        } else {
                            let name = self.consume_identifier()?;
                            self.consume_type(TokenType::Colon)?;
                            let mut value = self.parse_expression()?;

                            if matches!(value.kind, ExprKind::Literal(_)) {
                                if self.check_identifier() {
                                    if let Some(TokenType::Identifier(ref type_name)) =
                                        self.peek_type()
                                    {
                                        if type_name.as_str() == "i32"
                                            || type_name.as_str() == "i64"
                                            || type_name.as_str() == "f32"
                                            || type_name.as_str() == "f64"
                                        {
                                            let cast_type_name = self.consume_identifier()?;
                                            let cast_type = TypeAnnotation {
                                                meta: Meta {
                                                    filename: self.filename.clone(),
                                                    range: 0..0,
                                                    attributes: Vec::new(),
                                                },
                                                kind: TypeAnnotationKind::Constructor {
                                                    name: cast_type_name,
                                                    generic_args: Vec::new(),
                                                },
                                            };
                                            value = Expr {
                                                meta: Meta {
                                                    filename: self.filename.clone(),
                                                    range: value.meta.range.start
                                                        ..value.meta.range.end,
                                                    attributes: Vec::new(),
                                                },
                                                kind: ExprKind::Cast(Box::new(value), cast_type),
                                            };
                                        }
                                    }
                                }
                            }

                            fields.push(RowField::Field { name, value });
                        }
                        if !self.match_type(TokenType::Comma) {
                            break;
                        }
                    }
                }
                self.consume_type(TokenType::RBrace)?;
                ExprKind::Row(fields)
            }
            TokenType::Let => {
                self.advance();
                let name = self.consume_identifier()?;

                let type_annotation = if self.match_type(TokenType::Colon) {
                    Some(self.parse_type_annotation()?)
                } else {
                    None
                };

                self.consume_type(TokenType::Assign)?;
                let value = Box::new(self.parse_expression()?);

                ExprKind::Let {
                    name,
                    type_annotation,
                    value,
                }
            }
            TokenType::If => {
                self.advance();
                let condition = Box::new(self.parse_expression()?);

                let then_branch = self.parse_block(false)?;

                let else_branch = if self.match_type(TokenType::Else) {
                    Some(self.parse_block(false)?)
                } else {
                    None
                };

                ExprKind::If {
                    condition,
                    then_branch,
                    else_branch,
                }
            }
            TokenType::While => {
                self.advance();
                let condition = Box::new(self.parse_expression()?);
                let body = self.parse_block(false)?;

                ExprKind::While { condition, body }
            }
            TokenType::For => {
                self.advance();
                let iterator = self.consume_identifier()?;
                self.consume_type(TokenType::In)?;
                let iterable = Box::new(self.parse_expression()?);
                let body = self.parse_block(false)?;

                ExprKind::For {
                    iterator,
                    iterable,
                    body,
                }
            }
            TokenType::Match => {
                self.advance();
                let value = Box::new(self.parse_expression()?);

                let mut arms = Vec::new();
                while !self.check_type(TokenType::End) && !self.is_at_end() {
                    arms.push(self.parse_match_arm()?);
                }

                self.consume_type(TokenType::End)?;

                ExprKind::Match { value, arms }
            }
            TokenType::Do => {
                let body = self.parse_block(true)?;
                ExprKind::Do(body)
            }
            TokenType::Return => {
                self.advance();
                let value =
                    if !self.check_type(TokenType::End) && !self.check_type(TokenType::Semicolon) {
                        Some(Box::new(self.parse_expression()?))
                    } else {
                        None
                    };
                ExprKind::Return(value)
            }
            TokenType::Break => {
                self.advance();
                ExprKind::Break
            }
            TokenType::Continue => {
                self.advance();
                ExprKind::Continue
            }
            TokenType::Fn => {
                self.advance();

                let _generic_params = if self.match_type(TokenType::Less) {
                    self.parse_generic_params()?
                } else {
                    Vec::new()
                };

                self.consume_type(TokenType::LParen)?;
                let mut params = Vec::new();
                if !self.check_type(TokenType::RParen) {
                    loop {
                        let param_name = self.consume_identifier()?;

                        let param_type = if self.match_type(TokenType::Colon) {
                            self.parse_type_annotation()?
                        } else {
                            TypeAnnotation {
                                meta: Meta {
                                    filename: self.filename.clone(),
                                    range: 0..0,
                                    attributes: Vec::new(),
                                },
                                kind: TypeAnnotationKind::Constructor {
                                    name: "unknown".to_string(),
                                    generic_args: Vec::new(),
                                },
                            }
                        };

                        params.push(FunctionParam {
                            name: param_name,
                            type_annotation: param_type,
                        });

                        if !self.match_type(TokenType::Comma) {
                            break;
                        }
                    }
                }
                self.consume_type(TokenType::RParen)?;

                let return_type = if self.match_type(TokenType::Arrow) {
                    Some(self.parse_type_annotation()?)
                } else {
                    None
                };

                let body = Box::new(Expr {
                    meta: Meta {
                        filename: self.filename.clone(),
                        range: 0..0,
                        attributes: Vec::new(),
                    },
                    kind: ExprKind::Block {
                        expressions: self.parse_block(false)?,
                    },
                });

                ExprKind::Lambda {
                    params,
                    return_type,
                    body,
                }
            }
            token_type => {
                return Err(ParseError {
                    message: format!("Unexpected token in expression: {:?}", token_type),
                    range: Some(start_token.range.clone()),
                });
            }
        };

        let end_token = self.peek();
        let end = end_token.map(|t| t.range.end).unwrap_or(start);

        Ok(Expr {
            meta: Meta {
                filename: self.filename.clone(),
                range: start..end,
                attributes: Vec::new(),
            },
            kind,
        })
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm> {
        let pattern = self.parse_pattern()?;

        self.consume_type(TokenType::Assign)?;
        self.consume_type(TokenType::Greater)?;

        let guard = if self.check_type(TokenType::If) {
            self.advance();
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        let mut body = Vec::new();
        while !self.check_type(TokenType::End)
            && !self.check_type(TokenType::Comma)
            && !self.is_at_end()
        {
            body.push(self.parse_expression()?);
            if self.check_type(TokenType::Semicolon) {
                self.advance();
            }
        }

        self.match_type(TokenType::Comma);

        Ok(MatchArm {
            pattern,
            guard,
            body,
        })
    }

    fn parse_pattern(&mut self) -> Result<Pattern> {
        let start_token = self.peek().ok_or_else(|| ParseError {
            message: "Expected pattern".to_string(),
            range: None,
        })?;

        let token_type = start_token.type_.clone();

        match token_type {
            TokenType::Integer(i) => {
                self.advance();
                Ok(Pattern::Literal(Literal::Integer(i)))
            }
            TokenType::Float(f) => {
                self.advance();
                Ok(Pattern::Literal(Literal::Float(f)))
            }
            TokenType::Str(s) => {
                self.advance();
                Ok(Pattern::Literal(Literal::String(s)))
            }
            TokenType::True => {
                self.advance();
                Ok(Pattern::Literal(Literal::Boolean(true)))
            }
            TokenType::False => {
                self.advance();
                Ok(Pattern::Literal(Literal::Boolean(false)))
            }
            TokenType::Identifier(name) if name == "_" => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            TokenType::Identifier(name) => {
                self.advance();

                if self.match_type(TokenType::DoubleColon) {
                    let variant_name = self.consume_identifier()?;

                    if self.match_type(TokenType::LParen) {
                        let mut fields = Vec::new();
                        if !self.check_type(TokenType::RParen) {
                            loop {
                                fields.push(self.parse_pattern()?);
                                if !self.match_type(TokenType::Comma) {
                                    break;
                                }
                            }
                        }
                        self.consume_type(TokenType::RParen)?;
                        Ok(Pattern::EnumVariant {
                            name: format!("{}::{}", name, variant_name),
                            fields,
                        })
                    } else if self.match_type(TokenType::LBrace) {
                        let mut fields = Vec::new();
                        if !self.check_type(TokenType::RBrace) {
                            loop {
                                let field_name = self.consume_identifier()?;
                                if self.match_type(TokenType::Colon) {
                                    let pattern = self.parse_pattern()?;
                                    fields.push(StructPatternField::Typed {
                                        name: field_name,
                                        pattern,
                                    });
                                } else {
                                    fields.push(StructPatternField::Shorthand(field_name));
                                }
                                if !self.match_type(TokenType::Comma) {
                                    break;
                                }
                            }
                        }
                        self.consume_type(TokenType::RBrace)?;
                        Ok(Pattern::EnumVariant {
                            name: format!("{}::{}", name, variant_name),
                            fields: vec![],
                        })
                    } else {
                        Ok(Pattern::EnumVariant {
                            name: format!("{}::{}", name, variant_name),
                            fields: vec![],
                        })
                    }
                } else if self.match_type(TokenType::LBrace) {
                    let mut fields = Vec::new();
                    if !self.check_type(TokenType::RBrace) {
                        loop {
                            let field_name = self.consume_identifier()?;
                            if self.match_type(TokenType::Colon) {
                                let pattern = self.parse_pattern()?;
                                fields.push(StructPatternField::Typed {
                                    name: field_name,
                                    pattern,
                                });
                            } else {
                                fields.push(StructPatternField::Shorthand(field_name));
                            }
                            if !self.match_type(TokenType::Comma) {
                                break;
                            }
                        }
                    }
                    self.consume_type(TokenType::RBrace)?;
                    Ok(Pattern::Struct { name, fields })
                } else {
                    Ok(Pattern::Variable(name))
                }
            }
            TokenType::LParen => {
                self.advance();
                let mut patterns = Vec::new();
                if !self.check_type(TokenType::RParen) {
                    loop {
                        patterns.push(self.parse_pattern()?);
                        if !self.match_type(TokenType::Comma) {
                            break;
                        }
                    }
                }
                self.consume_type(TokenType::RParen)?;
                Ok(Pattern::Tuple(patterns))
            }
            token_type => Err(ParseError {
                message: format!("Expected pattern, found {:?}", token_type),
                range: Some(start_token.range.clone()),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_program(text: &str) -> Result<Program> {
        let tokens: Vec<Token> = Lexer::from_text(text).filter_map(|t| t.ok()).collect();
        let mut parser = Parser::new(tokens, "test".to_string());
        parser.parse()
    }

    fn parse_expression(text: &str) -> Result<Expr> {
        let tokens: Vec<Token> = Lexer::from_text(text).filter_map(|t| t.ok()).collect();
        let mut parser = Parser::new(tokens, "test".to_string());
        parser.parse_expression()
    }

    #[test]
    fn test_function_parsing() {
        let program = parse_program("fn add(x: i32, y: i32) -> i32 end");
        assert!(program.is_ok());
        let program = program.unwrap();
        assert_eq!(program.len(), 1);
        match &program[0].kind {
            AstNodeKind::Function {
                name,
                args,
                return_type,
                ..
            } => {
                assert_eq!(name, "add");
                assert_eq!(args.len(), 2);
                assert_eq!(args[0].name, "x");
                assert_eq!(args[1].name, "y");
                assert!(return_type.is_some());
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_function_with_body() {
        let program = parse_program("fn add(x: i32, y: i32) -> i32 return x + y end");
        assert!(program.is_ok());
        let program = program.unwrap();
        match &program[0].kind {
            AstNodeKind::Function { body, .. } => {
                assert_eq!(body.len(), 1);
                match &body[0].kind {
                    ExprKind::Return(Some(expr)) => match &expr.kind {
                        ExprKind::BinaryOp { operator, .. } => {
                            assert_eq!(operator, "+");
                        }
                        _ => panic!("Expected binary op"),
                    },
                    _ => panic!("Expected return"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_struct_parsing() {
        let program = parse_program("struct Point a: i32 b: f32 end");
        assert!(program.is_ok());
        let program = program.unwrap();
        assert_eq!(program.len(), 1);
        match &program[0].kind {
            AstNodeKind::Struct { name, fields, .. } => {
                assert_eq!(name, "Point");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].name, "a");
                assert_eq!(fields[1].name, "b");
            }
            _ => panic!("Expected struct"),
        }
    }

    #[test]
    fn test_enum_parsing() {
        let program = parse_program("enum Option Some(T) None end");
        assert!(program.is_ok());
        let program = program.unwrap();
        assert_eq!(program.len(), 1);
        match &program[0].kind {
            AstNodeKind::Enum { name, variants, .. } => {
                assert_eq!(name, "Option");
                assert_eq!(variants.len(), 2);
            }
            _ => panic!("Expected enum"),
        }
    }

    #[test]
    fn test_const_parsing() {
        let program = parse_program("const MAX: i32 = 100");
        assert!(program.is_ok());
        let program = program.unwrap();
        assert_eq!(program.len(), 1);
        match &program[0].kind {
            AstNodeKind::Const {
                name,
                type_annotation,
                value,
            } => {
                assert_eq!(name, "MAX");
                assert!(type_annotation.is_some());
                match &value.kind {
                    ExprKind::Literal(Literal::Integer(100)) => {}
                    _ => panic!("Expected integer literal"),
                }
            }
            _ => panic!("Expected const"),
        }
    }

    #[test]
    fn test_extern_parsing() {
        let program = parse_program("extern fn printf(format: i32) -> i32");
        assert!(program.is_ok());
        let program = program.unwrap();
        assert_eq!(program.len(), 1);
        match &program[0].kind {
            AstNodeKind::Extern {
                name,
                args,
                return_type,
            } => {
                assert_eq!(name, "printf");
                assert_eq!(args.len(), 1);
                assert_eq!(args[0].name, "format");
                assert!(return_type.is_some());
            }
            _ => panic!("Expected extern"),
        }
    }

    #[test]
    fn test_import_all() {
        let program = parse_program("import \"std/abc\"");
        assert!(program.is_ok());
        let program = program.unwrap();
        assert_eq!(program.len(), 1);
        match &program[0].kind {
            AstNodeKind::Import { path, items } => {
                assert_eq!(path, "std/abc");
                assert_eq!(items.len(), 0);
            }
            _ => panic!("Expected import"),
        }
    }

    #[test]
    fn test_import_selective() {
        let program = parse_program("import \"std/abc\".{A, B}");
        assert!(program.is_ok());
        let program = program.unwrap();
        assert_eq!(program.len(), 1);
        match &program[0].kind {
            AstNodeKind::Import { path, items } => {
                assert_eq!(path, "std/abc");
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].name, "A");
                assert_eq!(items[0].alias, None);
                assert_eq!(items[1].name, "B");
                assert_eq!(items[1].alias, None);
            }
            _ => panic!("Expected import"),
        }
    }

    #[test]
    fn test_import_with_alias() {
        let program = parse_program("import \"std/abc\".{A as B}");
        assert!(program.is_ok());
        let program = program.unwrap();
        assert_eq!(program.len(), 1);
        match &program[0].kind {
            AstNodeKind::Import { path, items } => {
                assert_eq!(path, "std/abc");
                assert_eq!(items.len(), 1);
                assert_eq!(items[0].name, "A");
                assert_eq!(items[0].alias, Some("B".to_string()));
            }
            _ => panic!("Expected import"),
        }
    }

    #[test]
    fn test_import_mixed() {
        let program = parse_program("import \"std/abc\".{A, B as C, D}");
        assert!(program.is_ok());
        let program = program.unwrap();
        assert_eq!(program.len(), 1);
        match &program[0].kind {
            AstNodeKind::Import { path, items } => {
                assert_eq!(path, "std/abc");
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].name, "A");
                assert_eq!(items[0].alias, None);
                assert_eq!(items[1].name, "B");
                assert_eq!(items[1].alias, Some("C".to_string()));
                assert_eq!(items[2].name, "D");
                assert_eq!(items[2].alias, None);
            }
            _ => panic!("Expected import"),
        }
    }

    #[test]
    fn test_literal_expressions() {
        let expr = parse_expression("42");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::Literal(Literal::Integer(42)) => {}
            _ => panic!("Expected integer literal"),
        }

        let expr = parse_expression("3.14");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::Literal(Literal::Float(f)) => {
                assert!((f - 3.14).abs() < 0.001);
            }
            _ => panic!("Expected float literal"),
        }

        let expr = parse_expression("\"hello\"");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::Literal(Literal::String(s)) => {
                assert_eq!(s, "hello");
            }
            _ => panic!("Expected string literal"),
        }

        let expr = parse_expression("true");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::Literal(Literal::Boolean(true)) => {}
            _ => panic!("Expected boolean literal"),
        }
    }

    #[test]
    fn test_binary_expressions() {
        let expr = parse_expression("1 + 2");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::BinaryOp {
                operator,
                left,
                right,
            } => {
                assert_eq!(operator, "+");
                match (&left.kind, &right.kind) {
                    (
                        ExprKind::Literal(Literal::Integer(1)),
                        ExprKind::Literal(Literal::Integer(2)),
                    ) => {}
                    _ => panic!("Expected integer literals"),
                }
            }
            _ => panic!("Expected binary operation"),
        }

        let expr = parse_expression("x == y");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::BinaryOp { operator, .. } => {
                assert_eq!(operator, "==");
            }
            _ => panic!("Expected binary operation"),
        }
    }

    #[test]
    fn test_unary_expressions() {
        let expr = parse_expression("-42");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::UnaryOp { operator, operand } => {
                assert_eq!(operator, "-");
                match operand.kind {
                    ExprKind::Literal(Literal::Integer(42)) => {}
                    _ => panic!("Expected integer literal"),
                }
            }
            _ => panic!("Expected unary operation"),
        }

        let expr = parse_expression("!true");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::UnaryOp { operator, .. } => {
                assert_eq!(operator, "!");
            }
            _ => panic!("Expected unary operation"),
        }
    }

    #[test]
    fn test_variable_expression() {
        let expr = parse_expression("x");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::Variable(name) => {
                assert_eq!(name, "x");
            }
            _ => panic!("Expected variable"),
        }
    }

    #[test]
    fn test_function_call() {
        let expr = parse_expression("add(1, 2)");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::FunctionCall { function, args } => {
                match function.kind {
                    ExprKind::Variable(name) => assert_eq!(name, "add"),
                    _ => panic!("Expected variable"),
                }
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected function call"),
        }
    }

    #[test]
    fn test_dot_access() {
        let expr = parse_expression("obj.field");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::DotAccess { value, field } => {
                match value.kind {
                    ExprKind::Variable(name) => assert_eq!(name, "obj"),
                    _ => panic!("Expected variable"),
                }
                assert_eq!(field, "field");
            }
            _ => panic!("Expected dot access"),
        }
    }

    #[test]
    fn test_lambda_expression() {
        let expr = parse_expression("fn(x: i32) -> i32 x + 1 end");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::Lambda {
                params,
                return_type,
                body,
            } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].name, "x");
                assert!(return_type.is_some());
                match body.kind {
                    ExprKind::Block { expressions } => {
                        assert_eq!(expressions.len(), 1);
                        match &expressions[0].kind {
                            ExprKind::BinaryOp {
                                left,
                                operator,
                                right,
                            } => {
                                assert_eq!(operator, "+");
                                match &left.kind {
                                    ExprKind::Variable(var) => assert_eq!(var, "x"),
                                    _ => panic!("Expected variable x"),
                                }
                                match right.kind {
                                    ExprKind::Literal(Literal::Integer(1)) => {}
                                    _ => panic!("Expected literal 1"),
                                }
                            }
                            _ => panic!("Expected binary operation"),
                        }
                    }
                    _ => panic!("Expected block"),
                }
            }
            _ => panic!("Expected lambda"),
        }
    }

    #[test]
    fn test_row_expression() {
        let expr = parse_expression("{x: 1, y: 2}");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::Row(fields) => {
                assert_eq!(fields.len(), 2);
            }
            _ => panic!("Expected row"),
        }
    }

    #[test]
    fn test_let_expression() {
        let expr = parse_expression("let x = 42");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::Let {
                name,
                type_annotation,
                value,
            } => {
                assert_eq!(name, "x");
                assert!(type_annotation.is_none());
                match value.kind {
                    ExprKind::Literal(Literal::Integer(42)) => {}
                    _ => panic!("Expected integer literal"),
                }
            }
            _ => panic!("Expected let expression"),
        }
    }

    #[test]
    fn test_let_with_type() {
        let expr = parse_expression("let x: i32 = 42");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::Let {
                type_annotation, ..
            } => {
                assert!(type_annotation.is_some());
            }
            _ => panic!("Expected let expression"),
        }
    }

    #[test]
    fn test_assignment_expression() {
        let expr = parse_expression("x = 42");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::Assign { target, value } => {
                match target.kind {
                    ExprKind::Variable(name) => assert_eq!(name, "x"),
                    _ => panic!("Expected variable"),
                }
                match value.kind {
                    ExprKind::Literal(Literal::Integer(42)) => {}
                    _ => panic!("Expected integer literal"),
                }
            }
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_augmented_assignment() {
        let expr = parse_expression("x += 1");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::BinaryOp { operator, .. } => {
                assert_eq!(operator, "+=");
            }
            _ => panic!("Expected binary operation"),
        }

        let expr = parse_expression("x -= 1");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::BinaryOp { operator, .. } => {
                assert_eq!(operator, "-=");
            }
            _ => panic!("Expected binary operation"),
        }
    }

    #[test]
    fn test_while_expression() {
        let expr = parse_expression("while x < 10 x = x + 1 end");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::While { condition: _, body } => {
                assert!(!body.is_empty());
            }
            _ => panic!("Expected while expression"),
        }
    }

    #[test]
    fn test_for_expression() {
        let expr = parse_expression("for i in 0..10 println(i) end");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::For {
                iterator,
                iterable: _,
                body,
            } => {
                assert_eq!(iterator, "i");
                assert!(!body.is_empty());
            }
            _ => panic!("Expected for expression"),
        }
    }

    #[test]
    fn test_return_expression() {
        let expr = parse_expression("return 42");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::Return(Some(value)) => match value.kind {
                ExprKind::Literal(Literal::Integer(42)) => {}
                _ => panic!("Expected integer literal"),
            },
            _ => panic!("Expected return expression"),
        }
    }

    #[test]
    fn test_break_continue() {
        let expr = parse_expression("break");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::Break => {}
            _ => panic!("Expected break"),
        }

        let expr = parse_expression("continue");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::Continue => {}
            _ => panic!("Expected continue"),
        }
    }

    #[test]
    fn test_range_expression() {
        let expr = parse_expression("1..10");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::Range { inclusive, .. } => {
                assert!(!inclusive);
            }
            _ => panic!("Expected range expression"),
        }
    }

    #[test]
    fn test_cast_expression() {
        let expr = parse_expression("x as i32");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::Cast(expr, type_annotation) => {
                match expr.kind {
                    ExprKind::Variable(name) => assert_eq!(name, "x"),
                    _ => panic!("Expected variable"),
                }
                match type_annotation.kind {
                    TypeAnnotationKind::Constructor { name, .. } => assert_eq!(name, "i32"),
                    _ => panic!("Expected type name"),
                }
            }
            _ => panic!("Expected cast expression"),
        }
    }

    #[test]
    fn test_generic_params() {
        let program = parse_program("fn identity<T>(x: T) -> T end");
        assert!(program.is_ok());
        let program = program.unwrap();
        match &program[0].kind {
            AstNodeKind::Function { generic_params, .. } => {
                assert_eq!(generic_params.len(), 1);
                assert_eq!(generic_params[0].name, "T");
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_generic_with_constraints() {
        let program = parse_program("fn compare<T: Comparable>(a: T, b: T) -> i32 end");
        assert!(program.is_ok());
        let program = program.unwrap();
        match &program[0].kind {
            AstNodeKind::Function { generic_params, .. } => {
                assert_eq!(generic_params.len(), 1);
                assert_eq!(generic_params[0].name, "T");
                assert_eq!(generic_params[0].constraints.len(), 1);
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_type_annotation() {
        let program = parse_program("fn process(data: Vec<i32>) -> Option<i32> end");
        assert!(program.is_ok());
        let program = program.unwrap();
        match &program[0].kind {
            AstNodeKind::Function {
                args, return_type, ..
            } => {
                assert_eq!(args.len(), 1);
                match &args[0].type_annotation.kind {
                    TypeAnnotationKind::Constructor { name, generic_args } => {
                        assert_eq!(name, "Vec");
                        assert_eq!(generic_args.len(), 1);
                    }
                    _ => panic!("Expected type constructor"),
                }
                assert!(return_type.is_some());
                match &return_type.as_ref().unwrap().kind {
                    TypeAnnotationKind::Constructor { name, .. } => {
                        assert_eq!(name, "Option");
                    }
                    _ => panic!("Expected type constructor"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_attributes() {
        let program = parse_program("@test fn example() end");
        assert!(program.is_ok());
        let program = program.unwrap();
        assert_eq!(program.len(), 1);
        assert_eq!(program[0].meta.attributes.len(), 1);
        assert_eq!(program[0].meta.attributes[0].name, "test");
    }

    #[test]
    fn test_attribute_with_args() {
        let program = parse_program("@doc(\"test\") fn example() end");
        assert!(program.is_ok());
        let program = program.unwrap();
        assert_eq!(program[0].meta.attributes.len(), 1);
        assert_eq!(program[0].meta.attributes[0].name, "doc");
        assert_eq!(program[0].meta.attributes[0].args.len(), 1);
    }

    #[test]
    fn test_multiple_attributes() {
        let program = parse_program("@test @doc(\"example\") fn example() end");
        assert!(program.is_ok());
        let program = program.unwrap();
        assert_eq!(program[0].meta.attributes.len(), 2);
    }

    #[test]
    fn test_pipeline_operator() {
        let expr = parse_expression("x |> f() |> g()");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::BinaryOp {
                operator,
                left,
                right,
            } => {
                assert_eq!(operator, "|>");
                match right.kind {
                    ExprKind::FunctionCall { .. } => {}
                    _ => panic!("Expected function call"),
                }
            }
            _ => panic!("Expected binary operation"),
        }
    }

    #[test]
    fn test_index_expression() {
        let expr = parse_expression("arr[0]");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::Index { value, index } => {
                match value.kind {
                    ExprKind::Variable(name) => assert_eq!(name, "arr"),
                    _ => panic!("Expected variable"),
                }
                match index.kind {
                    ExprKind::Literal(Literal::Integer(0)) => {}
                    _ => panic!("Expected integer literal"),
                }
            }
            _ => panic!("Expected index expression"),
        }
    }

    #[test]
    fn test_nested_expressions() {
        let expr = parse_expression("(1 + 2) * 3");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::BinaryOp { operator, .. } => {
                assert_eq!(operator, "*");
            }
            _ => panic!("Expected binary operation"),
        }
    }

    #[test]
    fn test_do_block() {
        let expr = parse_expression("do x = 1 y = 2 end");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::Do(expressions) => {
                assert_eq!(expressions.len(), 2);
            }
            _ => panic!("Expected do block"),
        }
    }

    #[test]
    fn test_spread_in_array() {
        let expr = parse_expression("[1, ..rest, 3]");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::Array(elements) => {
                assert_eq!(elements.len(), 3);
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_spread_in_row() {
        let expr = parse_expression("{x: 1, ..rest}");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::Row(fields) => {
                assert_eq!(fields.len(), 2);
            }
            _ => panic!("Expected row"),
        }
    }

    #[test]
    fn test_parse_error() {
        let result = parse_program("fn");
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_top_level_declarations() {
        let program = parse_program("fn foo() end fn bar() end const X = 1");
        assert!(program.is_ok());
        let program = program.unwrap();
        assert_eq!(program.len(), 3);
    }

    #[test]
    fn test_struct_init() {
        let expr = parse_expression("Point { x: 1, y: 2 }");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::StructInit { name, fields } => {
                assert_eq!(name, "Point");
                assert_eq!(fields.len(), 2);
                match &fields[0] {
                    StructInitField::Field { name, value } => {
                        assert_eq!(name, "x");
                        match value.kind {
                            ExprKind::Literal(Literal::Integer(1)) => {}
                            _ => panic!("Expected integer literal 1"),
                        }
                    }
                    _ => panic!("Expected field"),
                }
                match &fields[1] {
                    StructInitField::Field { name, value } => {
                        assert_eq!(name, "y");
                        match value.kind {
                            ExprKind::Literal(Literal::Integer(2)) => {}
                            _ => panic!("Expected integer literal 2"),
                        }
                    }
                    _ => panic!("Expected field"),
                }
            }
            _ => panic!("Expected struct init"),
        }
    }

    #[test]
    fn test_struct_init_with_spread() {
        let expr = parse_expression("Point { x: 10, ..other }");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::StructInit { name, fields } => {
                assert_eq!(name, "Point");
                assert_eq!(fields.len(), 2);
                match &fields[0] {
                    StructInitField::Field { name, value } => {
                        assert_eq!(name, "x");
                        match value.kind {
                            ExprKind::Literal(Literal::Integer(10)) => {}
                            _ => panic!("Expected integer literal 10"),
                        }
                    }
                    _ => panic!("Expected field"),
                }
                match &fields[1] {
                    StructInitField::Spread(expr) => match &expr.kind {
                        ExprKind::Variable(var) => assert_eq!(var, "other"),
                        _ => panic!("Expected variable"),
                    },
                    _ => panic!("Expected spread"),
                }
            }
            _ => panic!("Expected struct init"),
        }
    }

    #[test]
    fn test_enum_init_tuple() {
        let expr = parse_expression("Option::Some(42)");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::EnumInit {
                enum_name,
                variant_name,
                fields,
            } => {
                assert_eq!(enum_name, "Option");
                assert_eq!(variant_name, "Some");
                match fields {
                    EnumInitFields::Tuple(exprs) => {
                        assert_eq!(exprs.len(), 1);
                        match exprs[0].kind {
                            ExprKind::Literal(Literal::Integer(42)) => {}
                            _ => panic!("Expected integer literal 42"),
                        }
                    }
                    _ => panic!("Expected tuple fields"),
                }
            }
            _ => panic!("Expected enum init"),
        }
    }

    #[test]
    fn test_enum_init_struct() {
        let expr = parse_expression("Color::Rgb { r: 255, g: 0, b: 128 }");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::EnumInit {
                enum_name,
                variant_name,
                fields,
            } => {
                assert_eq!(enum_name, "Color");
                assert_eq!(variant_name, "Rgb");
                match fields {
                    EnumInitFields::Struct(fields) => {
                        assert_eq!(fields.len(), 3);
                        match &fields[0] {
                            StructInitField::Field { name, value } => {
                                assert_eq!(name, "r");
                                match value.kind {
                                    ExprKind::Literal(Literal::Integer(255)) => {}
                                    _ => panic!("Expected integer literal 255"),
                                }
                            }
                            _ => panic!("Expected field"),
                        }
                    }
                    _ => panic!("Expected struct fields"),
                }
            }
            _ => panic!("Expected enum init"),
        }
    }

    #[test]
    fn test_enum_variant_reference() {
        let expr = parse_expression("Option::None");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::Variable(var) => {
                assert_eq!(var, "Option::None");
            }
            _ => panic!("Expected variable"),
        }
    }

    #[test]
    fn test_match_expression() {
        let program = parse_program("fn f() match x 1 => 42, _ => 0 end end");
        assert!(program.is_ok());
        let program = program.unwrap();
        assert_eq!(program.len(), 1);
        match &program[0].kind {
            AstNodeKind::Function { body, .. } => {
                match &body[0].kind {
                    ExprKind::Match { value, arms } => {
                        match &value.kind {
                            ExprKind::Variable(var) => assert_eq!(var, "x"),
                            _ => panic!("Expected variable x"),
                        }
                        assert_eq!(arms.len(), 2);
                        // First arm: 1 => 42
                        match &arms[0].pattern {
                            Pattern::Literal(Literal::Integer(1)) => {}
                            _ => panic!("Expected literal pattern 1"),
                        }
                        match &arms[0].body[0].kind {
                            ExprKind::Literal(Literal::Integer(42)) => {}
                            _ => panic!("Expected literal 42"),
                        }
                        // Second arm: _ => 0
                        match &arms[1].pattern {
                            Pattern::Wildcard => {}
                            _ => panic!("Expected wildcard pattern"),
                        }
                        match &arms[1].body[0].kind {
                            ExprKind::Literal(Literal::Integer(0)) => {}
                            _ => panic!("Expected literal 0"),
                        }
                    }
                    _ => panic!("Expected match expression"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_tuple_expression() {
        let expr = parse_expression("(1, \"hello\", true)");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::Tuple(elements) => {
                assert_eq!(elements.len(), 3);
                match &elements[0].kind {
                    ExprKind::Literal(Literal::Integer(1)) => {}
                    _ => panic!("Expected integer 1"),
                }
                match &elements[1].kind {
                    ExprKind::Literal(Literal::String(s)) => assert_eq!(s, "hello"),
                    _ => panic!("Expected string hello"),
                }
                match &elements[2].kind {
                    ExprKind::Literal(Literal::Boolean(true)) => {}
                    _ => panic!("Expected boolean true"),
                }
            }
            _ => panic!("Expected tuple"),
        }
    }

    #[test]
    fn test_empty_struct_init() {
        let expr = parse_expression("Empty {}");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        match expr.kind {
            ExprKind::StructInit { name, fields } => {
                assert_eq!(name, "Empty");
                assert_eq!(fields.len(), 0);
            }
            _ => panic!("Expected struct init"),
        }
    }
}
