#[cfg(test)]
mod tests {
    use crate::ast;
    use crate::meta;
    use crate::typechecker;
    use crate::typed_ast;

    fn empty_meta() -> meta::Meta {
        meta::Meta {
            filename: "test".to_string(),
            range: 0..0,
            attributes: vec![],
        }
    }

    #[test]
    fn test_simple_function() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test".to_string(),
                args: vec![],
                return_type: Some(ast::TypeAnnotation {
                    meta: empty_meta(),
                    kind: ast::TypeAnnotationKind::Constructor {
                        name: "i32".to_string(),
                        generic_args: vec![],
                    },
                }),
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Literal(ast::Literal::Integer(42)),
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());

        let typed_program = result.unwrap();
        let function = &typed_program[0];
        if let typed_ast::TypedAstNodeKind::Function { name, .. } = &function.kind {
            assert_eq!(name, "test");
        } else {
            panic!("Expected Function node");
        }
        assert!(matches!(
            function.typ,
            typed_ast::Type::Fun { ref args, .. } if args.is_empty()
        ));
    }

    #[test]
    fn test_binary_expression_complex() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "calc".to_string(),
                args: vec![
                    ast::FunctionParam {
                        name: "a".to_string(),
                        type_annotation: ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "i32".to_string(),
                                generic_args: vec![],
                            },
                        },
                    },
                    ast::FunctionParam {
                        name: "b".to_string(),
                        type_annotation: ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "i32".to_string(),
                                generic_args: vec![],
                            },
                        },
                    },
                    ast::FunctionParam {
                        name: "c".to_string(),
                        type_annotation: ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "i32".to_string(),
                                generic_args: vec![],
                            },
                        },
                    },
                ],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::BinaryOp {
                        left: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::BinaryOp {
                                left: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Variable("a".to_string()),
                                }),
                                operator: "+".to_string(),
                                right: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Variable("b".to_string()),
                                }),
                            },
                        }),
                        operator: "*".to_string(),
                        right: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Variable("c".to_string()),
                        }),
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());

        let typed_program = result.unwrap();
        let function = &typed_program[0];
        if let typed_ast::TypedAstNodeKind::Function {
            name, args, body, ..
        } = &function.kind
        {
            assert_eq!(name, "calc");
            assert_eq!(args.len(), 3);
            assert_eq!(args[0].name, "a");
            assert_eq!(args[1].name, "b");
            assert_eq!(args[2].name, "c");
            assert!(matches!(&args[0].typ, typed_ast::Type::Con { name, .. } if name == "i32"));
            assert!(matches!(&args[1].typ, typed_ast::Type::Con { name, .. } if name == "i32"));
            assert!(matches!(&args[2].typ, typed_ast::Type::Con { name, .. } if name == "i32"));
            assert!(matches!(&body[0].typ, typed_ast::Type::Con { name, .. } if name == "i32"));
        } else {
            panic!("Expected Function node");
        }
    }

    #[test]
    fn test_nested_binary_expressions() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "nested".to_string(),
                args: vec![ast::FunctionParam {
                    name: "x".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "i32".to_string(),
                            generic_args: vec![],
                        },
                    },
                }],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::BinaryOp {
                        left: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::BinaryOp {
                                left: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Variable("x".to_string()),
                                }),
                                operator: "*".to_string(),
                                right: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Variable("x".to_string()),
                                }),
                            },
                        }),
                        operator: "+".to_string(),
                        right: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::BinaryOp {
                                left: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Variable("x".to_string()),
                                }),
                                operator: "*".to_string(),
                                right: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Variable("x".to_string()),
                                }),
                            },
                        }),
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_multiple_args_inference() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "sum".to_string(),
                args: vec![
                    ast::FunctionParam {
                        name: "a".to_string(),
                        type_annotation: ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "i32".to_string(),
                                generic_args: vec![],
                            },
                        },
                    },
                    ast::FunctionParam {
                        name: "b".to_string(),
                        type_annotation: ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "i32".to_string(),
                                generic_args: vec![],
                            },
                        },
                    },
                ],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::BinaryOp {
                        left: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::BinaryOp {
                                left: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Variable("a".to_string()),
                                }),
                                operator: "+".to_string(),
                                right: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Variable("b".to_string()),
                                }),
                            },
                        }),
                        operator: "+".to_string(),
                        right: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Literal(ast::Literal::Integer(42)),
                        }),
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generic_binary_operation() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "add_same".to_string(),
                args: vec![
                    ast::FunctionParam {
                        name: "x".to_string(),
                        type_annotation: ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "T".to_string(),
                                generic_args: vec![],
                            },
                        },
                    },
                    ast::FunctionParam {
                        name: "y".to_string(),
                        type_annotation: ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "T".to_string(),
                                generic_args: vec![],
                            },
                        },
                    },
                ],
                return_type: Some(ast::TypeAnnotation {
                    meta: empty_meta(),
                    kind: ast::TypeAnnotationKind::Constructor {
                        name: "T".to_string(),
                        generic_args: vec![],
                    },
                }),
                generic_params: vec![ast::GenericParam {
                    name: "T".to_string(),
                    constraints: vec![],
                }],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Literal(ast::Literal::Integer(1)),
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_array_expression() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_array".to_string(),
                args: vec![],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Array(vec![
                        ast::ArrayElement::Value(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Literal(ast::Literal::Integer(1)),
                        }),
                        ast::ArrayElement::Value(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Literal(ast::Literal::Integer(2)),
                        }),
                        ast::ArrayElement::Value(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Literal(ast::Literal::Integer(3)),
                        }),
                    ]),
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());

        let typed_program = result.unwrap();
        let function = &typed_program[0];
        if let typed_ast::TypedAstNodeKind::Function { body, .. } = &function.kind {
            let array_expr = &body[0];
            assert!(matches!(array_expr.typ, typed_ast::Type::Array { .. }));
            if let typed_ast::Type::Array { elem, .. } = &array_expr.typ {
                assert!(
                    matches!(elem.as_ref(), typed_ast::Type::Con { name, .. } if name == "i32")
                );
            }
        } else {
            panic!("Expected Function node");
        }
    }

    #[test]
    fn test_tuple_expression() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_tuple".to_string(),
                args: vec![],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Tuple(vec![
                        ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Literal(ast::Literal::Integer(42)),
                        },
                        ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Literal(ast::Literal::Float(3.14)),
                        },
                        ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Literal(ast::Literal::String("hello".to_string())),
                        },
                    ]),
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());

        let typed_program = result.unwrap();
        let function = &typed_program[0];
        if let typed_ast::TypedAstNodeKind::Function { body, .. } = &function.kind {
            let tuple_expr = &body[0];
            if let typed_ast::Type::Tuple(types) = &tuple_expr.typ {
                assert_eq!(types.len(), 3);
                assert!(matches!(&types[0], typed_ast::Type::Con { name, .. } if name == "i32"));
                assert!(matches!(&types[1], typed_ast::Type::Con { name, .. } if name == "f32"));
                assert!(matches!(&types[2], typed_ast::Type::Con { name, .. } if name == "String"));
            } else {
                panic!("Expected Tuple type");
            }
        } else {
            panic!("Expected Function node");
        }
    }

    #[test]
    fn test_row_expression() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_row".to_string(),
                args: vec![],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Row(vec![
                        ast::RowField::Field {
                            name: "x".to_string(),
                            value: ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Literal(ast::Literal::Integer(1)),
                            },
                        },
                        ast::RowField::Field {
                            name: "y".to_string(),
                            value: ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Literal(ast::Literal::Integer(2)),
                            },
                        },
                    ]),
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_let_expression() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_let".to_string(),
                args: vec![],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Let {
                        name: "x".to_string(),
                        type_annotation: Some(ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "i32".to_string(),
                                generic_args: vec![],
                            },
                        }),
                        value: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Literal(ast::Literal::Integer(42)),
                        }),
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_assign_expression() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_assign".to_string(),
                args: vec![ast::FunctionParam {
                    name: "x".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "i32".to_string(),
                            generic_args: vec![],
                        },
                    },
                }],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Assign {
                        target: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Variable("x".to_string()),
                        }),
                        value: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Literal(ast::Literal::Integer(10)),
                        }),
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unary_expression() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_unary".to_string(),
                args: vec![ast::FunctionParam {
                    name: "x".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "i32".to_string(),
                            generic_args: vec![],
                        },
                    },
                }],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::UnaryOp {
                        operator: "-".to_string(),
                        operand: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Variable("x".to_string()),
                        }),
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cast_expression() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_cast".to_string(),
                args: vec![ast::FunctionParam {
                    name: "x".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "i32".to_string(),
                            generic_args: vec![],
                        },
                    },
                }],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Cast(
                        Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Variable("x".to_string()),
                        }),
                        ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "f32".to_string(),
                                generic_args: vec![],
                            },
                        },
                    ),
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_index_expression() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_index".to_string(),
                args: vec![ast::FunctionParam {
                    name: "arr".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Array {
                            element_type: Box::new(ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "i32".to_string(),
                                    generic_args: vec![],
                                },
                            }),
                            size: None,
                        },
                    },
                }],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Index {
                        value: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Variable("arr".to_string()),
                        }),
                        index: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Literal(ast::Literal::Integer(0)),
                        }),
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_call() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_call".to_string(),
                args: vec![ast::FunctionParam {
                    name: "f".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Function {
                            args: vec![
                                ast::TypeAnnotation {
                                    meta: empty_meta(),
                                    kind: ast::TypeAnnotationKind::Constructor {
                                        name: "i32".to_string(),
                                        generic_args: vec![],
                                    },
                                },
                                ast::TypeAnnotation {
                                    meta: empty_meta(),
                                    kind: ast::TypeAnnotationKind::Constructor {
                                        name: "i32".to_string(),
                                        generic_args: vec![],
                                    },
                                },
                            ],
                            return_type: Box::new(ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "i32".to_string(),
                                    generic_args: vec![],
                                },
                            }),
                        },
                    },
                }],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::FunctionCall {
                        function: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Variable("f".to_string()),
                        }),
                        args: vec![
                            ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Literal(ast::Literal::Integer(1)),
                            },
                            ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Literal(ast::Literal::Integer(2)),
                            },
                        ],
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_if_expression() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_if".to_string(),
                args: vec![ast::FunctionParam {
                    name: "x".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "i32".to_string(),
                            generic_args: vec![],
                        },
                    },
                }],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::If {
                        condition: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::BinaryOp {
                                left: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Variable("x".to_string()),
                                }),
                                operator: ">".to_string(),
                                right: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::Integer(0)),
                                }),
                            },
                        }),
                        then_branch: vec![ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Literal(ast::Literal::Integer(1)),
                        }],
                        else_branch: Some(vec![ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Literal(ast::Literal::Integer(0)),
                        }]),
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_while_expression() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_while".to_string(),
                args: vec![ast::FunctionParam {
                    name: "n".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "i32".to_string(),
                            generic_args: vec![],
                        },
                    },
                }],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::While {
                        condition: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::BinaryOp {
                                left: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Variable("n".to_string()),
                                }),
                                operator: ">".to_string(),
                                right: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::Integer(0)),
                                }),
                            },
                        }),
                        body: vec![ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Literal(ast::Literal::Integer(1)),
                        }],
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_for_expression() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_for".to_string(),
                args: vec![ast::FunctionParam {
                    name: "arr".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Array {
                            element_type: Box::new(ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "i32".to_string(),
                                    generic_args: vec![],
                                },
                            }),
                            size: None,
                        },
                    },
                }],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::For {
                        iterator: "x".to_string(),
                        iterable: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Variable("arr".to_string()),
                        }),
                        body: vec![ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Literal(ast::Literal::Integer(1)),
                        }],
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_range_expression() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_range".to_string(),
                args: vec![],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Range {
                        start: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Literal(ast::Literal::Integer(0)),
                        }),
                        end: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Literal(ast::Literal::Integer(10)),
                        }),
                        inclusive: false,
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_break_expression() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_break".to_string(),
                args: vec![],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::While {
                        condition: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Literal(ast::Literal::Boolean(true)),
                        }),
                        body: vec![ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Break,
                        }],
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_continue_expression() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_continue".to_string(),
                args: vec![],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::While {
                        condition: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Literal(ast::Literal::Boolean(true)),
                        }),
                        body: vec![ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Continue,
                        }],
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_return_expression() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_return".to_string(),
                args: vec![],
                return_type: Some(ast::TypeAnnotation {
                    meta: empty_meta(),
                    kind: ast::TypeAnnotationKind::Constructor {
                        name: "i32".to_string(),
                        generic_args: vec![],
                    },
                }),
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Return(Some(Box::new(ast::Expr {
                        meta: empty_meta(),
                        kind: ast::ExprKind::Literal(ast::Literal::Integer(42)),
                    }))),
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_block_expression() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_block".to_string(),
                args: vec![],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Block {
                        expressions: vec![
                            ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Let {
                                    name: "x".to_string(),
                                    type_annotation: None,
                                    value: Box::new(ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::Literal(ast::Literal::Integer(1)),
                                    }),
                                },
                            },
                            ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Let {
                                    name: "y".to_string(),
                                    type_annotation: None,
                                    value: Box::new(ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::Literal(ast::Literal::Integer(2)),
                                    }),
                                },
                            },
                            ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::BinaryOp {
                                    left: Box::new(ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::Variable("x".to_string()),
                                    }),
                                    operator: "+".to_string(),
                                    right: Box::new(ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::Variable("y".to_string()),
                                    }),
                                },
                            },
                        ],
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_do_expression() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_do".to_string(),
                args: vec![],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Do(vec![
                        ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Let {
                                name: "x".to_string(),
                                type_annotation: None,
                                value: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::Integer(1)),
                                }),
                            },
                        },
                        ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Let {
                                name: "y".to_string(),
                                type_annotation: None,
                                value: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::Integer(2)),
                                }),
                            },
                        },
                    ]),
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_lambda_expression() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_lambda".to_string(),
                args: vec![],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Lambda {
                        params: vec![ast::FunctionParam {
                            name: "x".to_string(),
                            type_annotation: ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "i32".to_string(),
                                    generic_args: vec![],
                                },
                            },
                        }],
                        return_type: Some(ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "i32".to_string(),
                                generic_args: vec![],
                            },
                        }),
                        body: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Variable("x".to_string()),
                        }),
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_match_expression() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_match".to_string(),
                args: vec![ast::FunctionParam {
                    name: "x".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "i32".to_string(),
                            generic_args: vec![],
                        },
                    },
                }],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Match {
                        value: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Variable("x".to_string()),
                        }),
                        arms: vec![
                            ast::MatchArm {
                                pattern: ast::Pattern::Literal(ast::Literal::Integer(1)),
                                guard: None,
                                body: vec![ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::Integer(10)),
                                }],
                            },
                            ast::MatchArm {
                                pattern: ast::Pattern::Literal(ast::Literal::Integer(2)),
                                guard: None,
                                body: vec![ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::Integer(20)),
                                }],
                            },
                            ast::MatchArm {
                                pattern: ast::Pattern::Wildcard,
                                guard: None,
                                body: vec![ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::Integer(0)),
                                }],
                            },
                        ],
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_init_expression() {
        let program = vec![
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Struct {
                    name: "Point".to_string(),
                    generic_params: vec![],
                    fields: vec![
                        ast::StructField {
                            name: "x".to_string(),
                            type_annotation: ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "i32".to_string(),
                                    generic_args: vec![],
                                },
                            },
                            meta: empty_meta(),
                        },
                        ast::StructField {
                            name: "y".to_string(),
                            type_annotation: ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "i32".to_string(),
                                    generic_args: vec![],
                                },
                            },
                            meta: empty_meta(),
                        },
                    ],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Function {
                    name: "test_struct_init".to_string(),
                    args: vec![],
                    return_type: None,
                    generic_params: vec![],
                    body: vec![ast::Expr {
                        meta: empty_meta(),
                        kind: ast::ExprKind::StructInit {
                            name: "Point".to_string(),
                            fields: vec![
                                ast::StructInitField::Field {
                                    name: "x".to_string(),
                                    value: ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::Literal(ast::Literal::Integer(1)),
                                    },
                                },
                                ast::StructInitField::Field {
                                    name: "y".to_string(),
                                    value: ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::Literal(ast::Literal::Integer(2)),
                                    },
                                },
                            ],
                        },
                    }],
                },
            },
        ];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());

        let typed_program = result.unwrap();
        assert_eq!(typed_program.len(), 2);

        if let typed_ast::TypedAstNodeKind::Struct { name, fields, .. } = &typed_program[0].kind {
            assert_eq!(name, "Point");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].name, "x");
            assert_eq!(fields[1].name, "y");
            assert!(matches!(&fields[0].typ, typed_ast::Type::Con { name, .. } if name == "i32"));
            assert!(matches!(&fields[1].typ, typed_ast::Type::Con { name, .. } if name == "i32"));
        } else {
            panic!("Expected Struct node");
        }

        if let typed_ast::TypedAstNodeKind::Function { body, .. } = &typed_program[1].kind {
            let struct_init = &body[0];
            if let typed_ast::TypedExprKind::StructInit { name, fields, .. } = &struct_init.kind {
                assert_eq!(name, "Point");
                assert_eq!(fields.len(), 2);
                assert!(
                    matches!(&struct_init.typ, typed_ast::Type::Con { name, .. } if name == "Point")
                );
            } else {
                panic!("Expected StructInit expression");
            }
        } else {
            panic!("Expected Function node");
        }
    }

    #[test]
    fn test_enum_init_expression() {
        let program = vec![
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Enum {
                    name: "Option".to_string(),
                    generic_params: vec![ast::GenericParam {
                        name: "T".to_string(),
                        constraints: vec![],
                    }],
                    variants: vec![
                        ast::EnumVariant::Tuple {
                            name: "Some".to_string(),
                            fields: vec![ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "T".to_string(),
                                    generic_args: vec![],
                                },
                            }],
                            meta: empty_meta(),
                        },
                        ast::EnumVariant::Tuple {
                            name: "None".to_string(),
                            fields: vec![],
                            meta: empty_meta(),
                        },
                    ],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Function {
                    name: "test_enum_init".to_string(),
                    args: vec![],
                    return_type: None,
                    generic_params: vec![],
                    body: vec![ast::Expr {
                        meta: empty_meta(),
                        kind: ast::ExprKind::EnumInit {
                            enum_name: "Option".to_string(),
                            variant_name: "Some".to_string(),
                            fields: ast::EnumInitFields::Tuple(vec![ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Literal(ast::Literal::Integer(42)),
                            }]),
                        },
                    }],
                },
            },
        ];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_early_return_expression() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_early_return".to_string(),
                args: vec![],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::EarlyReturn(Box::new(ast::Expr {
                        meta: empty_meta(),
                        kind: ast::ExprKind::Literal(ast::Literal::Integer(42)),
                    })),
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_declaration() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Struct {
                name: "Point".to_string(),
                generic_params: vec![],
                fields: vec![
                    ast::StructField {
                        name: "x".to_string(),
                        type_annotation: ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "i32".to_string(),
                                generic_args: vec![],
                            },
                        },
                        meta: empty_meta(),
                    },
                    ast::StructField {
                        name: "y".to_string(),
                        type_annotation: ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "i32".to_string(),
                                generic_args: vec![],
                            },
                        },
                        meta: empty_meta(),
                    },
                ],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_enum_declaration() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Enum {
                name: "Color".to_string(),
                generic_params: vec![],
                variants: vec![
                    ast::EnumVariant::Tuple {
                        name: "Red".to_string(),
                        fields: vec![],
                        meta: empty_meta(),
                    },
                    ast::EnumVariant::Tuple {
                        name: "Green".to_string(),
                        fields: vec![],
                        meta: empty_meta(),
                    },
                    ast::EnumVariant::Tuple {
                        name: "Blue".to_string(),
                        fields: vec![],
                        meta: empty_meta(),
                    },
                ],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extern_declaration() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Extern {
                name: "printf".to_string(),
                args: vec![ast::FunctionParam {
                    name: "fmt".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "String".to_string(),
                            generic_args: vec![],
                        },
                    },
                }],
                return_type: Some(ast::TypeAnnotation {
                    meta: empty_meta(),
                    kind: ast::TypeAnnotationKind::Constructor {
                        name: "void".to_string(),
                        generic_args: vec![],
                    },
                }),
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_const_declaration() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Const {
                name: "PI".to_string(),
                type_annotation: Some(ast::TypeAnnotation {
                    meta: empty_meta(),
                    kind: ast::TypeAnnotationKind::Constructor {
                        name: "f32".to_string(),
                        generic_args: vec![],
                    },
                }),
                value: ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Literal(ast::Literal::Float(3.14159)),
                },
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_if_inference() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_if_inference".to_string(),
                args: vec![ast::FunctionParam {
                    name: "x".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "i32".to_string(),
                            generic_args: vec![],
                        },
                    },
                }],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::If {
                        condition: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::BinaryOp {
                                left: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Variable("x".to_string()),
                                }),
                                operator: ">".to_string(),
                                right: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::Integer(0)),
                                }),
                            },
                        }),
                        then_branch: vec![ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Literal(ast::Literal::Integer(100)),
                        }],
                        else_branch: Some(vec![ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Literal(ast::Literal::Integer(200)),
                        }]),
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());

        let typed_program = result.unwrap();
        let function = &typed_program[0];
        if let typed_ast::TypedAstNodeKind::Function { body, args, .. } = &function.kind {
            assert_eq!(body.len(), 1);
            assert_eq!(args[0].name, "x");
            assert!(matches!(&args[0].typ, typed_ast::Type::Con { name, .. } if name == "i32"));

            let if_expr = &body[0];
            if let typed_ast::TypedExprKind::If {
                condition,
                then_branch,
                else_branch,
                ..
            } = &if_expr.kind
            {
                assert!(
                    matches!(&condition.typ, typed_ast::Type::Con { name, .. } if name == "bool")
                );
                assert_eq!(then_branch.len(), 1);
                assert!(
                    matches!(&then_branch[0].typ, typed_ast::Type::Con { name, .. } if name == "i32")
                );
                if let Some(else_branch) = else_branch {
                    assert_eq!(else_branch.len(), 1);
                    assert!(
                        matches!(&else_branch[0].typ, typed_ast::Type::Con { name, .. } if name == "i32")
                    );
                    assert!(
                        matches!(&if_expr.typ, typed_ast::Type::Con { name, .. } if name == "i32")
                    );
                }
            } else {
                panic!("Expected If expression");
            }
        } else {
            panic!("Expected Function node");
        }
    }

    #[test]
    fn test_if_inference_with_variables() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_if_var_inference".to_string(),
                args: vec![ast::FunctionParam {
                    name: "x".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "bool".to_string(),
                            generic_args: vec![],
                        },
                    },
                }],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::If {
                        condition: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Variable("x".to_string()),
                        }),
                        then_branch: vec![
                            ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Literal(ast::Literal::Integer(1)),
                            },
                            ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Literal(ast::Literal::Integer(2)),
                            },
                        ],
                        else_branch: Some(vec![
                            ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Literal(ast::Literal::Integer(3)),
                            },
                            ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Literal(ast::Literal::Integer(4)),
                            },
                        ]),
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_match_inference() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_match_inference".to_string(),
                args: vec![ast::FunctionParam {
                    name: "x".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "i32".to_string(),
                            generic_args: vec![],
                        },
                    },
                }],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Match {
                        value: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Variable("x".to_string()),
                        }),
                        arms: vec![
                            ast::MatchArm {
                                pattern: ast::Pattern::Literal(ast::Literal::Integer(1)),
                                guard: None,
                                body: vec![ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::Integer(10)),
                                }],
                            },
                            ast::MatchArm {
                                pattern: ast::Pattern::Literal(ast::Literal::Integer(2)),
                                guard: None,
                                body: vec![ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::Integer(20)),
                                }],
                            },
                            ast::MatchArm {
                                pattern: ast::Pattern::Literal(ast::Literal::Integer(3)),
                                guard: None,
                                body: vec![ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::Integer(30)),
                                }],
                            },
                        ],
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());

        let typed_program = result.unwrap();
        let function = &typed_program[0];
        if let typed_ast::TypedAstNodeKind::Function { body, args, .. } = &function.kind {
            assert_eq!(body.len(), 1);
            assert_eq!(args[0].name, "x");
            assert!(matches!(&args[0].typ, typed_ast::Type::Con { name, .. } if name == "i32"));

            let match_expr = &body[0];
            if let typed_ast::TypedExprKind::Match { arms, value, .. } = &match_expr.kind {
                assert!(matches!(&value.typ, typed_ast::Type::Con { name, .. } if name == "i32"));
                assert_eq!(arms.len(), 3);

                for arm in arms {
                    assert!(matches!(&arm.typ, typed_ast::Type::Con { name, .. } if name == "i32"));
                }
                assert!(
                    matches!(&match_expr.typ, typed_ast::Type::Con { name, .. } if name == "i32")
                );
            } else {
                panic!("Expected Match expression");
            }
        } else {
            panic!("Expected Function node");
        }
    }

    #[test]
    fn test_match_inference_with_variable_binding() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_match_var_binding".to_string(),
                args: vec![ast::FunctionParam {
                    name: "x".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "i32".to_string(),
                            generic_args: vec![],
                        },
                    },
                }],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Match {
                        value: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Variable("x".to_string()),
                        }),
                        arms: vec![
                            ast::MatchArm {
                                pattern: ast::Pattern::Literal(ast::Literal::Integer(1)),
                                guard: None,
                                body: vec![ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::Integer(10)),
                                }],
                            },
                            ast::MatchArm {
                                pattern: ast::Pattern::Variable("n".to_string()),
                                guard: None,
                                body: vec![ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::BinaryOp {
                                        left: Box::new(ast::Expr {
                                            meta: empty_meta(),
                                            kind: ast::ExprKind::Variable("n".to_string()),
                                        }),
                                        operator: "*".to_string(),
                                        right: Box::new(ast::Expr {
                                            meta: empty_meta(),
                                            kind: ast::ExprKind::Literal(ast::Literal::Integer(2)),
                                        }),
                                    },
                                }],
                            },
                        ],
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_match_inference_with_wildcard() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_match_wildcard".to_string(),
                args: vec![ast::FunctionParam {
                    name: "x".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "i32".to_string(),
                            generic_args: vec![],
                        },
                    },
                }],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Match {
                        value: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Variable("x".to_string()),
                        }),
                        arms: vec![
                            ast::MatchArm {
                                pattern: ast::Pattern::Literal(ast::Literal::Integer(1)),
                                guard: None,
                                body: vec![ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::String(
                                        "one".to_string(),
                                    )),
                                }],
                            },
                            ast::MatchArm {
                                pattern: ast::Pattern::Wildcard,
                                guard: None,
                                body: vec![ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::String(
                                        "other".to_string(),
                                    )),
                                }],
                            },
                        ],
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_block_inference() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_block_inference".to_string(),
                args: vec![],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Block {
                        expressions: vec![
                            ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Let {
                                    name: "a".to_string(),
                                    type_annotation: None,
                                    value: Box::new(ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::Literal(ast::Literal::Integer(10)),
                                    }),
                                },
                            },
                            ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Let {
                                    name: "b".to_string(),
                                    type_annotation: None,
                                    value: Box::new(ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::Literal(ast::Literal::Integer(20)),
                                    }),
                                },
                            },
                            ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::BinaryOp {
                                    left: Box::new(ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::Variable("a".to_string()),
                                    }),
                                    operator: "+".to_string(),
                                    right: Box::new(ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::Variable("b".to_string()),
                                    }),
                                },
                            },
                        ],
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_let_inference() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_let_inference".to_string(),
                args: vec![],
                return_type: None,
                generic_params: vec![],
                body: vec![
                    ast::Expr {
                        meta: empty_meta(),
                        kind: ast::ExprKind::Let {
                            name: "x".to_string(),
                            type_annotation: None,
                            value: Box::new(ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Literal(ast::Literal::Integer(42)),
                            }),
                        },
                    },
                    ast::Expr {
                        meta: empty_meta(),
                        kind: ast::ExprKind::Let {
                            name: "y".to_string(),
                            type_annotation: None,
                            value: Box::new(ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Variable("x".to_string()),
                            }),
                        },
                    },
                    ast::Expr {
                        meta: empty_meta(),
                        kind: ast::ExprKind::Let {
                            name: "z".to_string(),
                            type_annotation: None,
                            value: Box::new(ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::BinaryOp {
                                    left: Box::new(ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::Variable("x".to_string()),
                                    }),
                                    operator: "+".to_string(),
                                    right: Box::new(ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::Variable("y".to_string()),
                                    }),
                                },
                            }),
                        },
                    },
                ],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());

        let typed_program = result.unwrap();
        let function = &typed_program[0];
        if let typed_ast::TypedAstNodeKind::Function { body, .. } = &function.kind {
            assert_eq!(body.len(), 3);

            if let typed_ast::TypedExprKind::Let { name, value, .. } = &body[0].kind {
                assert_eq!(name, "x");
                assert!(matches!(&value.typ, typed_ast::Type::Con { name, .. } if name == "i32"));
            } else {
                panic!("Expected Let expression");
            }

            if let typed_ast::TypedExprKind::Let { name, value, .. } = &body[1].kind {
                assert_eq!(name, "y");
                assert!(matches!(&value.typ, typed_ast::Type::Con { name, .. } if name == "i32"));
            } else {
                panic!("Expected Let expression");
            }

            if let typed_ast::TypedExprKind::Let { name, value, .. } = &body[2].kind {
                assert_eq!(name, "z");
                assert!(matches!(&value.typ, typed_ast::Type::Con { name, .. } if name == "i32"));
            } else {
                panic!("Expected Let expression");
            }
        } else {
            panic!("Expected Function node");
        }
    }

    #[test]
    fn test_lambda_inference() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_lambda_inference".to_string(),
                args: vec![],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::FunctionCall {
                        function: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Lambda {
                                params: vec![ast::FunctionParam {
                                    name: "x".to_string(),
                                    type_annotation: ast::TypeAnnotation {
                                        meta: empty_meta(),
                                        kind: ast::TypeAnnotationKind::Constructor {
                                            name: "i32".to_string(),
                                            generic_args: vec![],
                                        },
                                    },
                                }],
                                return_type: None,
                                body: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::BinaryOp {
                                        left: Box::new(ast::Expr {
                                            meta: empty_meta(),
                                            kind: ast::ExprKind::Variable("x".to_string()),
                                        }),
                                        operator: "*".to_string(),
                                        right: Box::new(ast::Expr {
                                            meta: empty_meta(),
                                            kind: ast::ExprKind::Literal(ast::Literal::Integer(2)),
                                        }),
                                    },
                                }),
                            },
                        }),
                        args: vec![ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Literal(ast::Literal::Integer(10)),
                        }],
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());

        let typed_program = result.unwrap();
        let function = &typed_program[0];
        if let typed_ast::TypedAstNodeKind::Function { body, .. } = &function.kind {
            assert_eq!(body.len(), 1);
            let call_expr = &body[0];

            if let typed_ast::TypedExprKind::FunctionCall { function, args, .. } = &call_expr.kind {
                assert!(matches!(function.typ, typed_ast::Type::Fun { .. }));
                assert_eq!(args.len(), 1);
                assert!(matches!(&args[0].typ, typed_ast::Type::Con { name, .. } if name == "i32"));
                assert!(
                    matches!(&call_expr.typ, typed_ast::Type::Con { name, .. } if name == "i32")
                );

                if let typed_ast::TypedExprKind::Lambda { params, body, .. } = &function.kind {
                    assert_eq!(params.len(), 1);
                    assert_eq!(params[0].name, "x");
                    assert!(
                        matches!(&params[0].typ, typed_ast::Type::Con { name, .. } if name == "i32")
                    );
                    assert!(
                        matches!(&body.typ, typed_ast::Type::Con { name, .. } if name == "i32")
                    );
                }
            } else {
                panic!("Expected FunctionCall expression");
            }
        } else {
            panic!("Expected Function node");
        }
    }

    #[test]
    fn test_binary_op_inference() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_binary_inference".to_string(),
                args: vec![ast::FunctionParam {
                    name: "x".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "i32".to_string(),
                            generic_args: vec![],
                        },
                    },
                }],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Let {
                        name: "y".to_string(),
                        type_annotation: None,
                        value: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::BinaryOp {
                                left: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Variable("x".to_string()),
                                }),
                                operator: "+".to_string(),
                                right: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::Integer(5)),
                                }),
                            },
                        }),
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_complex_if_inference() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_complex_if".to_string(),
                args: vec![ast::FunctionParam {
                    name: "x".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "i32".to_string(),
                            generic_args: vec![],
                        },
                    },
                }],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::If {
                        condition: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::BinaryOp {
                                left: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Variable("x".to_string()),
                                }),
                                operator: ">".to_string(),
                                right: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::Integer(0)),
                                }),
                            },
                        }),
                        then_branch: vec![
                            ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::If {
                                    condition: Box::new(ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::BinaryOp {
                                            left: Box::new(ast::Expr {
                                                meta: empty_meta(),
                                                kind: ast::ExprKind::Variable("x".to_string()),
                                            }),
                                            operator: "<".to_string(),
                                            right: Box::new(ast::Expr {
                                                meta: empty_meta(),
                                                kind: ast::ExprKind::Literal(
                                                    ast::Literal::Integer(10),
                                                ),
                                            }),
                                        },
                                    }),
                                    then_branch: vec![ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::Literal(ast::Literal::Integer(1)),
                                    }],
                                    else_branch: Some(vec![ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::Literal(ast::Literal::Integer(2)),
                                    }]),
                                },
                            },
                            ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Literal(ast::Literal::Integer(0)),
                            },
                        ],
                        else_branch: Some(vec![ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Literal(ast::Literal::Integer(-1)),
                        }]),
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_match_inference() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_nested_match".to_string(),
                args: vec![ast::FunctionParam {
                    name: "x".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "i32".to_string(),
                            generic_args: vec![],
                        },
                    },
                }],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Match {
                        value: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Variable("x".to_string()),
                        }),
                        arms: vec![
                            ast::MatchArm {
                                pattern: ast::Pattern::Literal(ast::Literal::Integer(1)),
                                guard: None,
                                body: vec![ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Match {
                                        value: Box::new(ast::Expr {
                                            meta: empty_meta(),
                                            kind: ast::ExprKind::Literal(ast::Literal::Integer(2)),
                                        }),
                                        arms: vec![
                                            ast::MatchArm {
                                                pattern: ast::Pattern::Literal(
                                                    ast::Literal::Integer(2),
                                                ),
                                                guard: None,
                                                body: vec![ast::Expr {
                                                    meta: empty_meta(),
                                                    kind: ast::ExprKind::Literal(
                                                        ast::Literal::Integer(100),
                                                    ),
                                                }],
                                            },
                                            ast::MatchArm {
                                                pattern: ast::Pattern::Wildcard,
                                                guard: None,
                                                body: vec![ast::Expr {
                                                    meta: empty_meta(),
                                                    kind: ast::ExprKind::Literal(
                                                        ast::Literal::Integer(200),
                                                    ),
                                                }],
                                            },
                                        ],
                                    },
                                }],
                            },
                            ast::MatchArm {
                                pattern: ast::Pattern::Wildcard,
                                guard: None,
                                body: vec![ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::Integer(0)),
                                }],
                            },
                        ],
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_tuple_pattern_inference() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_tuple_pattern".to_string(),
                args: vec![ast::FunctionParam {
                    name: "x".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Tuple(vec![
                            ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "i32".to_string(),
                                    generic_args: vec![],
                                },
                            },
                            ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "i32".to_string(),
                                    generic_args: vec![],
                                },
                            },
                        ]),
                    },
                }],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Match {
                        value: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Variable("x".to_string()),
                        }),
                        arms: vec![
                            ast::MatchArm {
                                pattern: ast::Pattern::Tuple(vec![
                                    ast::Pattern::Literal(ast::Literal::Integer(1)),
                                    ast::Pattern::Literal(ast::Literal::Integer(2)),
                                ]),
                                guard: None,
                                body: vec![ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::Integer(10)),
                                }],
                            },
                            ast::MatchArm {
                                pattern: ast::Pattern::Tuple(vec![
                                    ast::Pattern::Literal(ast::Literal::Integer(3)),
                                    ast::Pattern::Literal(ast::Literal::Integer(4)),
                                ]),
                                guard: None,
                                body: vec![ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::Integer(20)),
                                }],
                            },
                            ast::MatchArm {
                                pattern: ast::Pattern::Wildcard,
                                guard: None,
                                body: vec![ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::Integer(0)),
                                }],
                            },
                        ],
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_or_pattern_inference() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_or_pattern".to_string(),
                args: vec![ast::FunctionParam {
                    name: "x".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "i32".to_string(),
                            generic_args: vec![],
                        },
                    },
                }],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Match {
                        value: Box::new(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Variable("x".to_string()),
                        }),
                        arms: vec![
                            ast::MatchArm {
                                pattern: ast::Pattern::Or(vec![
                                    ast::Pattern::Literal(ast::Literal::Integer(1)),
                                    ast::Pattern::Literal(ast::Literal::Integer(2)),
                                    ast::Pattern::Literal(ast::Literal::Integer(3)),
                                ]),
                                guard: None,
                                body: vec![ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::String(
                                        "small".to_string(),
                                    )),
                                }],
                            },
                            ast::MatchArm {
                                pattern: ast::Pattern::Or(vec![
                                    ast::Pattern::Literal(ast::Literal::Integer(4)),
                                    ast::Pattern::Literal(ast::Literal::Integer(5)),
                                    ast::Pattern::Literal(ast::Literal::Integer(6)),
                                ]),
                                guard: None,
                                body: vec![ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::String(
                                        "medium".to_string(),
                                    )),
                                }],
                            },
                            ast::MatchArm {
                                pattern: ast::Pattern::Wildcard,
                                guard: None,
                                body: vec![ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::String(
                                        "large".to_string(),
                                    )),
                                }],
                            },
                        ],
                    },
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_array_element_inference() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test_array_inference".to_string(),
                args: vec![ast::FunctionParam {
                    name: "x".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "i32".to_string(),
                            generic_args: vec![],
                        },
                    },
                }],
                return_type: None,
                generic_params: vec![],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Array(vec![
                        ast::ArrayElement::Value(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::Variable("x".to_string()),
                        }),
                        ast::ArrayElement::Value(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::BinaryOp {
                                left: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Variable("x".to_string()),
                                }),
                                operator: "+".to_string(),
                                right: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::Integer(1)),
                                }),
                            },
                        }),
                        ast::ArrayElement::Value(ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::BinaryOp {
                                left: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Variable("x".to_string()),
                                }),
                                operator: "*".to_string(),
                                right: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Literal(ast::Literal::Integer(2)),
                                }),
                            },
                        }),
                    ]),
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_trait_definition() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Trait {
                name: "Display".to_string(),
                generic_params: vec![],
                methods: vec![ast::MethodSignature {
                    name: "to_string".to_string(),
                    args: vec![ast::MethodParam::SelfParam],
                    return_type: Some(ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "String".to_string(),
                            generic_args: vec![],
                        },
                    }),
                    generic_params: vec![],
                    meta: empty_meta(),
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());

        let typed_program = result.unwrap();
        let trait_node = &typed_program[0];
        if let typed_ast::TypedAstNodeKind::Trait { methods, .. } = &trait_node.kind {
            assert_eq!(methods.len(), 1);
            assert_eq!(methods[0].name, "to_string");
            assert!(matches!(
                &methods[0].args[0],
                typed_ast::TypedMethodParam::SelfParam { .. }
            ));
        } else {
            panic!("Expected Trait node");
        }
    }

    #[test]
    fn test_impl_with_self_parameter() {
        let program = vec![
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Struct {
                    name: "Counter".to_string(),
                    generic_params: vec![],
                    fields: vec![ast::StructField {
                        name: "count".to_string(),
                        type_annotation: ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "i32".to_string(),
                                generic_args: vec![],
                            },
                        },
                        meta: empty_meta(),
                    }],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Impl {
                    trait_name: None,
                    for_type: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "Counter".to_string(),
                            generic_args: vec![],
                        },
                    },
                    methods: vec![ast::AstNode {
                        meta: empty_meta(),
                        kind: ast::AstNodeKind::Function {
                            name: "increment".to_string(),
                            args: vec![ast::FunctionParam {
                                name: "self".to_string(),
                                type_annotation: ast::TypeAnnotation {
                                    meta: empty_meta(),
                                    kind: ast::TypeAnnotationKind::Constructor {
                                        name: "Counter".to_string(),
                                        generic_args: vec![],
                                    },
                                },
                            }],
                            return_type: None,
                            generic_params: vec![],
                            body: vec![ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::DotAccess {
                                    value: Box::new(ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::Variable("self".to_string()),
                                    }),
                                    field: "count".to_string(),
                                },
                            }],
                        },
                    }],
                },
            },
        ];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_method_lookup_through_dot_access() {
        let program = vec![
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Struct {
                    name: "Value".to_string(),
                    generic_params: vec![],
                    fields: vec![ast::StructField {
                        name: "data".to_string(),
                        type_annotation: ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "i32".to_string(),
                                generic_args: vec![],
                            },
                        },
                        meta: empty_meta(),
                    }],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Impl {
                    trait_name: None,
                    for_type: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "Value".to_string(),
                            generic_args: vec![],
                        },
                    },
                    methods: vec![ast::AstNode {
                        meta: empty_meta(),
                        kind: ast::AstNodeKind::Function {
                            name: "get_data".to_string(),
                            args: vec![ast::FunctionParam {
                                name: "self".to_string(),
                                type_annotation: ast::TypeAnnotation {
                                    meta: empty_meta(),
                                    kind: ast::TypeAnnotationKind::Constructor {
                                        name: "Value".to_string(),
                                        generic_args: vec![],
                                    },
                                },
                            }],
                            return_type: Some(ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "i32".to_string(),
                                    generic_args: vec![],
                                },
                            }),
                            generic_params: vec![],
                            body: vec![ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::DotAccess {
                                    value: Box::new(ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::Variable("self".to_string()),
                                    }),
                                    field: "data".to_string(),
                                },
                            }],
                        },
                    }],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Function {
                    name: "test_method_call".to_string(),
                    args: vec![ast::FunctionParam {
                        name: "v".to_string(),
                        type_annotation: ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "Value".to_string(),
                                generic_args: vec![],
                            },
                        },
                    }],
                    return_type: None,
                    generic_params: vec![],
                    body: vec![ast::Expr {
                        meta: empty_meta(),
                        kind: ast::ExprKind::DotAccess {
                            value: Box::new(ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Variable("v".to_string()),
                            }),
                            field: "get_data".to_string(),
                        },
                    }],
                },
            },
        ];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());

        let typed_program = result.unwrap();
        let function = &typed_program[2];
        if let typed_ast::TypedAstNodeKind::Function { body, .. } = &function.kind {
            let dot_access = &body[0];
            assert!(matches!(dot_access.typ, typed_ast::Type::Fun { .. }));
        } else {
            panic!("Expected Function node");
        }
    }

    #[test]
    fn test_trait_impl_signature_validation() {
        let program = vec![
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Trait {
                    name: "Equal".to_string(),
                    generic_params: vec![],
                    methods: vec![ast::MethodSignature {
                        name: "equals".to_string(),
                        args: vec![
                            ast::MethodParam::SelfParam,
                            ast::MethodParam::TypedParam {
                                name: "other".to_string(),
                                type_annotation: ast::TypeAnnotation {
                                    meta: empty_meta(),
                                    kind: ast::TypeAnnotationKind::Constructor {
                                        name: "i32".to_string(),
                                        generic_args: vec![],
                                    },
                                },
                            },
                        ],
                        return_type: Some(ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "bool".to_string(),
                                generic_args: vec![],
                            },
                        }),
                        generic_params: vec![],
                        meta: empty_meta(),
                    }],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Struct {
                    name: "Number".to_string(),
                    generic_params: vec![],
                    fields: vec![ast::StructField {
                        name: "value".to_string(),
                        type_annotation: ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "i32".to_string(),
                                generic_args: vec![],
                            },
                        },
                        meta: empty_meta(),
                    }],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Impl {
                    trait_name: Some("Equal".to_string()),
                    for_type: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "Number".to_string(),
                            generic_args: vec![],
                        },
                    },
                    methods: vec![ast::AstNode {
                        meta: empty_meta(),
                        kind: ast::AstNodeKind::Function {
                            name: "equals".to_string(),
                            args: vec![
                                ast::FunctionParam {
                                    name: "self".to_string(),
                                    type_annotation: ast::TypeAnnotation {
                                        meta: empty_meta(),
                                        kind: ast::TypeAnnotationKind::Constructor {
                                            name: "Number".to_string(),
                                            generic_args: vec![],
                                        },
                                    },
                                },
                                ast::FunctionParam {
                                    name: "other".to_string(),
                                    type_annotation: ast::TypeAnnotation {
                                        meta: empty_meta(),
                                        kind: ast::TypeAnnotationKind::Constructor {
                                            name: "i32".to_string(),
                                            generic_args: vec![],
                                        },
                                    },
                                },
                            ],
                            return_type: Some(ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "bool".to_string(),
                                    generic_args: vec![],
                                },
                            }),
                            generic_params: vec![],
                            body: vec![ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Literal(ast::Literal::Boolean(true)),
                            }],
                        },
                    }],
                },
            },
        ];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_field_access_priority() {
        let program = vec![
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Struct {
                    name: "MyStruct".to_string(),
                    generic_params: vec![],
                    fields: vec![
                        ast::StructField {
                            name: "field1".to_string(),
                            type_annotation: ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "i32".to_string(),
                                    generic_args: vec![],
                                },
                            },
                            meta: empty_meta(),
                        },
                        ast::StructField {
                            name: "field2".to_string(),
                            type_annotation: ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "f32".to_string(),
                                    generic_args: vec![],
                                },
                            },
                            meta: empty_meta(),
                        },
                    ],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Impl {
                    trait_name: None,
                    for_type: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "MyStruct".to_string(),
                            generic_args: vec![],
                        },
                    },
                    methods: vec![ast::AstNode {
                        meta: empty_meta(),
                        kind: ast::AstNodeKind::Function {
                            name: "method1".to_string(),
                            args: vec![ast::FunctionParam {
                                name: "self".to_string(),
                                type_annotation: ast::TypeAnnotation {
                                    meta: empty_meta(),
                                    kind: ast::TypeAnnotationKind::Constructor {
                                        name: "MyStruct".to_string(),
                                        generic_args: vec![],
                                    },
                                },
                            }],
                            return_type: None,
                            generic_params: vec![],
                            body: vec![],
                        },
                    }],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Function {
                    name: "test_priority".to_string(),
                    args: vec![ast::FunctionParam {
                        name: "s".to_string(),
                        type_annotation: ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "MyStruct".to_string(),
                                generic_args: vec![],
                            },
                        },
                    }],
                    return_type: None,
                    generic_params: vec![],
                    body: vec![
                        ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::DotAccess {
                                value: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Variable("s".to_string()),
                                }),
                                field: "method1".to_string(),
                            },
                        },
                        ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::DotAccess {
                                value: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Variable("s".to_string()),
                                }),
                                field: "field1".to_string(),
                            },
                        },
                        ast::Expr {
                            meta: empty_meta(),
                            kind: ast::ExprKind::DotAccess {
                                value: Box::new(ast::Expr {
                                    meta: empty_meta(),
                                    kind: ast::ExprKind::Variable("s".to_string()),
                                }),
                                field: "field2".to_string(),
                            },
                        },
                    ],
                },
            },
        ];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());

        let typed_program = result.unwrap();
        let function = &typed_program[2];
        if let typed_ast::TypedAstNodeKind::Function { body, .. } = &function.kind {
            assert!(matches!(&body[0].typ, typed_ast::Type::Fun { .. }));
            assert!(matches!(&body[1].typ, typed_ast::Type::Con { name, .. } if name == "i32"));
            assert!(matches!(&body[2].typ, typed_ast::Type::Con { name, .. } if name == "f32"));
        } else {
            panic!("Expected Function node");
        }
    }

    #[test]
    fn test_impl_trait_with_self_constraint() {
        let program = vec![
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Trait {
                    name: "Comparable".to_string(),
                    generic_params: vec![],
                    methods: vec![ast::MethodSignature {
                        name: "compare".to_string(),
                        args: vec![
                            ast::MethodParam::SelfParam,
                            ast::MethodParam::TypedParam {
                                name: "other".to_string(),
                                type_annotation: ast::TypeAnnotation {
                                    meta: empty_meta(),
                                    kind: ast::TypeAnnotationKind::Constructor {
                                        name: "Comparable".to_string(),
                                        generic_args: vec![],
                                    },
                                },
                            },
                        ],
                        return_type: Some(ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "i32".to_string(),
                                generic_args: vec![],
                            },
                        }),
                        generic_params: vec![],
                        meta: empty_meta(),
                    }],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Struct {
                    name: "IntWrapper".to_string(),
                    generic_params: vec![],
                    fields: vec![ast::StructField {
                        name: "val".to_string(),
                        type_annotation: ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "i32".to_string(),
                                generic_args: vec![],
                            },
                        },
                        meta: empty_meta(),
                    }],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Impl {
                    trait_name: Some("Comparable".to_string()),
                    for_type: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "IntWrapper".to_string(),
                            generic_args: vec![],
                        },
                    },
                    methods: vec![ast::AstNode {
                        meta: empty_meta(),
                        kind: ast::AstNodeKind::Function {
                            name: "compare".to_string(),
                            args: vec![
                                ast::FunctionParam {
                                    name: "self".to_string(),
                                    type_annotation: ast::TypeAnnotation {
                                        meta: empty_meta(),
                                        kind: ast::TypeAnnotationKind::Constructor {
                                            name: "IntWrapper".to_string(),
                                            generic_args: vec![],
                                        },
                                    },
                                },
                                ast::FunctionParam {
                                    name: "other".to_string(),
                                    type_annotation: ast::TypeAnnotation {
                                        meta: empty_meta(),
                                        kind: ast::TypeAnnotationKind::Constructor {
                                            name: "IntWrapper".to_string(),
                                            generic_args: vec![],
                                        },
                                    },
                                },
                            ],
                            return_type: Some(ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "i32".to_string(),
                                    generic_args: vec![],
                                },
                            }),
                            generic_params: vec![],
                            body: vec![ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::BinaryOp {
                                    left: Box::new(ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::DotAccess {
                                            value: Box::new(ast::Expr {
                                                meta: empty_meta(),
                                                kind: ast::ExprKind::Variable("self".to_string()),
                                            }),
                                            field: "val".to_string(),
                                        },
                                    }),
                                    operator: "-".to_string(),
                                    right: Box::new(ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::DotAccess {
                                            value: Box::new(ast::Expr {
                                                meta: empty_meta(),
                                                kind: ast::ExprKind::Variable("other".to_string()),
                                            }),
                                            field: "val".to_string(),
                                        },
                                    }),
                                },
                            }],
                        },
                    }],
                },
            },
        ];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_issue1_generic_constraint_validation() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Function {
                name: "test".to_string(),
                args: vec![ast::FunctionParam {
                    name: "x".to_string(),
                    type_annotation: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "T".to_string(),
                            generic_args: vec![],
                        },
                    },
                }],
                return_type: None,
                generic_params: vec![ast::GenericParam {
                    name: "T".to_string(),
                    constraints: vec![ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "NonExistentTrait".to_string(),
                            generic_args: vec![],
                        },
                    }],
                }],
                body: vec![ast::Expr {
                    meta: empty_meta(),
                    kind: ast::ExprKind::Literal(ast::Literal::Integer(1)),
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("trait 'NonExistentTrait' not defined"));
    }

    #[test]
    fn test_issue2_trait_method_generics() {
        let program = vec![
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Trait {
                    name: "Converter".to_string(),
                    generic_params: vec![],
                    methods: vec![ast::MethodSignature {
                        name: "convert".to_string(),
                        args: vec![
                            ast::MethodParam::SelfParam,
                            ast::MethodParam::TypedParam {
                                name: "other".to_string(),
                                type_annotation: ast::TypeAnnotation {
                                    meta: empty_meta(),
                                    kind: ast::TypeAnnotationKind::Constructor {
                                        name: "U".to_string(),
                                        generic_args: vec![],
                                    },
                                },
                            },
                        ],
                        return_type: Some(ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "U".to_string(),
                                generic_args: vec![],
                            },
                        }),
                        generic_params: vec![ast::GenericParam {
                            name: "U".to_string(),
                            constraints: vec![],
                        }],
                        meta: empty_meta(),
                    }],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Struct {
                    name: "MyType".to_string(),
                    generic_params: vec![],
                    fields: vec![],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Impl {
                    trait_name: Some("Converter".to_string()),
                    for_type: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "MyType".to_string(),
                            generic_args: vec![],
                        },
                    },
                    methods: vec![ast::AstNode {
                        meta: empty_meta(),
                        kind: ast::AstNodeKind::Function {
                            name: "convert".to_string(),
                            args: vec![
                                ast::FunctionParam {
                                    name: "self".to_string(),
                                    type_annotation: ast::TypeAnnotation {
                                        meta: empty_meta(),
                                        kind: ast::TypeAnnotationKind::Constructor {
                                            name: "MyType".to_string(),
                                            generic_args: vec![],
                                        },
                                    },
                                },
                                ast::FunctionParam {
                                    name: "other".to_string(),
                                    type_annotation: ast::TypeAnnotation {
                                        meta: empty_meta(),
                                        kind: ast::TypeAnnotationKind::Constructor {
                                            name: "i32".to_string(),
                                            generic_args: vec![],
                                        },
                                    },
                                },
                            ],
                            return_type: Some(ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "i32".to_string(),
                                    generic_args: vec![],
                                },
                            }),
                            generic_params: vec![],
                            body: vec![ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Variable("other".to_string()),
                            }],
                        },
                    }],
                },
            },
        ];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_issue3_impl_method_generics() {
        let program = vec![
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Struct {
                    name: "Container".to_string(),
                    generic_params: vec![ast::GenericParam {
                        name: "T".to_string(),
                        constraints: vec![],
                    }],
                    fields: vec![ast::StructField {
                        name: "value".to_string(),
                        type_annotation: ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "T".to_string(),
                                generic_args: vec![],
                            },
                        },
                        meta: empty_meta(),
                    }],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Impl {
                    trait_name: None,
                    for_type: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "Container".to_string(),
                            generic_args: vec![ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "i32".to_string(),
                                    generic_args: vec![],
                                },
                            }],
                        },
                    },
                    methods: vec![ast::AstNode {
                        meta: empty_meta(),
                        kind: ast::AstNodeKind::Function {
                            name: "get_value".to_string(),
                            args: vec![ast::FunctionParam {
                                name: "self".to_string(),
                                type_annotation: ast::TypeAnnotation {
                                    meta: empty_meta(),
                                    kind: ast::TypeAnnotationKind::Constructor {
                                        name: "Container".to_string(),
                                        generic_args: vec![ast::TypeAnnotation {
                                            meta: empty_meta(),
                                            kind: ast::TypeAnnotationKind::Constructor {
                                                name: "i32".to_string(),
                                                generic_args: vec![],
                                            },
                                        }],
                                    },
                                },
                            }],
                            return_type: Some(ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "i32".to_string(),
                                    generic_args: vec![],
                                },
                            }),
                            generic_params: vec![],
                            body: vec![ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::DotAccess {
                                    value: Box::new(ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::Variable("self".to_string()),
                                    }),
                                    field: "value".to_string(),
                                },
                            }],
                        },
                    }],
                },
            },
        ];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());

        let typed_program = result.unwrap();
        if let typed_ast::TypedAstNodeKind::Impl { methods, .. } = &typed_program[1].kind {
            if let typed_ast::TypedAstNodeKind::Function {
                args, return_type, ..
            } = &methods[0].kind
            {
                assert_eq!(args.len(), 1);
                assert_eq!(args[0].name, "self");
                if let typed_ast::Type::Con {
                    name,
                    args: type_args,
                } = &args[0].typ
                {
                    assert_eq!(name, "Container");
                    assert_eq!(type_args.len(), 1);
                    if let typed_ast::Type::Con { name: arg_name, .. } = &type_args[0] {
                        assert_eq!(arg_name, "i32");
                    }
                }
            }
        }
    }

    #[test]
    fn test_issue4_trait_generic_params_propagation() {
        let program = vec![
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Trait {
                    name: "Display".to_string(),
                    generic_params: vec![ast::GenericParam {
                        name: "T".to_string(),
                        constraints: vec![],
                    }],
                    methods: vec![ast::MethodSignature {
                        name: "show".to_string(),
                        args: vec![ast::MethodParam::SelfParam],
                        return_type: Some(ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "String".to_string(),
                                generic_args: vec![],
                            },
                        }),
                        generic_params: vec![],
                        meta: empty_meta(),
                    }],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Struct {
                    name: "MyType".to_string(),
                    generic_params: vec![],
                    fields: vec![],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Impl {
                    trait_name: Some("Display".to_string()),
                    for_type: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "MyType".to_string(),
                            generic_args: vec![],
                        },
                    },
                    methods: vec![ast::AstNode {
                        meta: empty_meta(),
                        kind: ast::AstNodeKind::Function {
                            name: "show".to_string(),
                            args: vec![ast::FunctionParam {
                                name: "self".to_string(),
                                type_annotation: ast::TypeAnnotation {
                                    meta: empty_meta(),
                                    kind: ast::TypeAnnotationKind::Constructor {
                                        name: "MyType".to_string(),
                                        generic_args: vec![],
                                    },
                                },
                            }],
                            return_type: Some(ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "String".to_string(),
                                    generic_args: vec![],
                                },
                            }),
                            generic_params: vec![],
                            body: vec![ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Literal(ast::Literal::String(
                                    "hello".to_string(),
                                )),
                            }],
                        },
                    }],
                },
            },
        ];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_issue5_self_in_trait_signatures() {
        let program = vec![
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Trait {
                    name: "Clone".to_string(),
                    generic_params: vec![],
                    methods: vec![ast::MethodSignature {
                        name: "clone".to_string(),
                        args: vec![ast::MethodParam::SelfParam],
                        return_type: Some(ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "Self".to_string(),
                                generic_args: vec![],
                            },
                        }),
                        generic_params: vec![],
                        meta: empty_meta(),
                    }],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Struct {
                    name: "Point".to_string(),
                    generic_params: vec![],
                    fields: vec![ast::StructField {
                        name: "x".to_string(),
                        type_annotation: ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "i32".to_string(),
                                generic_args: vec![],
                            },
                        },
                        meta: empty_meta(),
                    }],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Impl {
                    trait_name: Some("Clone".to_string()),
                    for_type: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "Point".to_string(),
                            generic_args: vec![],
                        },
                    },
                    methods: vec![ast::AstNode {
                        meta: empty_meta(),
                        kind: ast::AstNodeKind::Function {
                            name: "clone".to_string(),
                            args: vec![ast::FunctionParam {
                                name: "self".to_string(),
                                type_annotation: ast::TypeAnnotation {
                                    meta: empty_meta(),
                                    kind: ast::TypeAnnotationKind::Constructor {
                                        name: "Point".to_string(),
                                        generic_args: vec![],
                                    },
                                },
                            }],
                            return_type: Some(ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "Point".to_string(),
                                    generic_args: vec![],
                                },
                            }),
                            generic_params: vec![],
                            body: vec![ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Variable("self".to_string()),
                            }],
                        },
                    }],
                },
            },
        ];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_issue7_generic_struct_init_validation() {
        let program = vec![
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Struct {
                    name: "Pair".to_string(),
                    generic_params: vec![ast::GenericParam {
                        name: "T".to_string(),
                        constraints: vec![],
                    }],
                    fields: vec![
                        ast::StructField {
                            name: "first".to_string(),
                            type_annotation: ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "T".to_string(),
                                    generic_args: vec![],
                                },
                            },
                            meta: empty_meta(),
                        },
                        ast::StructField {
                            name: "second".to_string(),
                            type_annotation: ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "T".to_string(),
                                    generic_args: vec![],
                                },
                            },
                            meta: empty_meta(),
                        },
                    ],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Function {
                    name: "test".to_string(),
                    args: vec![],
                    return_type: None,
                    generic_params: vec![],
                    body: vec![ast::Expr {
                        meta: empty_meta(),
                        kind: ast::ExprKind::StructInit {
                            name: "Pair".to_string(),
                            fields: vec![
                                ast::StructInitField::Field {
                                    name: "first".to_string(),
                                    value: ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::Literal(ast::Literal::Integer(1)),
                                    },
                                },
                                ast::StructInitField::Field {
                                    name: "second".to_string(),
                                    value: ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::Literal(ast::Literal::Integer(2)),
                                    },
                                },
                            ],
                        },
                    }],
                },
            },
        ];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_issue12_pattern_matching_enum_variants() {
        let program = vec![
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Enum {
                    name: "Maybe".to_string(),
                    generic_params: vec![ast::GenericParam {
                        name: "T".to_string(),
                        constraints: vec![],
                    }],
                    variants: vec![
                        ast::EnumVariant::Tuple {
                            name: "Just".to_string(),
                            fields: vec![ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "T".to_string(),
                                    generic_args: vec![],
                                },
                            }],
                            meta: empty_meta(),
                        },
                        ast::EnumVariant::Tuple {
                            name: "Nothing".to_string(),
                            fields: vec![],
                            meta: empty_meta(),
                        },
                    ],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Function {
                    name: "test".to_string(),
                    args: vec![ast::FunctionParam {
                        name: "m".to_string(),
                        type_annotation: ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "Maybe".to_string(),
                                generic_args: vec![ast::TypeAnnotation {
                                    meta: empty_meta(),
                                    kind: ast::TypeAnnotationKind::Constructor {
                                        name: "i32".to_string(),
                                        generic_args: vec![],
                                    },
                                }],
                            },
                        },
                    }],
                    return_type: None,
                    generic_params: vec![],
                    body: vec![ast::Expr {
                        meta: empty_meta(),
                        kind: ast::ExprKind::Match {
                            value: Box::new(ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Variable("m".to_string()),
                            }),
                            arms: vec![
                                ast::MatchArm {
                                    pattern: ast::Pattern::EnumVariant {
                                        name: "Just".to_string(),
                                        fields: vec![ast::Pattern::Variable("x".to_string())],
                                    },
                                    guard: None,
                                    body: vec![ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::Variable("x".to_string()),
                                    }],
                                },
                                ast::MatchArm {
                                    pattern: ast::Pattern::EnumVariant {
                                        name: "Nothing".to_string(),
                                        fields: vec![],
                                    },
                                    guard: None,
                                    body: vec![ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::Literal(ast::Literal::Integer(0)),
                                    }],
                                },
                            ],
                        },
                    }],
                },
            },
        ];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_issue16_generic_type_arg_constraint_checking() {
        let program = vec![
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Trait {
                    name: "Display".to_string(),
                    generic_params: vec![],
                    methods: vec![],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Struct {
                    name: "BadStruct".to_string(),
                    generic_params: vec![ast::GenericParam {
                        name: "T".to_string(),
                        constraints: vec![],
                    }],
                    fields: vec![],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Impl {
                    trait_name: Some("Display".to_string()),
                    for_type: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "BadStruct".to_string(),
                            generic_args: vec![ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "i32".to_string(),
                                    generic_args: vec![],
                                },
                            }],
                        },
                    },
                    methods: vec![],
                },
            },
        ];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_critical1_method_generics_as_type_vars() {
        let program = vec![
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Trait {
                    name: "Converter".to_string(),
                    generic_params: vec![],
                    methods: vec![ast::MethodSignature {
                        name: "convert".to_string(),
                        args: vec![
                            ast::MethodParam::SelfParam,
                            ast::MethodParam::TypedParam {
                                name: "x".to_string(),
                                type_annotation: ast::TypeAnnotation {
                                    meta: empty_meta(),
                                    kind: ast::TypeAnnotationKind::Constructor {
                                        name: "U".to_string(),
                                        generic_args: vec![],
                                    },
                                },
                            },
                        ],
                        return_type: Some(ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "U".to_string(),
                                generic_args: vec![],
                            },
                        }),
                        generic_params: vec![ast::GenericParam {
                            name: "U".to_string(),
                            constraints: vec![],
                        }],
                        meta: empty_meta(),
                    }],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Struct {
                    name: "Wrapper".to_string(),
                    generic_params: vec![],
                    fields: vec![],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Impl {
                    trait_name: Some("Converter".to_string()),
                    for_type: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "Wrapper".to_string(),
                            generic_args: vec![],
                        },
                    },
                    methods: vec![ast::AstNode {
                        meta: empty_meta(),
                        kind: ast::AstNodeKind::Function {
                            name: "convert".to_string(),
                            args: vec![
                                ast::FunctionParam {
                                    name: "self".to_string(),
                                    type_annotation: ast::TypeAnnotation {
                                        meta: empty_meta(),
                                        kind: ast::TypeAnnotationKind::Constructor {
                                            name: "Wrapper".to_string(),
                                            generic_args: vec![],
                                        },
                                    },
                                },
                                ast::FunctionParam {
                                    name: "x".to_string(),
                                    type_annotation: ast::TypeAnnotation {
                                        meta: empty_meta(),
                                        kind: ast::TypeAnnotationKind::Constructor {
                                            name: "i32".to_string(),
                                            generic_args: vec![],
                                        },
                                    },
                                },
                            ],
                            return_type: Some(ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "i32".to_string(),
                                    generic_args: vec![],
                                },
                            }),
                            generic_params: vec![],
                            body: vec![ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Variable("x".to_string()),
                            }],
                        },
                    }],
                },
            },
        ];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());

        let typed_program = result.unwrap();
        if let typed_ast::TypedAstNodeKind::Trait { methods, .. } = &typed_program[0].kind {
            assert_eq!(methods.len(), 1);
            let method = &methods[0];
            assert_eq!(method.args.len(), 2);
            match &method.args[1] {
                typed_ast::TypedMethodParam::TypedParam { typ, .. } => {
                    if let typed_ast::Type::Var(_) = typ {
                    } else {
                        panic!("Method generic param should be Type::Var, not Type::Con");
                    }
                }
                _ => panic!("Expected TypedParam"),
            }
        }
    }

    #[test]
    fn test_critical2_method_generic_constraints_validated() {
        let program = vec![ast::AstNode {
            meta: empty_meta(),
            kind: ast::AstNodeKind::Trait {
                name: "ValidTrait".to_string(),
                generic_params: vec![],
                methods: vec![ast::MethodSignature {
                    name: "process".to_string(),
                    args: vec![
                        ast::MethodParam::SelfParam,
                        ast::MethodParam::TypedParam {
                            name: "x".to_string(),
                            type_annotation: ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "T".to_string(),
                                    generic_args: vec![],
                                },
                            },
                        },
                    ],
                    return_type: Some(ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "T".to_string(),
                            generic_args: vec![],
                        },
                    }),
                    generic_params: vec![ast::GenericParam {
                        name: "T".to_string(),
                        constraints: vec![],
                    }],
                    meta: empty_meta(),
                }],
            },
        }];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_critical4_self_as_implicit_generic() {
        let program = vec![
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Trait {
                    name: "Clone".to_string(),
                    generic_params: vec![],
                    methods: vec![ast::MethodSignature {
                        name: "clone".to_string(),
                        args: vec![ast::MethodParam::SelfParam],
                        return_type: Some(ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "Self".to_string(),
                                generic_args: vec![],
                            },
                        }),
                        generic_params: vec![],
                        meta: empty_meta(),
                    }],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Struct {
                    name: "Value".to_string(),
                    generic_params: vec![],
                    fields: vec![],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Impl {
                    trait_name: Some("Clone".to_string()),
                    for_type: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "Value".to_string(),
                            generic_args: vec![],
                        },
                    },
                    methods: vec![ast::AstNode {
                        meta: empty_meta(),
                        kind: ast::AstNodeKind::Function {
                            name: "clone".to_string(),
                            args: vec![ast::FunctionParam {
                                name: "self".to_string(),
                                type_annotation: ast::TypeAnnotation {
                                    meta: empty_meta(),
                                    kind: ast::TypeAnnotationKind::Constructor {
                                        name: "Value".to_string(),
                                        generic_args: vec![],
                                    },
                                },
                            }],
                            return_type: Some(ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "Value".to_string(),
                                    generic_args: vec![],
                                },
                            }),
                            generic_params: vec![],
                            body: vec![ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::Variable("self".to_string()),
                            }],
                        },
                    }],
                },
            },
        ];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_moderate5_method_generic_scoping() {
        let program = vec![
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Trait {
                    name: "Mapper".to_string(),
                    generic_params: vec![ast::GenericParam {
                        name: "T".to_string(),
                        constraints: vec![],
                    }],
                    methods: vec![ast::MethodSignature {
                        name: "map".to_string(),
                        args: vec![
                            ast::MethodParam::SelfParam,
                            ast::MethodParam::TypedParam {
                                name: "f".to_string(),
                                type_annotation: ast::TypeAnnotation {
                                    meta: empty_meta(),
                                    kind: ast::TypeAnnotationKind::Function {
                                        args: vec![ast::TypeAnnotation {
                                            meta: empty_meta(),
                                            kind: ast::TypeAnnotationKind::Constructor {
                                                name: "T".to_string(),
                                                generic_args: vec![],
                                            },
                                        }],
                                        return_type: Box::new(ast::TypeAnnotation {
                                            meta: empty_meta(),
                                            kind: ast::TypeAnnotationKind::Constructor {
                                                name: "U".to_string(),
                                                generic_args: vec![],
                                            },
                                        }),
                                    },
                                },
                            },
                        ],
                        return_type: Some(ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "U".to_string(),
                                generic_args: vec![],
                            },
                        }),
                        generic_params: vec![ast::GenericParam {
                            name: "U".to_string(),
                            constraints: vec![],
                        }],
                        meta: empty_meta(),
                    }],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Struct {
                    name: "Container".to_string(),
                    generic_params: vec![ast::GenericParam {
                        name: "T".to_string(),
                        constraints: vec![],
                    }],
                    fields: vec![ast::StructField {
                        name: "value".to_string(),
                        type_annotation: ast::TypeAnnotation {
                            meta: empty_meta(),
                            kind: ast::TypeAnnotationKind::Constructor {
                                name: "T".to_string(),
                                generic_args: vec![],
                            },
                        },
                        meta: empty_meta(),
                    }],
                },
            },
            ast::AstNode {
                meta: empty_meta(),
                kind: ast::AstNodeKind::Impl {
                    trait_name: Some("Mapper".to_string()),
                    for_type: ast::TypeAnnotation {
                        meta: empty_meta(),
                        kind: ast::TypeAnnotationKind::Constructor {
                            name: "Container".to_string(),
                            generic_args: vec![ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "i32".to_string(),
                                    generic_args: vec![],
                                },
                            }],
                        },
                    },
                    methods: vec![ast::AstNode {
                        meta: empty_meta(),
                        kind: ast::AstNodeKind::Function {
                            name: "map".to_string(),
                            args: vec![
                                ast::FunctionParam {
                                    name: "self".to_string(),
                                    type_annotation: ast::TypeAnnotation {
                                        meta: empty_meta(),
                                        kind: ast::TypeAnnotationKind::Constructor {
                                            name: "Container".to_string(),
                                            generic_args: vec![ast::TypeAnnotation {
                                                meta: empty_meta(),
                                                kind: ast::TypeAnnotationKind::Constructor {
                                                    name: "i32".to_string(),
                                                    generic_args: vec![],
                                                },
                                            }],
                                        },
                                    },
                                },
                                ast::FunctionParam {
                                    name: "f".to_string(),
                                    type_annotation: ast::TypeAnnotation {
                                        meta: empty_meta(),
                                        kind: ast::TypeAnnotationKind::Function {
                                            args: vec![ast::TypeAnnotation {
                                                meta: empty_meta(),
                                                kind: ast::TypeAnnotationKind::Constructor {
                                                    name: "i32".to_string(),
                                                    generic_args: vec![],
                                                },
                                            }],
                                            return_type: Box::new(ast::TypeAnnotation {
                                                meta: empty_meta(),
                                                kind: ast::TypeAnnotationKind::Constructor {
                                                    name: "i32".to_string(),
                                                    generic_args: vec![],
                                                },
                                            }),
                                        },
                                    },
                                },
                            ],
                            return_type: Some(ast::TypeAnnotation {
                                meta: empty_meta(),
                                kind: ast::TypeAnnotationKind::Constructor {
                                    name: "i32".to_string(),
                                    generic_args: vec![],
                                },
                            }),
                            generic_params: vec![],
                            body: vec![ast::Expr {
                                meta: empty_meta(),
                                kind: ast::ExprKind::FunctionCall {
                                    function: Box::new(ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::Variable("f".to_string()),
                                    }),
                                    args: vec![ast::Expr {
                                        meta: empty_meta(),
                                        kind: ast::ExprKind::DotAccess {
                                            value: Box::new(ast::Expr {
                                                meta: empty_meta(),
                                                kind: ast::ExprKind::Variable("self".to_string()),
                                            }),
                                            field: "value".to_string(),
                                        },
                                    }],
                                },
                            }],
                        },
                    }],
                },
            },
        ];
        let checker = typechecker::TypeChecker::new();
        let result = checker.typecheck(program);
        assert!(result.is_ok());
    }
}
