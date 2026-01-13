use crate::ast;
use crate::typed_ast;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Constraint {
    Eq(typed_ast::Type, typed_ast::Type),
    Class(String, Vec<typed_ast::Type>),
}

#[derive(Clone)]
pub struct Substitution(HashMap<typed_ast::TypeVar, typed_ast::Type>);

impl Substitution {
    fn new() -> Self {
        Substitution(HashMap::new())
    }

    fn apply(&self, t: &typed_ast::Type) -> typed_ast::Type {
        match t {
            typed_ast::Type::Var(v) => self.0.get(v).cloned().unwrap_or_else(|| t.clone()),
            typed_ast::Type::Con { name, args } => typed_ast::Type::Con {
                name: name.clone(),
                args: args.iter().map(|a| self.apply(a)).collect(),
            },
            typed_ast::Type::Tuple(ts) => {
                typed_ast::Type::Tuple(ts.iter().map(|t| self.apply(t)).collect())
            }
            typed_ast::Type::Fun { args, ret } => typed_ast::Type::Fun {
                args: args.iter().map(|a| self.apply(a)).collect(),
                ret: Box::new(self.apply(ret)),
            },
            typed_ast::Type::Array { elem, size } => typed_ast::Type::Array {
                elem: Box::new(self.apply(elem)),
                size: *size,
            },
            typed_ast::Type::Row(fields) => typed_ast::Type::Row(
                fields
                    .iter()
                    .map(|(n, t)| (n.clone(), self.apply(t)))
                    .collect(),
            ),
            typed_ast::Type::Rec(t) => typed_ast::Type::Rec(Box::new(self.apply(t))),
        }
    }

    fn compose(&self, other: &Substitution) -> Substitution {
        let mut new_sub = self.0.clone();
        for (v, t) in &other.0 {
            new_sub.insert(v.clone(), self.apply(t));
        }
        Substitution(new_sub)
    }

    fn unify(&mut self, t1: &typed_ast::Type, t2: &typed_ast::Type) -> Result<(), String> {
        let t1 = self.apply(t1);
        let t2 = self.apply(t2);
        match (&t1, &t2) {
            (typed_ast::Type::Var(v1), typed_ast::Type::Var(v2)) if v1 == v2 => Ok(()),
            (typed_ast::Type::Var(v), _) => {
                if occurs_check(v, &t2) {
                    return Err("occurs check failed".to_string());
                }
                self.0.insert(v.clone(), t2);
                Ok(())
            }
            (_, typed_ast::Type::Var(v)) => self.unify(&t2, &t1),
            (
                typed_ast::Type::Con { name: n1, args: a1 },
                typed_ast::Type::Con { name: n2, args: a2 },
            ) if n1 == n2 && a1.len() == a2.len() => {
                for (a1, a2) in a1.iter().zip(a2) {
                    self.unify(a1, a2)?;
                }
                Ok(())
            }
            (typed_ast::Type::Tuple(ts1), typed_ast::Type::Tuple(ts2))
                if ts1.len() == ts2.len() =>
            {
                for (t1, t2) in ts1.iter().zip(ts2) {
                    self.unify(t1, t2)?;
                }
                Ok(())
            }
            (
                typed_ast::Type::Fun { args: a1, ret: r1 },
                typed_ast::Type::Fun { args: a2, ret: r2 },
            ) if a1.len() == a2.len() => {
                for (a1, a2) in a1.iter().zip(a2) {
                    self.unify(a1, a2)?;
                }
                self.unify(r1, r2)
            }
            (
                typed_ast::Type::Array { elem: e1, size: s1 },
                typed_ast::Type::Array { elem: e2, size: s2 },
            ) if s1 == s2 => self.unify(e1, e2),
            (typed_ast::Type::Row(f1), typed_ast::Type::Row(f2)) => {
                if f1.len() == f2.len() {
                    for ((n1, t1), (n2, t2)) in f1.iter().zip(f2) {
                        if n1 != n2 {
                            return Err("row fields mismatch".to_string());
                        }
                        self.unify(t1, t2)?;
                    }
                    Ok(())
                } else {
                    Err("row length mismatch".to_string())
                }
            }
            (typed_ast::Type::Rec(t1), typed_ast::Type::Rec(t2)) => self.unify(t1, t2),
            _ => Err(format!("cannot unify {:?} and {:?}", t1, t2)),
        }
    }
}

fn occurs_check(v: &typed_ast::TypeVar, t: &typed_ast::Type) -> bool {
    match t {
        typed_ast::Type::Var(v2) => v == v2,
        typed_ast::Type::Con { args, .. } => args.iter().any(|a| occurs_check(v, a)),
        typed_ast::Type::Tuple(ts) => ts.iter().any(|t| occurs_check(v, t)),
        typed_ast::Type::Fun { args, ret } => {
            args.iter().any(|a| occurs_check(v, a)) || occurs_check(v, ret)
        }
        typed_ast::Type::Array { elem, .. } => occurs_check(v, elem),
        typed_ast::Type::Row(fields) => fields.iter().any(|(_, t)| occurs_check(v, t)),
        typed_ast::Type::Rec(t) => occurs_check(v, t),
    }
}

#[derive(Clone)]
struct FunctionSignature {
    generic_params: Vec<String>,
    typ: typed_ast::Type,
}

