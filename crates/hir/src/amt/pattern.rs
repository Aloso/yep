use ast::Spanned;

use crate::arena::Idx;

use super::expr::{Expr, Literal, TypeAscription};
use super::name::Ident;

#[derive(Clone)]
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

#[derive(Clone)]
pub struct RangePattern {
    pub from: Box<Pattern>,
    pub to: Box<Pattern>,
}

#[derive(Clone)]
pub struct ClassPattern {
    pub name: Ident,
    pub fields: Vec<Pattern>,
}

#[derive(Clone)]
pub struct EnumPattern {
    pub name: Ident,
    pub field: Option<Box<Pattern>>,
}

#[derive(Clone)]
pub struct GuardPattern {
    pub pattern: Box<Pattern>,
    pub guard: Spanned<Idx<Expr>>,
}
