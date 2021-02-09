use string_interner::DefaultSymbol;

use crate::arena::Arena;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Ident(DefaultSymbol);

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct UpperIdent(DefaultSymbol);

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Operator(DefaultSymbol);

impl Ident {
    pub fn new(inner: DefaultSymbol) -> Self { Self(inner) }

    pub fn get(self, arena: &Arena) -> &str { &arena[self.0] }
}

impl Operator {
    pub fn new(inner: DefaultSymbol) -> Self { Self(inner) }

    pub fn get(self, arena: &Arena) -> &str { &arena[self.0] }
}

impl UpperIdent {
    pub fn new(inner: DefaultSymbol) -> Self { Self(inner) }

    pub fn get(self, arena: &Arena) -> &str { &arena[self.0] }
}
