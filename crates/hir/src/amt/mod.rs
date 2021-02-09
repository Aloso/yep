//! The abstract module tree

use ast::{Spanned, SpannedList};

use crate::arena::Idx;

use self::name::{Ident, Operator, UpperIdent};
use self::types::GenericParam;

pub mod expr;
pub mod literal;
pub mod name;
pub mod pattern;
pub mod types;


pub struct Namespace {
    pub kind: NamespaceKind,
    pub items: Box<[Spanned<Idx<Item>>]>,
    pub types: Vec<Idx<Item>>,
    pub values: Vec<Idx<Item>>,
}

pub enum NamespaceKind {
    Module,
    Impl,
    Enum,
}

pub enum Item {
    Function(Function),
    Class(Class),
    Enum(Enum),
    Impl(Impl),
}

pub struct Function {
    pub name: Spanned<Name>,
    pub generics: SpannedList<GenericParam>,
    pub args: SpannedList<FunctionArg>,
}

pub struct FunctionArg {
    pub name: Spanned<Name>,
    pub ty: Spanned<Type>,
}

pub struct Class {
    pub name: Spanned<Name>,
    pub generics: SpannedList<GenericParam>,
}

pub struct Enum {
    pub name: Spanned<Name>,
    pub generics: SpannedList<GenericParam>,
}

pub struct Impl {
    pub generics: SpannedList<GenericParam>,
}

pub struct Type {
    pub name: Spanned<UpperIdent>,
    pub impls: Vec<Impl>,
}

#[derive(Clone)]
pub enum Name {
    Operator(Operator),
    Ident(Ident),
    Type(UpperIdent),
}
