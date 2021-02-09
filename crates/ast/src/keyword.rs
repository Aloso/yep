#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
