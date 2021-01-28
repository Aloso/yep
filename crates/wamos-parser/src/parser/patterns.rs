use crate::lexer::Ident;

use super::expr::{Expr, Literal, TypeAscription};


#[derive(Debug)]
pub enum Pattern {
    Wildcard,
    Binding(Ident),
    Literal(Literal),
    Range(RangePattern),
    RangeExclusive(RangePattern),
    Class(ClassPattern),
    Enum(EnumPattern),
    TypeAscription(TypeAscription),
    Or(Vec<Pattern>),
    Guard(GuardPattern),
    // TODO: Tuple patterns
}

#[derive(Debug)]
pub struct RangePattern {
    pub from: Box<Pattern>,
    pub to: Box<Pattern>,
}

#[derive(Debug)]
pub struct ClassPattern {
    pub name: Ident,
    pub fields: Vec<Pattern>,
}

#[derive(Debug)]
pub struct EnumPattern {
    pub name: Ident,
    pub field: Option<Box<Pattern>>,
}

#[derive(Debug)]
pub struct GuardPattern {
    pub pattern: Box<Pattern>,
    pub guard: Expr,
}
