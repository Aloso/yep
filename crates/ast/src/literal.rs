use std::fmt;

use tinystring::TinyString;

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

impl fmt::Display for NumberLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumberLiteral::Int(i) => fmt::Display::fmt(i, f),
            NumberLiteral::UInt(u) => fmt::Display::fmt(u, f),
            NumberLiteral::Float(n) => fmt::Display::fmt(n, f),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct StringLiteral(TinyString);


impl StringLiteral {
    pub fn new(string: impl Into<TinyString>) -> Self { Self(string.into()) }

    pub fn get(&self) -> &str { &*self.0 }

    pub fn inner(&self) -> TinyString { self.0.clone() }
}

impl fmt::Display for StringLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for StringLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StringLiteral {:?}", &self.0)
    }
}


#[cfg(feature = "fuzz")]
impl arbitrary::Arbitrary for StringLiteral {
    fn arbitrary(_: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        Ok(StringLiteral(TinyString::from("\"s\"")))
    }
}

#[cfg(feature = "fuzz")]
impl arbitrary::Arbitrary for NumberLiteral {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        #[derive(arbitrary::Arbitrary)]
        enum ArbitraryNumerLit {
            Int,
            UInt,
            Float,
        }

        Ok(match u.arbitrary::<ArbitraryNumerLit>()? {
            ArbitraryNumerLit::Int => NumberLiteral::Int(42),
            ArbitraryNumerLit::UInt => NumberLiteral::UInt(41),
            ArbitraryNumerLit::Float => NumberLiteral::Float(40.0),
        })
    }
}
