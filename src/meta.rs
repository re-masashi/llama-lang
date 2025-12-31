use std::ops::Range;

#[derive(Debug, Clone)]
pub struct Meta {
    pub filename: String,
    pub range: Range<usize>,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<AttributeArg>,
}

#[derive(Debug, Clone)]
pub enum AttributeArg {
    Literal(String),
    KeyValue(String, String),
    Variable(String),
}
