use crate::expr::{Expr, Literal, TypeAscription};
use crate::token::Ident;

#[derive(Debug, Clone)]
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
}

#[derive(Debug, Clone)]
pub struct RangePattern {
    pub from: Box<Pattern>,
    pub to: Box<Pattern>,
}

#[derive(Debug, Clone)]
pub struct ClassPattern {
    pub name: Ident,
    pub fields: Vec<Pattern>,
}

#[derive(Debug, Clone)]
pub struct EnumPattern {
    pub name: Ident,
    pub field: Option<Box<Pattern>>,
}

#[derive(Debug, Clone)]
pub struct GuardPattern {
    pub pattern: Box<Pattern>,
    pub guard: Expr,
}
