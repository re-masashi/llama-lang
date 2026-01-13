use crate::meta::Meta;

pub type Program = Vec<AstNode>;

#[derive(Debug, Clone)]
pub struct AstNode {
    pub meta: Meta,
    pub kind: AstNodeKind,
}

#[derive(Debug, Clone)]
pub enum AstNodeKind {
    Function {
        name: String,
        args: Vec<FunctionParam>,
        return_type: Option<TypeAnnotation>,
        generic_params: Vec<GenericParam>,
        body: Vec<Expr>,
    },
    Struct {
        name: String,
        generic_params: Vec<GenericParam>,
        fields: Vec<StructField>,
    },
    Enum {
        name: String,
        generic_params: Vec<GenericParam>,
        variants: Vec<EnumVariant>,
    },
    Trait {
        name: String,
        generic_params: Vec<GenericParam>,
        methods: Vec<MethodSignature>,
    },
    Impl {
        trait_name: Option<String>,
        for_type: TypeAnnotation,
        methods: Vec<AstNode>,
    },
    Extern {
        name: String,
        args: Vec<FunctionParam>,
        return_type: Option<TypeAnnotation>,
    },
    Const {
        name: String,
        type_annotation: Option<TypeAnnotation>,
        value: Expr,
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
pub struct MethodSignature {
    pub name: String,
    pub args: Vec<MethodParam>,
    pub return_type: Option<TypeAnnotation>,
    pub generic_params: Vec<GenericParam>,
    pub meta: Meta,
}

#[derive(Debug, Clone)]
pub enum MethodParam {
    SelfParam,
    TypedParam {
        name: String,
        type_annotation: TypeAnnotation,
    },
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub type_annotation: TypeAnnotation,
    pub meta: Meta,
}

#[derive(Debug, Clone)]
pub enum EnumVariant {
    Tuple {
        name: String,
        fields: Vec<TypeAnnotation>,
        meta: Meta,
    },
    Struct {
        name: String,
        fields: Vec<StructField>,
        meta: Meta,
    },
}

#[derive(Debug, Clone)]
pub struct FunctionParam {
    pub name: String,
    pub type_annotation: TypeAnnotation,
}

#[derive(Debug, Clone)]
pub struct TypeAnnotation {
    pub meta: Meta, // attributes not applicable here
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
pub struct Expr {
    pub meta: Meta,
    pub kind: ExprKind,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Literal(Literal),
    Array(Vec<ArrayElement>),
    Tuple(Vec<Expr>),
    Row(Vec<RowField>),
    Variable(String),
    Let {
        name: String,
        type_annotation: Option<TypeAnnotation>,
        value: Box<Expr>,
    },
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    UnaryOp {
        operator: String,
        operand: Box<Expr>,
    },
    BinaryOp {
        left: Box<Expr>,
        operator: String,
        right: Box<Expr>,
    },
    Cast(Box<Expr>, TypeAnnotation),
    EarlyReturn(Box<Expr>),
    DotAccess {
        // method calls become FunctionCall { function: DotAccess { .., field: method },
        // args: [..] }
        value: Box<Expr>,
        field: String,
    },
    Index {
        value: Box<Expr>,
        index: Box<Expr>,
    },
    FunctionCall {
        function: Box<Expr>,
        args: Vec<Expr>,
    },
    If {
        condition: Box<Expr>,
        then_branch: Vec<Expr>,
        else_branch: Option<Vec<Expr>>,
    },
    While {
        condition: Box<Expr>,
        body: Vec<Expr>,
    },
    For {
        iterator: String,
        iterable: Box<Expr>,
        body: Vec<Expr>,
    },
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
    },
    Break,
    Continue,
    Return(Option<Box<Expr>>),
    Block {
        expressions: Vec<Expr>,
    },
    Do(Vec<Expr>),
    Lambda {
        params: Vec<FunctionParam>,
        return_type: Option<TypeAnnotation>,
        body: Box<Expr>,
    },
    Match {
        value: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    StructInit {
        name: String,
        fields: Vec<StructInitField>,
    },
    EnumInit {
        enum_name: String,
        variant_name: String,
        fields: EnumInitFields,
    },
}

#[derive(Debug, Clone)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,
    pub body: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(Literal),
    Variable(String),
    Wildcard,
    Tuple(Vec<Pattern>),
    Struct {
        name: String,
        fields: Vec<StructPatternField>,
    },
    EnumVariant {
        name: String,
        fields: Vec<Pattern>,
    },
    Or(Vec<Pattern>),
}

#[derive(Debug, Clone)]
pub enum StructPatternField {
    Shorthand(String),
    Typed { name: String, pattern: Pattern },
}

#[derive(Debug, Clone)]
pub struct RowTypeField {
    pub name: String,
    pub type_annotation: TypeAnnotation,
}

#[derive(Debug, Clone)]
pub enum ArrayElement {
    Value(Expr),
    Spread(Expr),
}

#[derive(Debug, Clone)]
pub enum RowField {
    Field { name: String, value: Expr },
    Spread(Expr),
}

#[derive(Debug, Clone)]
pub enum StructInitField {
    Field { name: String, value: Expr },
    Spread(Expr),
}

#[derive(Debug, Clone)]
pub enum EnumInitFields {
    Tuple(Vec<Expr>),
    Struct(Vec<StructInitField>),
}
