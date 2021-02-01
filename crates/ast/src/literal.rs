use std::fmt;

use string_interner::{DefaultSymbol, StringInterner};

/// Supported literals are
///
/// * Signed integer (Int)
/// * Unsigned integer (UInt)
/// * Float (Number)
///
/// # Grammar
///
/// ```no_test
/// SIGN  := '+' | '-'
/// E     := 'e' | 'E'
///
/// BIN_DIGIT := '0' | '1'
/// OCT_DIGIT := '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7'
/// DEC_DIGIT := OCT_DIGIT | '8' | '9'
/// HEX_DIGIT := DEC_DIGIT | 'a' | 'b' | 'c' | 'd' | 'e' | 'f'
///                        | 'A' | 'B' | 'C' | 'D' | 'E' | 'F'
///
/// BIN_SEQUENCE := BIN_DIGIT (BIN_DIGIT | '_')*
/// OCT_SEQUENCE := OCT_DIGIT (OCT_DIGIT | '_')*
/// DEC_SEQUENCE := DEC_DIGIT (DEC_DIGIT | '_')*
/// HEX_SEQUENCE := HEX_DIGIT (HEX_DIGIT | '_')*
///
/// BINARY      := SIGN? '0b' BIN_SEQUENCE
/// OCTAL       := SIGN? '0o' OCT_SEQUENCE
/// HEXADECIMAL := SIGN? '0x' HEX_SEQUENCE
/// DECIMAL     := SIGN? DEC_SEQUENCE
///
/// EXPONENT    := E SIGN? DEC_SEQUENCE
/// FLOAT       := SIGN? DEC_SEQUENCE '.' DEC_SEQUENCE EXPONENT?
///              | SIGN? DEC_SEQUENCE EXPONENT
///              | '.' DEC_SEQUENCE EXPONENT?
/// ```
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NumberLiteral {
    Int(i64),
    UInt(u64),
    Float(f64),
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct StringLiteral(DefaultSymbol);


impl StringLiteral {
    pub fn new(string: &str, interner: &mut StringInterner) -> Self {
        Self(interner.get_or_intern(string))
    }

    pub fn lookup<'a>(&self, interner: &'a StringInterner) -> Option<&'a str> {
        interner.resolve(self.0)
    }

    pub fn symbol(&self) -> DefaultSymbol { self.0 }
}

impl fmt::Debug for StringLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StringLit #{}", get_value!(self.0))
    }
}
