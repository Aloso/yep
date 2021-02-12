use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Keyword {
    // Constructs
    Fun,
    Type,
    Class,
    Enum,
    Impl,
    Use,

    // Expressions
    Let,
    Var,
    Match,
    And,
    Or,
    Not,
    For,
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Keyword::Fun => "fun",
            Keyword::Type => "type",
            Keyword::Class => "class",
            Keyword::Enum => "enum",
            Keyword::Impl => "impl",
            Keyword::Use => "use",
            Keyword::Let => "let",
            Keyword::Var => "var",
            Keyword::Match => "match",
            Keyword::And => "and",
            Keyword::Or => "or",
            Keyword::Not => "not",
            Keyword::For => "for",
        })
    }
}
