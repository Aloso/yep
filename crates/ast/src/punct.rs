use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
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

impl fmt::Display for Punctuation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Punctuation::Dot => ".",
            Punctuation::Comma => ",",
            Punctuation::Colon => ":",
            Punctuation::Semicolon => ";",
            Punctuation::Equals => "=",
            Punctuation::And => "&",
            Punctuation::Pipe => "|",
            Punctuation::QuestionMark => "?",
            Punctuation::Backslash => "\\",
            Punctuation::At => "@",
            Punctuation::Underscore => "_",
            Punctuation::OpenParen => "(",
            Punctuation::CloseParen => ")",
            Punctuation::OpenBracket => "[",
            Punctuation::CloseBracket => "]",
            Punctuation::OpenBrace => "{",
            Punctuation::CloseBrace => "}",
        })
    }
}
