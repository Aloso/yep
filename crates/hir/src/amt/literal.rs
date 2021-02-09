use string_interner::DefaultSymbol;

use crate::arena::Arena;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NumberLiteral {
    Int(i64),
    UInt(u64),
    Float(f64),
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct StringLiteral(DefaultSymbol);

impl StringLiteral {
    pub fn new(inner: DefaultSymbol) -> Self { Self(inner) }

    pub fn get(self, arena: &Arena) -> &str { &arena[self.0] }
}
