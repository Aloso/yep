use crate::expr::{Block, Expr};
use crate::name::{Ident, Operator};
use crate::token::UpperIdent;
use crate::{Spanned, SpannedList};

#[derive(Debug, Clone)]
pub enum Item {
    Function(Function),
    Class(Class),
    Enum(Enum),
}

#[derive(Debug, Clone)]
pub struct NamedType {
    pub name: Spanned<UpperIdent>,
    pub args: Spanned<SpannedList<TypeArgument>>,
}

#[derive(Debug, Clone)]
pub enum TypeArgument {
    Type(NamedType),
    Wildcard,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Spanned<Name>,
    pub generics: Spanned<SpannedList<GenericParam>>,
    pub args: Spanned<SpannedList<FunArgument>>,
    pub return_ty: Option<Spanned<NamedType>>,
    pub body: Option<Spanned<Block>>,
}

#[derive(Debug, Clone, Copy)]
pub enum Name {
    Operator(Operator),
    Ident(Ident),
    Type(UpperIdent),
}

#[derive(Debug, Clone)]
pub struct FunArgument {
    pub name: Ident,
    pub ty: Option<Spanned<NamedType>>,
    pub default: Option<Spanned<Expr>>,
}

#[derive(Debug, Clone)]
pub struct Class {
    pub name: Spanned<UpperIdent>,
    pub generics: Spanned<SpannedList<GenericParam>>,
    pub fields: Spanned<SpannedList<ClassField>>,
}

#[derive(Debug, Clone)]
pub struct GenericParam {
    pub name: Spanned<UpperIdent>,
    pub bounds: SpannedList<TypeBound>,
}

#[derive(Debug, Clone)]
pub enum TypeBound {
    // TODO: Interface/trait/contract/superclass
}

#[derive(Debug, Clone)]
pub struct ClassField {
    pub name: Spanned<Ident>,
    pub ty: Option<Spanned<NamedType>>,
    pub default: Option<Spanned<Expr>>,
}

#[derive(Debug, Clone)]
pub struct Enum {
    pub name: Spanned<UpperIdent>,
    pub generics: Spanned<SpannedList<GenericParam>>,
    pub variants: Spanned<SpannedList<EnumVariant>>,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: Spanned<Ident>,
    pub arguments: Option<Spanned<SpannedList<ClassField>>>,
}
