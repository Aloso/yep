#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Keyword {
    // Constructs
    Fun,
    Type,
    Class,
    Enum,
    Impl,

    // Expressions
    Let,
    Var,
    Case,
    And,
    Or,
    Not,
    For,
}
