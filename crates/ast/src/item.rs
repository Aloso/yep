use std::fmt;

use crate::expr::{Block, Expr};
use crate::name::{Ident, Operator};
use crate::token::UpperIdent;
use crate::{Spanned, SpannedList};

#[derive(Debug, Clone)]
pub enum Item {
    Function(Function),
    Class(Class),
    Enum(Enum),
    Impl(Impl),
    Use(Use),
}

#[derive(Debug, Copy, Clone)]
pub enum ItemKind {
    Function,
    Class,
    Enum,
    Impl,
    Use,
}

impl Item {
    pub fn kind(&self) -> ItemKind {
        match self {
            Item::Function(_) => ItemKind::Function,
            Item::Class(_) => ItemKind::Class,
            Item::Enum(_) => ItemKind::Enum,
            Item::Impl(_) => ItemKind::Impl,
            Item::Use(_) => ItemKind::Use,
        }
    }
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

#[derive(Clone)]
pub enum Name {
    Operator(Operator),
    Ident(Ident),
    Type(UpperIdent),
}

impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Name::Operator(o) => fmt::Debug::fmt(o, f),
            Name::Ident(o) => fmt::Debug::fmt(o, f),
            Name::Type(o) => fmt::Debug::fmt(o, f),
        }
    }
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

#[derive(Debug, Clone)]
pub struct Impl {
    pub generics: Spanned<SpannedList<GenericParam>>,
    pub r#trait: Option<Spanned<NamedType>>,
    pub ty: Spanned<NamedType>,
    pub items: Spanned<SpannedList<Item>>,
}

#[derive(Debug, Clone)]
pub struct Use {
    pub path: Spanned<SpannedList<Name>>,
    pub wildcard: Option<Spanned<()>>,
}
