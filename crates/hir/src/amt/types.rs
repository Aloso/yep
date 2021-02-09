use ast::{Spanned, SpannedList};

use super::name::UpperIdent;

#[derive(Clone)]
pub struct NamedType {
    pub name: Spanned<UpperIdent>,
    pub args: Spanned<SpannedList<TypeArgument>>,
}

#[derive(Clone)]
pub enum TypeArgument {
    Type(NamedType),
    Wildcard,
}

#[derive(Clone)]
pub struct GenericParam {
    pub name: Spanned<UpperIdent>,
    pub bounds: SpannedList<TypeBound>,
}

#[derive(Clone)]
pub enum TypeBound {
    // TODO: Interface/trait/contract/superclass
}
