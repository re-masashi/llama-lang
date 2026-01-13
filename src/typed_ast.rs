use crate::meta::Meta;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Var(TypeVar),
    Con {
        name: String,
        args: Vec<Type>,
    },
    Tuple(Vec<Type>),
    Fun {
        args: Vec<Type>,
        ret: Box<Type>,
    },
    Array {
        elem: Box<Type>,
        size: Option<usize>,
    },
    Row(Vec<(String, Type)>),
    Rec(Box<Type>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct TypeVar(pub usize);

pub type TypedProgram = Vec<TypedAstNode>;

#[derive(Debug, Clone)]
pub struct TypedAstNode {
    pub meta: Meta,
    pub kind: TypedAstNodeKind,
    pub typ: Type,
}

#[derive(Debug, Clone)]
pub enum TypedAstNodeKind {
    Function {
        name: String,
        args: Vec<TypedFunctionParam>,
        return_type: Option<TypeAnnotation>,
        generic_params: Vec<GenericParam>,
        body: Vec<TypedExpr>,
    },
    Struct {
        name: String,
        generic_params: Vec<GenericParam>,
        fields: Vec<TypedStructField>,
    },
    Enum {
        name: String,
        generic_params: Vec<GenericParam>,
        variants: Vec<TypedEnumVariant>,
    },
    Trait {
        name: String,
        generic_params: Vec<GenericParam>,
        methods: Vec<TypedMethodSignature>,
    },
    Impl {
        trait_name: Option<String>,
        for_type: TypeAnnotation,
        methods: Vec<TypedAstNode>,
    },
    Extern {
        name: String,
        args: Vec<TypedFunctionParam>,
        return_type: Option<TypeAnnotation>,
    },
    Const {
        name: String,
        type_annotation: Option<TypeAnnotation>,
        value: TypedExpr,
    },
    Import {
        path: String,
        items: Vec<ImportItem>,
    },
}

#[derive(Debug, Clone)]
pub struct ImportItem {
    pub name: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TypedMethodSignature {
    pub name: String,
    pub args: Vec<TypedMethodParam>,
    pub return_type: Option<TypeAnnotation>,
    pub generic_params: Vec<GenericParam>,
    pub meta: Meta,
    pub typ: Type,
}

#[derive(Debug, Clone)]
pub enum TypedMethodParam {
    SelfParam {
        typ: Type,
    },
    TypedParam {
        name: String,
        type_annotation: TypeAnnotation,
        typ: Type,
    },
}

#[derive(Debug, Clone)]
pub struct TypedStructField {
    pub name: String,
    pub type_annotation: TypeAnnotation,
    pub meta: Meta,
    pub typ: Type,
}

#[derive(Debug, Clone)]
pub enum TypedEnumVariant {
    Tuple {
        name: String,
        fields: Vec<TypeAnnotation>,
        meta: Meta,
        typ: Type,
    },
    Struct {
        name: String,
        fields: Vec<TypedStructField>,
        meta: Meta,
        typ: Type,
    },
}

#[derive(Debug, Clone)]
pub struct TypedFunctionParam {
    pub name: String,
    pub type_annotation: TypeAnnotation,
    pub typ: Type,
}

#[derive(Debug, Clone)]
pub struct TypeAnnotation {
    pub meta: Meta,
    pub kind: TypeAnnotationKind,
}

#[derive(Debug, Clone)]
pub enum TypeAnnotationKind {
    Constructor {
        name: String,
        generic_args: Vec<TypeAnnotation>,
    },
    Tuple(Vec<TypeAnnotation>),
    Function {
        args: Vec<TypeAnnotation>,
        return_type: Box<TypeAnnotation>,
    },
    Array {
        element_type: Box<TypeAnnotation>,
        size: Option<usize>,
    },
    Row(Vec<RowTypeField>),
}

#[derive(Debug, Clone)]
pub struct GenericParam {
    pub name: String,
    pub constraints: Vec<TypeAnnotation>,
}

#[derive(Debug, Clone)]
pub struct TypedExpr {
    pub meta: Meta,
    pub kind: TypedExprKind,
    pub typ: Type,
}

#[derive(Debug, Clone)]
pub enum TypedExprKind {
    Literal(TypedLiteral),
    Array(Vec<TypedArrayElement>),
    Tuple(Vec<TypedExpr>),
    Row(Vec<TypedRowField>),
    Variable(String),
    Let {
        name: String,
        type_annotation: Option<TypeAnnotation>,
        value: Box<TypedExpr>,
    },
    Assign {
        target: Box<TypedExpr>,
        value: Box<TypedExpr>,
    },
    UnaryOp {
        operator: String,
        operand: Box<TypedExpr>,
    },
    BinaryOp {
        left: Box<TypedExpr>,
        operator: String,
        right: Box<TypedExpr>,
    },
    Cast(Box<TypedExpr>, TypeAnnotation),
    EarlyReturn(Box<TypedExpr>),
    DotAccess {
        value: Box<TypedExpr>,
        field: String,
    },
    Index {
        value: Box<TypedExpr>,
        index: Box<TypedExpr>,
    },
    FunctionCall {
        function: Box<TypedExpr>,
        args: Vec<TypedExpr>,
    },
    If {
        condition: Box<TypedExpr>,
        then_branch: Vec<TypedExpr>,
        else_branch: Option<Vec<TypedExpr>>,
    },
    While {
        condition: Box<TypedExpr>,
        body: Vec<TypedExpr>,
    },
    For {
        iterator: String,
        iterable: Box<TypedExpr>,
        body: Vec<TypedExpr>,
    },
    Range {
        start: Box<TypedExpr>,
        end: Box<TypedExpr>,
        inclusive: bool,
    },
    Break,
    Continue,
    Return(Option<Box<TypedExpr>>),
    Block {
        expressions: Vec<TypedExpr>,
    },
    Do(Vec<TypedExpr>),
    Lambda {
        params: Vec<TypedFunctionParam>,
        return_type: Option<TypeAnnotation>,
        body: Box<TypedExpr>,
    },
    Match {
        value: Box<TypedExpr>,
        arms: Vec<TypedMatchArm>,
    },
    StructInit {
        name: String,
        fields: Vec<TypedStructInitField>,
    },
    EnumInit {
        enum_name: String,
        variant_name: String,
        fields: TypedEnumInitFields,
    },
}

#[derive(Debug, Clone)]
pub enum TypedLiteral {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone)]
pub struct TypedMatchArm {
    pub pattern: TypedPattern,
    pub guard: Option<Box<TypedExpr>>,
    pub body: Vec<TypedExpr>,
    pub typ: Type,
}

#[derive(Debug, Clone)]
pub enum TypedPattern {
    Literal(TypedLiteral),
    Variable(String),
    Wildcard,
    Tuple(Vec<TypedPattern>),
    Struct {
        name: String,
        fields: Vec<TypedStructPatternField>,
    },
    EnumVariant {
        name: String,
        fields: Vec<TypedPattern>,
    },
    Or(Vec<TypedPattern>),
}

#[derive(Debug, Clone)]
pub enum TypedStructPatternField {
    Shorthand(String),
    Typed { name: String, pattern: TypedPattern },
}

#[derive(Debug, Clone)]
pub struct RowTypeField {
    pub name: String,
    pub type_annotation: TypeAnnotation,
}

#[derive(Debug, Clone)]
pub enum TypedArrayElement {
    Value(TypedExpr),
    Spread(TypedExpr),
}

#[derive(Debug, Clone)]
pub enum TypedRowField {
    Field { name: String, value: TypedExpr },
    Spread(TypedExpr),
}

#[derive(Debug, Clone)]
pub enum TypedStructInitField {
    Field { name: String, value: TypedExpr },
    Spread(TypedExpr),
}

#[derive(Debug, Clone)]
pub enum TypedEnumInitFields {
    Tuple(Vec<TypedExpr>),
    Struct(Vec<TypedStructInitField>),
}
