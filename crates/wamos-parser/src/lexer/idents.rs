use std::fmt;

use string_interner::{DefaultSymbol, StringInterner};


/// An identifier. It has to fulfill the following criteria:
///
/// * ASCII-only
/// * It can only contain letters (`a-z`, `A-Z`), digits (`0-9`), underscores
///   (`_`) and operators (`+-*/%~<>=?!`)
/// * It must start with a lowercase letter (`a-z`)
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Ident(DefaultSymbol);

/// An operator. It has to fulfill the following criteria:
///
/// * ASCII-only
/// * It can only contain letters (`a-z`, `A-Z`), underscores (`_`) and
///   operators (`+-*/%~<>=?!`)
/// * It must start with an operator (`+-*/%~<>=?!`)
/// * The single equality sign (`=`) is NOT a valid operator
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Operator(DefaultSymbol);

/// An type name. It has to fulfill the following criteria:
///
/// * ASCII-only
/// * It can only contain letters (`a-z`, `A-Z`), digits (`0-9`), underscores
///   (`_`) and operators (`+-*/%~<>=?!`)
/// * It must start with an uppercase letter (`A-Z`)
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct UpperIdent(DefaultSymbol);

impl Ident {
    pub(crate) fn new(string: &str, interner: &mut StringInterner) -> Self {
        Self(interner.get_or_intern(string))
    }

    pub(crate) fn lookup<'a>(&self, interner: &'a StringInterner) -> Option<&'a str> {
        interner.resolve(self.0)
    }
}

impl Operator {
    pub(crate) fn new(string: &str, interner: &mut StringInterner) -> Self {
        Self(interner.get_or_intern(string))
    }

    pub(crate) fn lookup<'a>(&self, interner: &'a StringInterner) -> Option<&'a str> {
        interner.resolve(self.0)
    }
}

impl UpperIdent {
    pub(crate) fn new(string: &str, interner: &mut StringInterner) -> Self {
        Self(interner.get_or_intern(string))
    }

    pub(crate) fn lookup<'a>(&self, interner: &'a StringInterner) -> Option<&'a str> {
        interner.resolve(self.0)
    }
}

macro_rules! get_value {
    ($symbol:expr) => {
        format!("{:?}", $symbol)
            .trim_end_matches(" }")
            .trim_start_matches("SymbolU32 { value: ")
    };
}

impl fmt::Debug for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ident #{}", get_value!(self.0))
    }
}

impl fmt::Debug for UpperIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UpperIdent #{}", get_value!(self.0))
    }
}

impl fmt::Debug for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Operator #{}", get_value!(self.0))
    }
}
