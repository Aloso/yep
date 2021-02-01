#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Punctuation {
    Dot,
    Comma,
    Colon,
    Semicolon,
    Equals,
    And,
    Pipe,
    QuestionMark,
    Backslash,
    At,
    Underscore,

    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    OpenBrace,
    CloseBrace,
}