pub struct TypeChecker {
    next_var: usize,
    constraints: Vec<Constraint>,
    substitution: Substitution,
    env: HashMap<String, typed_ast::Type>,
    struct_env: HashMap<String, Vec<(String, typed_ast::Type)>>,
    enum_env: HashMap<String, Vec<String>>,
    trait_env: HashMap<String, HashMap<String, typed_ast::Type>>,
    impl_env: HashMap<String, HashMap<String, HashMap<String, typed_ast::Type>>>,
    generic_mapping: HashMap<String, typed_ast::Type>,
    function_env: HashMap<String, FunctionSignature>,
    current_impl_type: Option<typed_ast::Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            next_var: 0,
            constraints: vec![],
            substitution: Substitution::new(),
            env: HashMap::new(),
            struct_env: HashMap::new(),
            enum_env: HashMap::new(),
            trait_env: HashMap::new(),
            impl_env: HashMap::new(),
            generic_mapping: HashMap::new(),
            function_env: HashMap::new(),
            current_impl_type: None,
        }
    }

    fn fresh_var(&mut self) -> typed_ast::Type {
        let v = self.next_var;
        self.next_var += 1;
        typed_ast::Type::Var(typed_ast::TypeVar(v))
    }

    pub fn typecheck(mut self, program: ast::Program) -> Result<typed_ast::TypedProgram, String> {
        let mut typed_program = vec![];
        for node in program {
            let typed_node = self.typecheck_node(&node)?;
            typed_program.push(typed_node);
        }
        self.solve_constraints()?;
        Ok(self.apply_substitution_to_program(typed_program))
    }

    fn apply_substitution_to_program(
        &self,
        program: typed_ast::TypedProgram,
    ) -> typed_ast::TypedProgram {
        program
            .into_iter()
            .map(|node| self.apply_substitution_to_node(node))
            .collect()
    }

    fn apply_substitution_to_node(
        &self,
        mut node: typed_ast::TypedAstNode,
    ) -> typed_ast::TypedAstNode {
        node.typ = self.substitution.apply(&node.typ);
        node.kind = self.apply_substitution_to_kind(node.kind);
        node
    }

    fn apply_substitution_to_kind(
        &self,
        kind: typed_ast::TypedAstNodeKind,
    ) -> typed_ast::TypedAstNodeKind {
        match kind {
            typed_ast::TypedAstNodeKind::Function {
                name,
                args,
                return_type,
                generic_params,
                body,
            } => {
                let args = args
                    .into_iter()
                    .map(|mut arg| {
                        arg.typ = self.substitution.apply(&arg.typ);
                        arg.type_annotation =
                            self.apply_substitution_to_type_annotation(arg.type_annotation);
                        arg
                    })
                    .collect();
                let body = body
                    .into_iter()
                    .map(|mut expr| self.apply_substitution_to_expr(expr))
                    .collect();
                typed_ast::TypedAstNodeKind::Function {
                    name,
                    args,
                    return_type: return_type
                        .map(|rt| self.apply_substitution_to_type_annotation(rt)),
                    generic_params,
                    body,
                }
            }
            typed_ast::TypedAstNodeKind::Struct {
                name,
                generic_params,
                fields,
            } => {
                let fields = fields
                    .into_iter()
                    .map(|mut field| {
                        field.typ = self.substitution.apply(&field.typ);
                        field.type_annotation =
                            self.apply_substitution_to_type_annotation(field.type_annotation);
                        field
                    })
                    .collect();
                typed_ast::TypedAstNodeKind::Struct {
                    name,
                    generic_params,
                    fields,
                }
            }
            typed_ast::TypedAstNodeKind::Enum {
                name,
                generic_params,
                variants,
            } => {
                let variants = variants
                    .into_iter()
                    .map(|variant| match variant {
                        typed_ast::TypedEnumVariant::Tuple {
                            name,
                            fields,
                            meta,
                            typ,
                        } => {
                            let fields = fields
                                .into_iter()
                                .map(|t| self.apply_substitution_to_type_annotation(t))
                                .collect();
                            typed_ast::TypedEnumVariant::Tuple {
                                name,
                                fields,
                                meta,
                                typ: self.substitution.apply(&typ),
                            }
                        }
                        typed_ast::TypedEnumVariant::Struct {
                            name,
                            fields,
                            meta,
                            typ,
                        } => {
                            let fields = fields
                                .into_iter()
                                .map(|mut field| {
                                    field.typ = self.substitution.apply(&field.typ);
                                    field.type_annotation = self
                                        .apply_substitution_to_type_annotation(
                                            field.type_annotation,
                                        );
                                    field
                                })
                                .collect();
                            typed_ast::TypedEnumVariant::Struct {
                                name,
                                fields,
                                meta,
                                typ: self.substitution.apply(&typ),
                            }
                        }
                    })
                    .collect();
                typed_ast::TypedAstNodeKind::Enum {
                    name,
                    generic_params,
                    variants,
                }
            }
            typed_ast::TypedAstNodeKind::Import { path, items } => {
                typed_ast::TypedAstNodeKind::Import { path, items }
            }
            _ => kind,
        }
    }

    fn apply_substitution_to_expr(&self, mut expr: typed_ast::TypedExpr) -> typed_ast::TypedExpr {
        expr.typ = self.substitution.apply(&expr.typ);
        expr.kind = match expr.kind {
            typed_ast::TypedExprKind::Literal(l) => typed_ast::TypedExprKind::Literal(l),
            typed_ast::TypedExprKind::Variable(name) => typed_ast::TypedExprKind::Variable(name),
            typed_ast::TypedExprKind::BinaryOp {
                left,
                operator,
                right,
            } => typed_ast::TypedExprKind::BinaryOp {
                left: Box::new(self.apply_substitution_to_expr(*left)),
                operator,
                right: Box::new(self.apply_substitution_to_expr(*right)),
            },
            typed_ast::TypedExprKind::UnaryOp { operator, operand } => {
                typed_ast::TypedExprKind::UnaryOp {
                    operator,
                    operand: Box::new(self.apply_substitution_to_expr(*operand)),
                }
            }
            typed_ast::TypedExprKind::Let {
                name,
                type_annotation,
                value,
            } => {
                let type_annotation =
                    type_annotation.map(|ta| self.apply_substitution_to_type_annotation(ta));
                let value = Box::new(self.apply_substitution_to_expr(*value));
                typed_ast::TypedExprKind::Let {
                    name,
                    type_annotation,
                    value,
                }
            }
            typed_ast::TypedExprKind::Assign { target, value } => {
                typed_ast::TypedExprKind::Assign {
                    target: Box::new(self.apply_substitution_to_expr(*target)),
                    value: Box::new(self.apply_substitution_to_expr(*value)),
                }
            }
            typed_ast::TypedExprKind::Cast(expr, type_annotation) => {
                typed_ast::TypedExprKind::Cast(
                    Box::new(self.apply_substitution_to_expr(*expr)),
                    self.apply_substitution_to_type_annotation(type_annotation),
                )
            }
            typed_ast::TypedExprKind::EarlyReturn(expr) => typed_ast::TypedExprKind::EarlyReturn(
                Box::new(self.apply_substitution_to_expr(*expr)),
            ),
            typed_ast::TypedExprKind::DotAccess { value, field } => {
                typed_ast::TypedExprKind::DotAccess {
                    value: Box::new(self.apply_substitution_to_expr(*value)),
                    field,
                }
            }
            typed_ast::TypedExprKind::Array(elements) => {
                let elements = elements
                    .into_iter()
                    .map(|elem| match elem {
                        typed_ast::TypedArrayElement::Value(e) => {
                            typed_ast::TypedArrayElement::Value(self.apply_substitution_to_expr(e))
                        }
                        typed_ast::TypedArrayElement::Spread(e) => {
                            typed_ast::TypedArrayElement::Spread(self.apply_substitution_to_expr(e))
                        }
                    })
                    .collect();
                typed_ast::TypedExprKind::Array(elements)
            }
            typed_ast::TypedExprKind::Tuple(exprs) => {
                let exprs = exprs
                    .into_iter()
                    .map(|e| self.apply_substitution_to_expr(e))
                    .collect();
                typed_ast::TypedExprKind::Tuple(exprs)
            }
            typed_ast::TypedExprKind::Row(fields) => {
                let fields = fields
                    .into_iter()
                    .map(|f| match f {
                        typed_ast::TypedRowField::Field { name, value } => {
                            typed_ast::TypedRowField::Field {
                                name,
                                value: self.apply_substitution_to_expr(value),
                            }
                        }
                        typed_ast::TypedRowField::Spread(expr) => {
                            typed_ast::TypedRowField::Spread(self.apply_substitution_to_expr(expr))
                        }
                    })
                    .collect();
                typed_ast::TypedExprKind::Row(fields)
            }
            typed_ast::TypedExprKind::Index { value, index } => typed_ast::TypedExprKind::Index {
                value: Box::new(self.apply_substitution_to_expr(*value)),
                index: Box::new(self.apply_substitution_to_expr(*index)),
            },
            typed_ast::TypedExprKind::FunctionCall { function, args } => {
                let function = Box::new(self.apply_substitution_to_expr(*function));
                let args = args
                    .into_iter()
                    .map(|e| self.apply_substitution_to_expr(e))
                    .collect();
                typed_ast::TypedExprKind::FunctionCall { function, args }
            }
            typed_ast::TypedExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = Box::new(self.apply_substitution_to_expr(*condition));
                let then_branch = then_branch
                    .into_iter()
                    .map(|e| self.apply_substitution_to_expr(e))
                    .collect();
                let else_branch = else_branch.map(|branch| {
                    branch
                        .into_iter()
                        .map(|e| self.apply_substitution_to_expr(e))
                        .collect()
                });
                typed_ast::TypedExprKind::If {
                    condition,
                    then_branch,
                    else_branch,
                }
            }
            typed_ast::TypedExprKind::While { condition, body } => {
                let condition = Box::new(self.apply_substitution_to_expr(*condition));
                let body = body
                    .into_iter()
                    .map(|e| self.apply_substitution_to_expr(e))
                    .collect();
                typed_ast::TypedExprKind::While { condition, body }
            }
            typed_ast::TypedExprKind::For {
                iterator,
                iterable,
                body,
            } => {
                let iterable = Box::new(self.apply_substitution_to_expr(*iterable));
                let body = body
                    .into_iter()
                    .map(|e| self.apply_substitution_to_expr(e))
                    .collect();
                typed_ast::TypedExprKind::For {
                    iterator,
                    iterable,
                    body,
                }
            }
            typed_ast::TypedExprKind::Range {
                start,
                end,
                inclusive,
            } => typed_ast::TypedExprKind::Range {
                start: Box::new(self.apply_substitution_to_expr(*start)),
                end: Box::new(self.apply_substitution_to_expr(*end)),
                inclusive,
            },
            typed_ast::TypedExprKind::Break => typed_ast::TypedExprKind::Break,
            typed_ast::TypedExprKind::Continue => typed_ast::TypedExprKind::Continue,
            typed_ast::TypedExprKind::Return(value) => {
                let value = value.map(|e| Box::new(self.apply_substitution_to_expr(*e)));
                typed_ast::TypedExprKind::Return(value)
            }
            typed_ast::TypedExprKind::Block { expressions } => {
                let expressions = expressions
                    .into_iter()
                    .map(|e| self.apply_substitution_to_expr(e))
                    .collect();
                typed_ast::TypedExprKind::Block { expressions }
            }
            typed_ast::TypedExprKind::Do(expressions) => {
                let expressions = expressions
                    .into_iter()
                    .map(|e| self.apply_substitution_to_expr(e))
                    .collect();
                typed_ast::TypedExprKind::Do(expressions)
            }
            typed_ast::TypedExprKind::Lambda {
                params,
                return_type,
                body,
            } => {
                let params = params
                    .into_iter()
                    .map(|mut p| {
                        p.typ = self.substitution.apply(&p.typ);
                        p.type_annotation =
                            self.apply_substitution_to_type_annotation(p.type_annotation);
                        p
                    })
                    .collect();
                let return_type =
                    return_type.map(|rt| self.apply_substitution_to_type_annotation(rt));
                let body = Box::new(self.apply_substitution_to_expr(*body));
                typed_ast::TypedExprKind::Lambda {
                    params,
                    return_type,
                    body,
                }
            }
            typed_ast::TypedExprKind::Match { value, arms } => {
                let value = Box::new(self.apply_substitution_to_expr(*value));
                let arms = arms
                    .into_iter()
                    .map(|mut arm| {
                        arm.typ = self.substitution.apply(&arm.typ);
                        arm.body = arm
                            .body
                            .into_iter()
                            .map(|e| self.apply_substitution_to_expr(e))
                            .collect();
                        arm
                    })
                    .collect();
                typed_ast::TypedExprKind::Match { value, arms }
            }
            typed_ast::TypedExprKind::StructInit { name, fields } => {
                let fields = fields
                    .into_iter()
                    .map(|f| match f {
                        typed_ast::TypedStructInitField::Field { name, value } => {
                            typed_ast::TypedStructInitField::Field {
                                name,
                                value: self.apply_substitution_to_expr(value),
                            }
                        }
                        typed_ast::TypedStructInitField::Spread(expr) => {
                            typed_ast::TypedStructInitField::Spread(
                                self.apply_substitution_to_expr(expr),
                            )
                        }
                    })
                    .collect();
                typed_ast::TypedExprKind::StructInit { name, fields }
            }
            typed_ast::TypedExprKind::EnumInit {
                enum_name,
                variant_name,
                fields,
            } => {
                let fields = match fields {
                    typed_ast::TypedEnumInitFields::Tuple(args) => {
                        typed_ast::TypedEnumInitFields::Tuple(
                            args.into_iter()
                                .map(|e| self.apply_substitution_to_expr(e))
                                .collect(),
                        )
                    }
                    typed_ast::TypedEnumInitFields::Struct(fields) => {
                        typed_ast::TypedEnumInitFields::Struct(
                            fields
                                .into_iter()
                                .map(|f| match f {
                                    typed_ast::TypedStructInitField::Field { name, value } => {
                                        typed_ast::TypedStructInitField::Field {
                                            name,
                                            value: self.apply_substitution_to_expr(value),
                                        }
                                    }
                                    typed_ast::TypedStructInitField::Spread(expr) => {
                                        typed_ast::TypedStructInitField::Spread(
                                            self.apply_substitution_to_expr(expr),
                                        )
                                    }
                                })
                                .collect(),
                        )
                    }
                };
                typed_ast::TypedExprKind::EnumInit {
                    enum_name,
                    variant_name,
                    fields,
                }
            }
        };
        expr
    }

    fn apply_substitution_to_type_annotation(
        &self,
        ta: typed_ast::TypeAnnotation,
    ) -> typed_ast::TypeAnnotation {
        typed_ast::TypeAnnotation {
            meta: ta.meta,
            kind: match ta.kind {
                typed_ast::TypeAnnotationKind::Constructor { name, generic_args } => {
                    typed_ast::TypeAnnotationKind::Constructor { name, generic_args }
                }
                typed_ast::TypeAnnotationKind::Function { args, return_type } => {
                    let args = args
                        .into_iter()
                        .map(|ta| self.apply_substitution_to_type_annotation(ta))
                        .collect();
                    let return_type =
                        Box::new(self.apply_substitution_to_type_annotation(*return_type));
                    typed_ast::TypeAnnotationKind::Function { args, return_type }
                }
                typed_ast::TypeAnnotationKind::Array { element_type, size } => {
                    let element_type =
                        Box::new(self.apply_substitution_to_type_annotation(*element_type));
                    typed_ast::TypeAnnotationKind::Array { element_type, size }
                }
                typed_ast::TypeAnnotationKind::Tuple(types) => {
                    let types = types
                        .into_iter()
                        .map(|ta| self.apply_substitution_to_type_annotation(ta))
                        .collect();
                    typed_ast::TypeAnnotationKind::Tuple(types)
                }
                typed_ast::TypeAnnotationKind::Row(fields) => {
                    let fields = fields
                        .into_iter()
                        .map(|f| typed_ast::RowTypeField {
                            name: f.name,
                            type_annotation: self
                                .apply_substitution_to_type_annotation(f.type_annotation),
                        })
                        .collect();
                    typed_ast::TypeAnnotationKind::Row(fields)
                }
            },
        }
    }

    fn validate_constraint(&self, trait_name: &str) -> Result<(), String> {
        if !self.trait_env.contains_key(trait_name) {
            Err(format!("trait '{}' not defined", trait_name))
        } else {
            Ok(())
        }
    }

    fn process_generic_params(
        &mut self,
        generic_params: &[ast::GenericParam],
    ) -> Result<HashMap<String, typed_ast::Type>, String> {
        let mut generic_map = HashMap::new();
        for gp in generic_params {
            let tv = self.fresh_var();
            generic_map.insert(gp.name.clone(), tv.clone());

            for constraint in &gp.constraints {
                if let ast::TypeAnnotationKind::Constructor { name, .. } = &constraint.kind {
                    self.validate_constraint(name)?;
                    self.constraints
                        .push(Constraint::Class(name.clone(), vec![tv.clone()]));
                }
            }
        }
        Ok(generic_map)
    }

    fn process_method_generic_params(
        &mut self,
        generic_params: &[ast::GenericParam],
    ) -> Result<HashMap<String, typed_ast::Type>, String> {
        let mut generic_map = HashMap::new();
        for gp in generic_params {
            let tv = self.fresh_var();
            generic_map.insert(gp.name.clone(), tv.clone());

            for constraint in &gp.constraints {
                if let ast::TypeAnnotationKind::Constructor { name, .. } = &constraint.kind {
                    self.validate_constraint(name)?;
                    self.constraints
                        .push(Constraint::Class(name.clone(), vec![tv.clone()]));
                }
            }
        }
        Ok(generic_map)
    }

    fn typecheck_node(&mut self, node: &ast::AstNode) -> Result<typed_ast::TypedAstNode, String> {
        let typ = self.fresh_var();
        let kind = match &node.kind {
            ast::AstNodeKind::Function {
                name,
                args,
                return_type,
                generic_params,
                body,
            } => {
                let generic_map = self.process_generic_params(generic_params)?;
                let old_generic_mapping = std::mem::replace(&mut self.generic_mapping, generic_map);

                let mut arg_types = vec![];
                let is_method = self.current_impl_type.is_some();
                let first_is_self = is_method && args.first().map_or(false, |p| p.name == "self");

                for (i, param) in args.iter().enumerate() {
                    if first_is_self && i == 0 {
                        if let Some(impl_type) = &self.current_impl_type {
                            self.env.insert(param.name.clone(), impl_type.clone());
                            arg_types.push(impl_type.clone());
                        } else {
                            let t = self.convert_to_internal_type(&param.type_annotation);
                            self.env.insert(param.name.clone(), t.clone());
                            arg_types.push(t);
                        }
                    } else {
                        let t = self.convert_to_internal_type(&param.type_annotation);
                        self.env.insert(param.name.clone(), t.clone());
                        arg_types.push(t);
                    }
                }
                let ret_type = return_type
                    .as_ref()
                    .map(|rt| self.convert_to_internal_type(rt))
                    .unwrap_or_else(|| self.fresh_var());
                let fun_type = typed_ast::Type::Fun {
                    args: arg_types,
                    ret: Box::new(ret_type.clone()),
                };
                self.unify(&typ, &fun_type);
                let typed_body = body
                    .iter()
                    .map(|e| self.typecheck_expr(e))
                    .collect::<Result<Vec<_>, _>>()?;

                self.generic_mapping = old_generic_mapping;

                let generic_names: Vec<String> =
                    generic_params.iter().map(|gp| gp.name.clone()).collect();
                self.function_env.insert(
                    name.clone(),
                    FunctionSignature {
                        generic_params: generic_names.clone(),
                        typ: fun_type.clone(),
                    },
                );

                typed_ast::TypedAstNodeKind::Function {
                    name: name.clone(),
                    args: args
                        .iter()
                        .map(|p| typed_ast::TypedFunctionParam {
                            name: p.name.clone(),
                            type_annotation: self.convert_type_annotation(&p.type_annotation),
                            typ: self.env[&p.name].clone(),
                        })
                        .collect(),
                    return_type: return_type
                        .as_ref()
                        .map(|rt| self.convert_type_annotation(rt)),
                    generic_params: self.convert_generic_params(generic_params),
                    body: typed_body,
                }
            }
            ast::AstNodeKind::Struct {
                name,
                generic_params,
                fields,
            } => {
                let generic_map = self.process_generic_params(generic_params)?;
                let old_generic_mapping = std::mem::replace(&mut self.generic_mapping, generic_map);

                let field_types = fields
                    .iter()
                    .map(|f| {
                        (
                            f.name.clone(),
                            self.convert_to_internal_type(&f.type_annotation),
                        )
                    })
                    .collect();
                self.struct_env.insert(name.clone(), field_types);
                let con_type = typed_ast::Type::Con {
                    name: name.clone(),
                    args: self.generic_mapping.values().cloned().collect(),
                };
                self.unify(&typ, &con_type);

                self.generic_mapping = old_generic_mapping;
                typed_ast::TypedAstNodeKind::Struct {
                    name: name.clone(),
                    generic_params: self.convert_generic_params(generic_params),
                    fields: fields
                        .iter()
                        .map(|f| typed_ast::TypedStructField {
                            name: f.name.clone(),
                            type_annotation: self.convert_type_annotation(&f.type_annotation),
                            meta: f.meta.clone(),
                            typ: self.convert_to_internal_type(&f.type_annotation),
                        })
                        .collect(),
                }
            }
            ast::AstNodeKind::Trait {
                name,
                generic_params,
                methods,
            } => {
                let generic_map = self.process_generic_params(generic_params)?;
                let old_generic_mapping = std::mem::replace(&mut self.generic_mapping, generic_map);

                let self_type = self.fresh_var();
                self.generic_mapping
                    .insert("Self".to_string(), self_type.clone());

                let mut method_types = HashMap::new();
                let typed_methods = methods
                    .iter()
                    .map(|m| {
                        let method_generic_map = if m.generic_params.is_empty() {
                            HashMap::new()
                        } else {
                            self.process_method_generic_params(&m.generic_params)
                                .unwrap_or_else(|_| HashMap::new())
                        };
                        let old_mapping =
                            std::mem::replace(&mut self.generic_mapping, method_generic_map);

                        let mut arg_types = vec![];
                        let typed_args = m
                            .args
                            .iter()
                            .map(|a| match a {
                                ast::MethodParam::SelfParam => {
                                    let param_self_type = self_type.clone();
                                    arg_types.push(param_self_type.clone());
                                    typed_ast::TypedMethodParam::SelfParam {
                                        typ: param_self_type,
                                    }
                                }
                                ast::MethodParam::TypedParam {
                                    name,
                                    type_annotation,
                                } => {
                                    let param_type = self.convert_to_internal_type(type_annotation);
                                    arg_types.push(param_type.clone());
                                    typed_ast::TypedMethodParam::TypedParam {
                                        name: name.clone(),
                                        type_annotation: self
                                            .convert_type_annotation(type_annotation),
                                        typ: param_type,
                                    }
                                }
                            })
                            .collect();
                        let ret_type = m
                            .return_type
                            .as_ref()
                            .map(|rt| self.convert_to_internal_type(rt))
                            .unwrap_or_else(|| self.fresh_var());
                        let method_type = typed_ast::Type::Fun {
                            args: arg_types,
                            ret: Box::new(ret_type.clone()),
                        };
                        method_types.insert(m.name.clone(), method_type.clone());
                        let typed_signature = typed_ast::TypedMethodSignature {
                            name: m.name.clone(),
                            args: typed_args,
                            return_type: m
                                .return_type
                                .as_ref()
                                .map(|rt| self.convert_type_annotation(rt)),
                            generic_params: self.convert_generic_params(&m.generic_params),
                            meta: m.meta.clone(),
                            typ: method_type,
                        };

                        self.generic_mapping = old_mapping;
                        typed_signature
                    })
                    .collect();
                self.trait_env.insert(name.clone(), method_types);

                self.generic_mapping = old_generic_mapping;

                let typ = self.fresh_var();
                typed_ast::TypedAstNodeKind::Trait {
                    name: name.clone(),
                    generic_params: self.convert_generic_params(generic_params),
                    methods: typed_methods,
                }
            }
            ast::AstNodeKind::Enum {
                name,
                generic_params,
                variants,
            } => {
                let generic_map = self.process_generic_params(generic_params)?;
                let old_generic_mapping = std::mem::replace(&mut self.generic_mapping, generic_map);

                let variant_names = variants
                    .iter()
                    .map(|v| match v {
                        ast::EnumVariant::Tuple { name, .. } => name.clone(),
                        ast::EnumVariant::Struct { name, .. } => name.clone(),
                    })
                    .collect();
                self.enum_env.insert(name.clone(), variant_names);
                let con_type = typed_ast::Type::Con {
                    name: name.clone(),
                    args: self.generic_mapping.values().cloned().collect(),
                };
                self.unify(&typ, &con_type);

                let typed_variants = variants
                    .iter()
                    .map(|v| match v {
                        ast::EnumVariant::Tuple { name, fields, meta } => {
                            typed_ast::TypedEnumVariant::Tuple {
                                name: name.clone(),
                                fields: fields
                                    .iter()
                                    .map(|f| self.convert_type_annotation(f))
                                    .collect(),
                                meta: meta.clone(),
                                typ: self.fresh_var(),
                            }
                        }
                        ast::EnumVariant::Struct { name, fields, meta } => {
                            typed_ast::TypedEnumVariant::Struct {
                                name: name.clone(),
                                fields: fields
                                    .iter()
                                    .map(|f| typed_ast::TypedStructField {
                                        name: f.name.clone(),
                                        type_annotation: self
                                            .convert_type_annotation(&f.type_annotation),
                                        meta: f.meta.clone(),
                                        typ: self.convert_to_internal_type(&f.type_annotation),
                                    })
                                    .collect(),
                                meta: meta.clone(),
                                typ: self.fresh_var(),
                            }
                        }
                    })
                    .collect();

                self.generic_mapping = old_generic_mapping;

                typed_ast::TypedAstNodeKind::Enum {
                    name: name.clone(),
                    generic_params: self.convert_generic_params(generic_params),
                    variants: typed_variants,
                }
            }
            ast::AstNodeKind::Impl {
                trait_name,
                for_type,
                methods,
            } => {
                if let Some(trait_name) = trait_name {
                    if !self.trait_env.contains_key(trait_name) {
                        return Err(format!("trait {} not found", trait_name));
                    }
                }

                let impl_type = self.convert_to_internal_type(for_type);

                let for_type_key = match &for_type.kind {
                    ast::TypeAnnotationKind::Constructor { name, generic_args } => {
                        if generic_args.is_empty() {
                            name.clone()
                        } else {
                            let args_str = generic_args
                                .iter()
                                .map(|arg| match &arg.kind {
                                    ast::TypeAnnotationKind::Constructor { name, .. } => {
                                        name.clone()
                                    }
                                    _ => "_".to_string(),
                                })
                                .collect::<Vec<_>>()
                                .join(",");
                            format!("{}<{}>", name, args_str)
                        }
                    }
                    _ => return Err("impl for_type must be a constructor".to_string()),
                };

                let old_impl_type =
                    std::mem::replace(&mut self.current_impl_type, Some(impl_type.clone()));

                let mut impl_method_types = HashMap::new();
                let typed_methods = methods
                    .iter()
                    .map(|m| -> Result<typed_ast::TypedAstNode, String> {
                        let typed_method = self.typecheck_node(m)?;
                        if let typed_ast::TypedAstNodeKind::Function { name, .. } =
                            &typed_method.kind
                        {
                            impl_method_types.insert(name.clone(), typed_method.typ.clone());
                        }
                        Ok(typed_method)
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                self.current_impl_type = old_impl_type;

                let trait_key = trait_name.as_deref().unwrap_or("");
                if !self.impl_env.contains_key(trait_key) {
                    self.impl_env.insert(trait_key.to_string(), HashMap::new());
                }
                self.impl_env
                    .get_mut(trait_key)
                    .unwrap()
                    .insert(for_type_key.clone(), impl_method_types);

                let typ = self.fresh_var();
                typed_ast::TypedAstNodeKind::Impl {
                    trait_name: trait_name.clone(),
                    for_type: self.convert_type_annotation(for_type),
                    methods: typed_methods,
                }
            }
            ast::AstNodeKind::Extern {
                name,
                args,
                return_type,
            } => {
                let arg_types = args
                    .iter()
                    .map(|p| self.convert_to_internal_type(&p.type_annotation))
                    .collect();
                let ret_type = return_type
                    .as_ref()
                    .map(|rt| self.convert_to_internal_type(rt))
                    .unwrap_or(typed_ast::Type::Con {
                        name: "void".to_string(),
                        args: vec![],
                    });
                let typ = typed_ast::Type::Fun {
                    args: arg_types,
                    ret: Box::new(ret_type),
                };
                typed_ast::TypedAstNodeKind::Extern {
                    name: name.clone(),
                    args: args
                        .iter()
                        .map(|p| typed_ast::TypedFunctionParam {
                            name: p.name.clone(),
                            type_annotation: self.convert_type_annotation(&p.type_annotation),
                            typ: self.convert_to_internal_type(&p.type_annotation),
                        })
                        .collect(),
                    return_type: return_type
                        .as_ref()
                        .map(|rt| self.convert_type_annotation(rt)),
                }
            }
            ast::AstNodeKind::Const {
                name,
                type_annotation,
                value,
            } => {
                let typed_value = self.typecheck_expr(value)?;
                if let Some(ta) = type_annotation {
                    let expected_typ = self.convert_to_internal_type(ta);
                    self.unify(&typed_value.typ, &expected_typ);
                }
                let typ = typed_value.typ.clone();
                typed_ast::TypedAstNodeKind::Const {
                    name: name.clone(),
                    type_annotation: type_annotation
                        .as_ref()
                        .map(|ta| self.convert_type_annotation(ta)),
                    value: typed_value,
                }
            }
            ast::AstNodeKind::Import { path, items } => {
                let typ = self.fresh_var();
                typed_ast::TypedAstNodeKind::Import {
                    path: path.clone(),
                    items: items
                        .iter()
                        .map(|item| typed_ast::ImportItem {
                            name: item.name.clone(),
                            alias: item.alias.clone(),
                        })
                        .collect(),
                }
            }
        };
        Ok(typed_ast::TypedAstNode {
            meta: node.meta.clone(),
            kind,
            typ,
        })
    }

    fn typecheck_expr(&mut self, expr: &ast::Expr) -> Result<typed_ast::TypedExpr, String> {
        let typ = self.fresh_var();
        let kind = match &expr.kind {
            ast::ExprKind::Literal(lit) => {
                let lit_typ = self.infer_literal(lit);
                self.unify(&typ, &lit_typ);
                typed_ast::TypedExprKind::Literal(match lit {
                    ast::Literal::Integer(i) => typed_ast::TypedLiteral::Integer(*i),
                    ast::Literal::Float(f) => typed_ast::TypedLiteral::Float(*f),
                    ast::Literal::String(s) => typed_ast::TypedLiteral::String(s.clone()),
                    ast::Literal::Boolean(b) => typed_ast::TypedLiteral::Boolean(*b),
                })
            }
            ast::ExprKind::Variable(name) => {
                let var_typ = if let Some(sig) = self.function_env.get(name) {
                    let sig = sig.clone();
                    let old_mapping = std::mem::replace(&mut self.generic_mapping, HashMap::new());
                    let generic_vars: Vec<(String, typed_ast::Type)> = sig
                        .generic_params
                        .iter()
                        .map(|name| (name.clone(), self.fresh_var()))
                        .collect();
                    for (gen_name, var) in generic_vars {
                        self.generic_mapping.insert(gen_name, var);
                    }
                    let instantiated_type = self.convert_type_with_mapping(&sig.typ);
                    self.generic_mapping = old_mapping;
                    instantiated_type
                } else {
                    self.env.get(name).cloned().ok_or("unbound variable")?
                };
                self.unify(&typ, &var_typ);
                typed_ast::TypedExprKind::Variable(name.clone())
            }
            ast::ExprKind::Array(elements) => {
                let mut elem_type = None;
                let typed_elements = elements
                    .iter()
                    .map(|el| match el {
                        ast::ArrayElement::Value(e) => {
                            let typed = self.typecheck_expr(e)?;
                            if let Some(et) = &elem_type {
                                self.unify(et, &typed.typ);
                            } else {
                                elem_type = Some(typed.typ.clone());
                            }
                            Ok(typed_ast::TypedArrayElement::Value(typed))
                        }
                        ast::ArrayElement::Spread(e) => {
                            let typed = self.typecheck_expr(e)?;
                            Ok(typed_ast::TypedArrayElement::Spread(typed))
                        }
                    })
                    .collect::<Result<Vec<typed_ast::TypedArrayElement>, String>>()?;
                let elem_typ = elem_type.unwrap_or_else(|| self.fresh_var());
                let array_typ = typed_ast::Type::Array {
                    elem: Box::new(elem_typ.clone()),
                    size: None,
                };
                self.unify(&typ, &array_typ);
                typed_ast::TypedExprKind::Array(typed_elements)
            }
            ast::ExprKind::Tuple(elements) => {
                let typed_elements = elements
                    .iter()
                    .map(|e| self.typecheck_expr(e))
                    .collect::<Result<Vec<_>, _>>()?;
                let tuple_typ =
                    typed_ast::Type::Tuple(typed_elements.iter().map(|e| e.typ.clone()).collect());
                self.unify(&typ, &tuple_typ);
                typed_ast::TypedExprKind::Tuple(typed_elements)
            }
            ast::ExprKind::Row(fields) => {
                let mut field_types = vec![];
                let typed_fields = fields
                    .iter()
                    .map(|f| match f {
                        ast::RowField::Field { name, value } => {
                            let typed_value = self.typecheck_expr(value)?;
                            field_types.push((name.clone(), typed_value.typ.clone()));
                            Ok(typed_ast::TypedRowField::Field {
                                name: name.clone(),
                                value: typed_value,
                            })
                        }
                        ast::RowField::Spread(e) => {
                            let typed_value = self.typecheck_expr(e)?;
                            Ok(typed_ast::TypedRowField::Spread(typed_value))
                        }
                    })
                    .collect::<Result<Vec<typed_ast::TypedRowField>, String>>()?;
                let row_typ = typed_ast::Type::Row(field_types);
                self.unify(&typ, &row_typ);
                typed_ast::TypedExprKind::Row(typed_fields)
            }
            ast::ExprKind::Let {
                name,
                type_annotation,
                value,
            } => {
                let typed_value = self.typecheck_expr(value)?;
                if let Some(ta) = type_annotation {
                    let expected_typ = self.convert_to_internal_type(ta);
                    self.unify(&typed_value.typ, &expected_typ);
                }
                self.env.insert(name.clone(), typed_value.typ.clone());
                let typ = typed_ast::Type::Con {
                    name: "void".to_string(),
                    args: vec![],
                };
                typed_ast::TypedExprKind::Let {
                    name: name.clone(),
                    type_annotation: type_annotation
                        .as_ref()
                        .map(|ta| self.convert_type_annotation(ta)),
                    value: Box::new(typed_value),
                }
            }
            ast::ExprKind::Assign { target, value } => {
                let typed_target = self.typecheck_expr(target)?;
                let typed_value = self.typecheck_expr(value)?;
                self.unify(&typed_target.typ, &typed_value.typ);
                let assign_typ = typed_ast::Type::Con {
                    name: "void".to_string(),
                    args: vec![],
                };
                self.unify(&typ, &assign_typ);
                typed_ast::TypedExprKind::Assign {
                    target: Box::new(typed_target),
                    value: Box::new(typed_value),
                }
            }
            ast::ExprKind::UnaryOp { operator, operand } => {
                let typed_operand = self.typecheck_expr(operand)?;
                let op_typ = typed_operand.typ.clone();
                self.unify(&typ, &op_typ);
                typed_ast::TypedExprKind::UnaryOp {
                    operator: operator.clone(),
                    operand: Box::new(typed_operand),
                }
            }
            ast::ExprKind::Cast(expr, type_annotation) => {
                let typed_expr = self.typecheck_expr(expr)?;
                let target_typ = self.convert_to_internal_type(type_annotation);
                self.unify(&typ, &target_typ);
                typed_ast::TypedExprKind::Cast(
                    Box::new(typed_expr),
                    self.convert_type_annotation(type_annotation),
                )
            }
            ast::ExprKind::EarlyReturn(expr) => {
                let typed_expr = self.typecheck_expr(expr)?;
                typed_ast::TypedExprKind::EarlyReturn(Box::new(typed_expr))
            }
            ast::ExprKind::DotAccess { value, field } => {
                let typed_value = self.typecheck_expr(value)?;
                let field_typ = match &typed_value.typ {
                    typed_ast::Type::Con { name, .. } => {
                        let mut method_type = None;

                        for (_trait_name, type_impls) in &self.impl_env {
                            if let Some(methods) = type_impls.get(name) {
                                if let Some(mt) = methods.get(field) {
                                    method_type = Some(mt.clone());
                                    break;
                                }
                            }
                        }

                        if let Some(mt) = method_type {
                            mt
                        } else if let Some(fields) = self.struct_env.get(name) {
                            fields
                                .iter()
                                .find(|(n, _)| n == field)
                                .map(|(_, t)| t.clone())
                                .unwrap_or_else(|| self.fresh_var())
                        } else {
                            self.fresh_var()
                        }
                    }
                    _ => self.fresh_var(),
                };
                self.unify(&typ, &field_typ);
                typed_ast::TypedExprKind::DotAccess {
                    value: Box::new(typed_value),
                    field: field.clone(),
                }
            }
            ast::ExprKind::Index { value, index } => {
                let typed_value = self.typecheck_expr(value)?;
                let typed_index = self.typecheck_expr(index)?;
                let index_typ = typed_ast::Type::Con {
                    name: "i32".to_string(),
                    args: vec![],
                };
                self.unify(&typed_index.typ, &index_typ);
                let elem_typ = match &typed_value.typ {
                    typed_ast::Type::Array { elem, .. } => (**elem).clone(),
                    _ => self.fresh_var(),
                };
                self.unify(&typ, &elem_typ);
                typed_ast::TypedExprKind::Index {
                    value: Box::new(typed_value),
                    index: Box::new(typed_index),
                }
            }
            ast::ExprKind::BinaryOp {
                left,
                operator,
                right,
            } => {
                let left_typ = self.typecheck_expr(left)?;
                let right_typ = self.typecheck_expr(right)?;
                let is_comparison =
                    matches!(operator.as_str(), ">" | "<" | ">=" | "<=" | "==" | "!=");
                if is_comparison {
                    let op_typ = self.fresh_var();
                    self.unify(&left_typ.typ, &op_typ);
                    self.unify(&right_typ.typ, &op_typ);
                    let bool_typ = typed_ast::Type::Con {
                        name: "bool".to_string(),
                        args: vec![],
                    };
                    self.unify(&typ, &bool_typ);
                } else {
                    let op_typ = self.fresh_var();
                    self.unify(&left_typ.typ, &op_typ);
                    self.unify(&right_typ.typ, &op_typ);
                    self.unify(&typ, &op_typ);
                }
                typed_ast::TypedExprKind::BinaryOp {
                    left: Box::new(left_typ),
                    operator: operator.clone(),
                    right: Box::new(right_typ),
                }
            }
            ast::ExprKind::FunctionCall { function, args } => {
                let typed_function = self.typecheck_expr(function)?;
                let typed_args = args
                    .iter()
                    .map(|a| self.typecheck_expr(a))
                    .collect::<Result<Vec<_>, _>>()?;
                let arg_types: Vec<typed_ast::Type> =
                    typed_args.iter().map(|a| a.typ.clone()).collect();
                let ret_typ = self.fresh_var();
                let fun_typ = typed_ast::Type::Fun {
                    args: arg_types,
                    ret: Box::new(ret_typ.clone()),
                };

                if let ast::ExprKind::Variable(func_name) = &function.kind {
                    if let Some(sig) = self.function_env.get(func_name) {
                        let sig = sig.clone();
                        let old_mapping =
                            std::mem::replace(&mut self.generic_mapping, HashMap::new());
                        let generic_vars: Vec<(String, typed_ast::Type)> = sig
                            .generic_params
                            .iter()
                            .map(|name| (name.clone(), self.fresh_var()))
                            .collect();
                        for (gen_name, var) in generic_vars {
                            self.generic_mapping.insert(gen_name, var);
                        }
                        let instantiated_type = self.convert_type_with_mapping(&sig.typ);
                        self.generic_mapping = old_mapping;
                        self.unify(&typed_function.typ, &instantiated_type);
                    } else {
                        self.unify(&typed_function.typ, &fun_typ);
                    }
                } else {
                    self.unify(&typed_function.typ, &fun_typ);
                }
                self.unify(&typ, &ret_typ);
                typed_ast::TypedExprKind::FunctionCall {
                    function: Box::new(typed_function),
                    args: typed_args,
                }
            }
            ast::ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let typed_condition = self.typecheck_expr(condition)?;
                let bool_typ = typed_ast::Type::Con {
                    name: "bool".to_string(),
                    args: vec![],
                };
                self.unify(&typed_condition.typ, &bool_typ);
                let typed_then = then_branch
                    .iter()
                    .map(|e| self.typecheck_expr(e))
                    .collect::<Result<Vec<_>, _>>()?;
                let typed_else = else_branch
                    .as_ref()
                    .map(|branch| {
                        branch
                            .iter()
                            .map(|e| self.typecheck_expr(e))
                            .collect::<Result<Vec<_>, _>>()
                    })
                    .transpose()?;
                let branch_typ = if let Some(else_branch) = &typed_else {
                    if let (Some(last_then), Some(last_else)) =
                        (typed_then.last(), else_branch.last())
                    {
                        self.unify(&last_then.typ, &last_else.typ);
                        last_then.typ.clone()
                    } else {
                        self.fresh_var()
                    }
                } else {
                    typed_ast::Type::Con {
                        name: "void".to_string(),
                        args: vec![],
                    }
                };
                self.unify(&typ, &branch_typ);
                typed_ast::TypedExprKind::If {
                    condition: Box::new(typed_condition),
                    then_branch: typed_then,
                    else_branch: typed_else,
                }
            }
            ast::ExprKind::While { condition, body } => {
                let typed_condition = self.typecheck_expr(condition)?;
                let bool_typ = typed_ast::Type::Con {
                    name: "bool".to_string(),
                    args: vec![],
                };
                self.unify(&typed_condition.typ, &bool_typ);
                let typed_body = body
                    .iter()
                    .map(|e| self.typecheck_expr(e))
                    .collect::<Result<Vec<_>, _>>()?;
                let while_typ = typed_ast::Type::Con {
                    name: "void".to_string(),
                    args: vec![],
                };
                self.unify(&typ, &while_typ);
                typed_ast::TypedExprKind::While {
                    condition: Box::new(typed_condition),
                    body: typed_body,
                }
            }
            ast::ExprKind::For {
                iterator,
                iterable,
                body,
            } => {
                let typed_iterable = self.typecheck_expr(iterable)?;
                let elem_typ = self.fresh_var();
                self.env.insert(iterator.clone(), elem_typ.clone());
                let typed_body = body
                    .iter()
                    .map(|e| self.typecheck_expr(e))
                    .collect::<Result<Vec<_>, _>>()?;
                let for_typ = typed_ast::Type::Con {
                    name: "void".to_string(),
                    args: vec![],
                };
                self.unify(&typ, &for_typ);
                typed_ast::TypedExprKind::For {
                    iterator: iterator.clone(),
                    iterable: Box::new(typed_iterable),
                    body: typed_body,
                }
            }
            ast::ExprKind::Range {
                start,
                end,
                inclusive,
            } => {
                let typed_start = self.typecheck_expr(start)?;
                let typed_end = self.typecheck_expr(end)?;
                self.unify(&typed_start.typ, &typed_end.typ);
                let range_typ = typed_ast::Type::Con {
                    name: "Range".to_string(),
                    args: vec![typed_start.typ.clone()],
                };
                self.unify(&typ, &range_typ);
                typed_ast::TypedExprKind::Range {
                    start: Box::new(typed_start),
                    end: Box::new(typed_end),
                    inclusive: *inclusive,
                }
            }
            ast::ExprKind::Break => {
                let break_typ = typed_ast::Type::Con {
                    name: "void".to_string(),
                    args: vec![],
                };
                self.unify(&typ, &break_typ);
                typed_ast::TypedExprKind::Break
            }
            ast::ExprKind::Continue => {
                let continue_typ = typed_ast::Type::Con {
                    name: "void".to_string(),
                    args: vec![],
                };
                self.unify(&typ, &continue_typ);
                typed_ast::TypedExprKind::Continue
            }
            ast::ExprKind::Return(expr) => {
                let typed_expr = expr.as_ref().map(|e| self.typecheck_expr(e)).transpose()?;
                let return_typ = typed_expr
                    .as_ref()
                    .map(|e| e.typ.clone())
                    .unwrap_or_else(|| typed_ast::Type::Con {
                        name: "void".to_string(),
                        args: vec![],
                    });
                self.unify(&typ, &return_typ);
                typed_ast::TypedExprKind::Return(typed_expr.map(Box::new))
            }
            ast::ExprKind::Block { expressions } => {
                let mut old_bindings = vec![];
                for e in expressions {
                    if let ast::ExprKind::Let { name, .. } = &e.kind {
                        if let Some(old) = self.env.get(name).cloned() {
                            old_bindings.push((name.clone(), old));
                        }
                    }
                }
                let typed_expressions = expressions
                    .iter()
                    .map(|e| self.typecheck_expr(e))
                    .collect::<Result<Vec<_>, _>>()?;
                for (name, old_typ) in old_bindings {
                    self.env.insert(name, old_typ);
                }
                let block_typ = typed_expressions
                    .last()
                    .map(|e| e.typ.clone())
                    .unwrap_or_else(|| typed_ast::Type::Con {
                        name: "void".to_string(),
                        args: vec![],
                    });
                self.unify(&typ, &block_typ);
                typed_ast::TypedExprKind::Block {
                    expressions: typed_expressions,
                }
            }
            ast::ExprKind::Do(expressions) => {
                let typed_expressions = expressions
                    .iter()
                    .map(|e| self.typecheck_expr(e))
                    .collect::<Result<Vec<_>, _>>()?;
                let do_typ = typed_ast::Type::Con {
                    name: "void".to_string(),
                    args: vec![],
                };
                self.unify(&typ, &do_typ);
                typed_ast::TypedExprKind::Do(typed_expressions)
            }
            ast::ExprKind::Lambda {
                params,
                return_type,
                body,
            } => {
                let mut old_bindings = vec![];
                let mut arg_types = vec![];
                for param in params {
                    let t = self.convert_to_internal_type(&param.type_annotation);
                    self.env.insert(param.name.clone(), t.clone());
                    old_bindings.push(param.name.clone());
                    arg_types.push(t);
                }
                let typed_body = self.typecheck_expr(body)?;
                let ret_typ = return_type
                    .as_ref()
                    .map(|rt| self.convert_to_internal_type(rt))
                    .unwrap_or_else(|| typed_body.typ.clone());
                for binding in old_bindings {
                    self.env.remove(&binding);
                }
                let lambda_typ = typed_ast::Type::Fun {
                    args: arg_types,
                    ret: Box::new(ret_typ),
                };
                self.unify(&typ, &lambda_typ);
                typed_ast::TypedExprKind::Lambda {
                    params: params
                        .iter()
                        .map(|p| typed_ast::TypedFunctionParam {
                            name: p.name.clone(),
                            type_annotation: self.convert_type_annotation(&p.type_annotation),
                            typ: self.convert_to_internal_type(&p.type_annotation),
                        })
                        .collect(),
                    return_type: return_type
                        .as_ref()
                        .map(|rt| self.convert_type_annotation(rt)),
                    body: Box::new(typed_body),
                }
            }
            ast::ExprKind::Match { value, arms } => {
                let typed_value = self.typecheck_expr(value)?;
                let mut arm_types = vec![];
                let typed_arms = arms
                    .iter()
                    .map(|arm| -> Result<typed_ast::TypedMatchArm, String> {
                        let typed_guard = arm
                            .guard
                            .as_ref()
                            .map(|g| self.typecheck_expr(g))
                            .transpose()?;
                        self.typecheck_pattern(&arm.pattern)?;
                        let typed_body = arm
                            .body
                            .iter()
                            .map(|e| self.typecheck_expr(e))
                            .collect::<Result<Vec<typed_ast::TypedExpr>, String>>()?;
                        let arm_typ =
                            typed_body.last().map(|e| e.typ.clone()).unwrap_or_else(|| {
                                typed_ast::Type::Con {
                                    name: "void".to_string(),
                                    args: vec![],
                                }
                            });
                        arm_types.push(arm_typ.clone());
                        Ok(typed_ast::TypedMatchArm {
                            pattern: self.convert_pattern(&arm.pattern),
                            guard: typed_guard.map(Box::new),
                            body: typed_body,
                            typ: arm_typ,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let match_typ = arm_types
                    .first()
                    .cloned()
                    .unwrap_or_else(|| self.fresh_var());
                for arm_typ in &arm_types {
                    self.unify(&match_typ, arm_typ);
                }
                self.unify(&typ, &match_typ);
                typed_ast::TypedExprKind::Match {
                    value: Box::new(typed_value),
                    arms: typed_arms,
                }
            }
            ast::ExprKind::StructInit { name, fields } => {
                let typed_fields = fields
                    .iter()
                    .map(|f| match f {
                        ast::StructInitField::Field { name, value } => {
                            let typed_value = self.typecheck_expr(value)?;
                            Ok(typed_ast::TypedStructInitField::Field {
                                name: name.clone(),
                                value: typed_value,
                            })
                        }
                        ast::StructInitField::Spread(e) => {
                            let typed_value = self.typecheck_expr(e)?;
                            Ok(typed_ast::TypedStructInitField::Spread(typed_value))
                        }
                    })
                    .collect::<Result<Vec<typed_ast::TypedStructInitField>, String>>()?;
                let struct_typ = typed_ast::Type::Con {
                    name: name.clone(),
                    args: vec![],
                };
                self.unify(&typ, &struct_typ);
                typed_ast::TypedExprKind::StructInit {
                    name: name.clone(),
                    fields: typed_fields,
                }
            }
            ast::ExprKind::EnumInit {
                enum_name,
                variant_name,
                fields,
            } => {
                let typed_fields = match fields {
                    ast::EnumInitFields::Tuple(fields) => {
                        let typed = fields
                            .iter()
                            .map(|f| self.typecheck_expr(f))
                            .collect::<Result<Vec<typed_ast::TypedExpr>, String>>()?;
                        typed_ast::TypedEnumInitFields::Tuple(typed)
                    }
                    ast::EnumInitFields::Struct(fields) => {
                        let typed = fields
                            .iter()
                            .map(|f| match f {
                                ast::StructInitField::Field { name, value } => {
                                    let typed_value = self.typecheck_expr(value)?;
                                    Ok(typed_ast::TypedStructInitField::Field {
                                        name: name.clone(),
                                        value: typed_value,
                                    })
                                }
                                ast::StructInitField::Spread(e) => {
                                    let typed_value = self.typecheck_expr(e)?;
                                    Ok(typed_ast::TypedStructInitField::Spread(typed_value))
                                }
                            })
                            .collect::<Result<Vec<typed_ast::TypedStructInitField>, String>>()?;
                        typed_ast::TypedEnumInitFields::Struct(typed)
                    }
                };
                let enum_typ = typed_ast::Type::Con {
                    name: enum_name.clone(),
                    args: vec![],
                };
                self.unify(&typ, &enum_typ);
                typed_ast::TypedExprKind::EnumInit {
                    enum_name: enum_name.clone(),
                    variant_name: variant_name.clone(),
                    fields: typed_fields,
                }
            }
        };
        let final_typ = self.substitution.apply(&typ);
        let result = typed_ast::TypedExpr {
            meta: expr.meta.clone(),
            kind,
            typ: final_typ,
        };
        Ok(result)
    }

    fn infer_literal(&mut self, lit: &ast::Literal) -> typed_ast::Type {
        match lit {
            ast::Literal::Integer(_) => typed_ast::Type::Con {
                name: "i32".to_string(),
                args: vec![],
            },
            ast::Literal::Float(_) => typed_ast::Type::Con {
                name: "f32".to_string(),
                args: vec![],
            },
            ast::Literal::String(_) => typed_ast::Type::Con {
                name: "String".to_string(),
                args: vec![],
            },
            ast::Literal::Boolean(_) => typed_ast::Type::Con {
                name: "bool".to_string(),
                args: vec![],
            },
        }
    }

    fn convert_to_internal_type(&mut self, ta: &ast::TypeAnnotation) -> typed_ast::Type {
        match &ta.kind {
            ast::TypeAnnotationKind::Constructor { name, generic_args } => {
                if name == "Self" {
                    if let Some(impl_type) = &self.current_impl_type {
                        impl_type.clone()
                    } else {
                        self.fresh_var()
                    }
                } else if let Some(t) = self.generic_mapping.get(name) {
                    t.clone()
                } else {
                    typed_ast::Type::Con {
                        name: name.clone(),
                        args: generic_args
                            .iter()
                            .map(|arg| self.convert_to_internal_type(arg))
                            .collect(),
                    }
                }
            }
            ast::TypeAnnotationKind::Tuple(ts) => typed_ast::Type::Tuple(
                ts.iter()
                    .map(|t| self.convert_to_internal_type(t))
                    .collect(),
            ),
            ast::TypeAnnotationKind::Function { args, return_type } => typed_ast::Type::Fun {
                args: args
                    .iter()
                    .map(|a| self.convert_to_internal_type(a))
                    .collect(),
                ret: Box::new(self.convert_to_internal_type(return_type)),
            },
            ast::TypeAnnotationKind::Array { element_type, size } => typed_ast::Type::Array {
                elem: Box::new(self.convert_to_internal_type(element_type)),
                size: *size,
            },
            ast::TypeAnnotationKind::Row(fields) => typed_ast::Type::Row(
                fields
                    .iter()
                    .map(|f| {
                        (
                            f.name.clone(),
                            self.convert_to_internal_type(&f.type_annotation),
                        )
                    })
                    .collect(),
            ),
        }
    }

    fn convert_type_annotation(&self, ta: &ast::TypeAnnotation) -> typed_ast::TypeAnnotation {
        typed_ast::TypeAnnotation {
            meta: ta.meta.clone(),
            kind: match &ta.kind {
                ast::TypeAnnotationKind::Constructor { name, generic_args } => {
                    typed_ast::TypeAnnotationKind::Constructor {
                        name: name.clone(),
                        generic_args: generic_args
                            .iter()
                            .map(|arg| self.convert_type_annotation(arg))
                            .collect(),
                    }
                }
                ast::TypeAnnotationKind::Tuple(ts) => typed_ast::TypeAnnotationKind::Tuple(
                    ts.iter().map(|t| self.convert_type_annotation(t)).collect(),
                ),
                ast::TypeAnnotationKind::Function { args, return_type } => {
                    typed_ast::TypeAnnotationKind::Function {
                        args: args
                            .iter()
                            .map(|a| self.convert_type_annotation(a))
                            .collect(),
                        return_type: Box::new(self.convert_type_annotation(return_type)),
                    }
                }
                ast::TypeAnnotationKind::Array { element_type, size } => {
                    typed_ast::TypeAnnotationKind::Array {
                        element_type: Box::new(self.convert_type_annotation(element_type)),
                        size: *size,
                    }
                }
                ast::TypeAnnotationKind::Row(fields) => typed_ast::TypeAnnotationKind::Row(
                    fields
                        .iter()
                        .map(|f| typed_ast::RowTypeField {
                            name: f.name.clone(),
                            type_annotation: self.convert_type_annotation(&f.type_annotation),
                        })
                        .collect(),
                ),
            },
        }
    }

    fn convert_generic_params(&self, gps: &Vec<ast::GenericParam>) -> Vec<typed_ast::GenericParam> {
        gps.iter()
            .map(|gp| self.convert_generic_param(gp))
            .collect()
    }

    fn convert_generic_param(&self, gp: &ast::GenericParam) -> typed_ast::GenericParam {
        typed_ast::GenericParam {
            name: gp.name.clone(),
            constraints: gp
                .constraints
                .iter()
                .map(|c| self.convert_type_annotation(c))
                .collect(),
        }
    }

    fn convert_type_with_mapping(&self, t: &typed_ast::Type) -> typed_ast::Type {
        match t {
            typed_ast::Type::Var(_) => t.clone(),
            typed_ast::Type::Con { name, args } => {
                if let Some(mapped) = self.generic_mapping.get(name) {
                    mapped.clone()
                } else {
                    typed_ast::Type::Con {
                        name: name.clone(),
                        args: args
                            .iter()
                            .map(|a| self.convert_type_with_mapping(a))
                            .collect(),
                    }
                }
            }
            typed_ast::Type::Tuple(ts) => typed_ast::Type::Tuple(
                ts.iter()
                    .map(|t| self.convert_type_with_mapping(t))
                    .collect(),
            ),
            typed_ast::Type::Fun { args, ret } => typed_ast::Type::Fun {
                args: args
                    .iter()
                    .map(|a| self.convert_type_with_mapping(a))
                    .collect(),
                ret: Box::new(self.convert_type_with_mapping(ret)),
            },
            typed_ast::Type::Array { elem, size } => typed_ast::Type::Array {
                elem: Box::new(self.convert_type_with_mapping(elem)),
                size: *size,
            },
            typed_ast::Type::Row(fields) => typed_ast::Type::Row(
                fields
                    .iter()
                    .map(|(n, t)| (n.clone(), self.convert_type_with_mapping(t)))
                    .collect(),
            ),
            typed_ast::Type::Rec(t) => {
                typed_ast::Type::Rec(Box::new(self.convert_type_with_mapping(t)))
            }
        }
    }

    fn unify(&mut self, t1: &typed_ast::Type, t2: &typed_ast::Type) {
        if let Err(e) = self.substitution.unify(t1, t2) {
            panic!("unification error: {}", e);
        }
    }

    fn solve_constraints(&mut self) -> Result<(), String> {
        for constraint in &self.constraints.clone() {
            match constraint {
                Constraint::Eq(t1, t2) => {
                    self.substitution.unify(&t1, &t2)?;
                }
                Constraint::Class(trait_name, types) => {
                    if let Some(type_map) = self.impl_env.get(trait_name) {
                        let type_key = if types.len() == 1 {
                            if let typed_ast::Type::Con { name, args } = &types[0] {
                                if args.is_empty() {
                                    name.clone()
                                } else {
                                    let args_str = args
                                        .iter()
                                        .map(|a| match a {
                                            typed_ast::Type::Con { name, .. } => name.clone(),
                                            _ => "_".to_string(),
                                        })
                                        .collect::<Vec<_>>()
                                        .join(",");
                                    format!("{}<{}>", name, args_str)
                                }
                            } else {
                                return Err("class constraint on non-con type".to_string());
                            }
                        } else {
                            types
                                .iter()
                                .map(|t| match t {
                                    typed_ast::Type::Con { name, args } => {
                                        if args.is_empty() {
                                            name.clone()
                                        } else {
                                            let args_str = args
                                                .iter()
                                                .map(|a| match a {
                                                    typed_ast::Type::Con { name, .. } => {
                                                        name.clone()
                                                    }
                                                    _ => "_".to_string(),
                                                })
                                                .collect::<Vec<_>>()
                                                .join(",");
                                            format!("{}<{}>", name, args_str)
                                        }
                                    }
                                    _ => "_".to_string(),
                                })
                                .collect::<Vec<_>>()
                                .join(",")
                        };

                        if let Some(methods) = type_map.get(&type_key) {
                            if let Some(trait_methods) = self.trait_env.get(trait_name) {
                                for (method_name, trait_method_type) in trait_methods {
                                    if let Some(impl_method_type) = methods.get(method_name) {
                                        self.substitution
                                            .unify(trait_method_type, impl_method_type)?;
                                    } else {
                                        return Err(format!(
                                            "impl for trait {} for type {} missing method {}",
                                            trait_name, type_key, method_name
                                        ));
                                    }
                                }
                            }
                        } else {
                            return Err(format!(
                                "no impl for trait {} for type {}",
                                trait_name, type_key
                            ));
                        }
                    } else {
                        return Err(format!("trait {} not defined", trait_name));
                    }
                }
            }
        }
        Ok(())
    }

    fn typecheck_pattern(&mut self, pattern: &ast::Pattern) -> Result<(), String> {
        match pattern {
            ast::Pattern::Literal(_) => Ok(()),
            ast::Pattern::Variable(name) => {
                let typ = self.fresh_var();
                self.env.insert(name.clone(), typ);
                Ok(())
            }
            ast::Pattern::Wildcard => Ok(()),
            ast::Pattern::Tuple(patterns) => {
                for p in patterns {
                    self.typecheck_pattern(p)?;
                }
                Ok(())
            }
            ast::Pattern::Struct { name, fields } => {
                for f in fields {
                    match f {
                        ast::StructPatternField::Shorthand(name) => {
                            let typ = self.fresh_var();
                            self.env.insert(name.clone(), typ);
                        }
                        ast::StructPatternField::Typed { name, pattern } => {
                            self.typecheck_pattern(pattern)?;
                        }
                    }
                }
                Ok(())
            }
            ast::Pattern::EnumVariant { name, fields } => {
                for p in fields {
                    self.typecheck_pattern(p)?;
                }
                Ok(())
            }
            ast::Pattern::Or(patterns) => {
                for p in patterns {
                    self.typecheck_pattern(p)?;
                }
                Ok(())
            }
        }
    }

    fn convert_pattern(&self, pattern: &ast::Pattern) -> typed_ast::TypedPattern {
        match pattern {
            ast::Pattern::Literal(lit) => typed_ast::TypedPattern::Literal(match lit {
                ast::Literal::Integer(i) => typed_ast::TypedLiteral::Integer(*i),
                ast::Literal::Float(f) => typed_ast::TypedLiteral::Float(*f),
                ast::Literal::String(s) => typed_ast::TypedLiteral::String(s.clone()),
                ast::Literal::Boolean(b) => typed_ast::TypedLiteral::Boolean(*b),
            }),
            ast::Pattern::Variable(name) => typed_ast::TypedPattern::Variable(name.clone()),
            ast::Pattern::Wildcard => typed_ast::TypedPattern::Wildcard,
            ast::Pattern::Tuple(patterns) => typed_ast::TypedPattern::Tuple(
                patterns.iter().map(|p| self.convert_pattern(p)).collect(),
            ),
            ast::Pattern::Struct { name, fields } => typed_ast::TypedPattern::Struct {
                name: name.clone(),
                fields: fields
                    .iter()
                    .map(|f| match f {
                        ast::StructPatternField::Shorthand(name) => {
                            typed_ast::TypedStructPatternField::Shorthand(name.clone())
                        }
                        ast::StructPatternField::Typed { name, pattern } => {
                            typed_ast::TypedStructPatternField::Typed {
                                name: name.clone(),
                                pattern: self.convert_pattern(pattern),
                            }
                        }
                    })
                    .collect(),
            },
            ast::Pattern::EnumVariant { name, fields } => typed_ast::TypedPattern::EnumVariant {
                name: name.clone(),
                fields: fields.iter().map(|p| self.convert_pattern(p)).collect(),
            },
            ast::Pattern::Or(patterns) => typed_ast::TypedPattern::Or(
                patterns.iter().map(|p| self.convert_pattern(p)).collect(),
            ),
        }
    }
}
