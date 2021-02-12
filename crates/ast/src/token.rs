use std::fmt;

pub use crate::keyword::Keyword;
pub use crate::literal::{NumberLiteral, StringLiteral};
pub use crate::name::{Ident, Operator, UpperIdent};
pub use crate::punct::Punctuation;
use crate::LexError;

#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum Token {
    Punct(Punctuation),
    StringLit(StringLiteral),
    NumberLit(NumberLiteral),
    Ident(Ident),
    UpperIdent(UpperIdent),
    Operator(Operator),
    Keyword(Keyword),
    Error(LexError),
    EOF,
}

impl From<Punctuation> for Token {
    fn from(x: Punctuation) -> Self { Token::Punct(x) }
}
impl From<NumberLiteral> for Token {
    fn from(x: NumberLiteral) -> Self { Token::NumberLit(x) }
}
impl From<Ident> for Token {
    fn from(x: Ident) -> Self { Token::Ident(x) }
}
impl From<UpperIdent> for Token {
    fn from(x: UpperIdent) -> Self { Token::UpperIdent(x) }
}
impl From<Operator> for Token {
    fn from(x: Operator) -> Self { Token::Operator(x) }
}
impl From<Keyword> for Token {
    fn from(x: Keyword) -> Self { Token::Keyword(x) }
}
impl From<LexError> for Token {
    fn from(x: LexError) -> Self { Token::Error(x) }
}

impl Token {
    pub fn kind(&self) -> TokenKind {
        match self {
            Token::Punct(_) => TokenKind::Punct,
            Token::StringLit(_) => TokenKind::StringLit,
            Token::NumberLit(_) => TokenKind::NumberLit,
            Token::Ident(_) => TokenKind::Ident,
            Token::UpperIdent(_) => TokenKind::UpperIdent,
            Token::Operator(_) => TokenKind::Operator,
            Token::Keyword(_) => TokenKind::Keyword,
            Token::Error(_) => TokenKind::Error,
            Token::EOF => TokenKind::EOF,
        }
    }

    pub fn lex_error(&self) -> Option<LexError> {
        match *self {
            Token::Error(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Punct,
    StringLit,
    NumberLit,
    Ident,
    UpperIdent,
    Operator,
    Keyword,
    Error,
    EOF,
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Punct(p) => write!(f, "`{}`", p),
            Token::StringLit(l) => write!(f, "s`{}`", l),
            Token::NumberLit(l) => write!(f, "{:?}", l),
            Token::Ident(i) => write!(f, "i`{}`", i),
            Token::UpperIdent(i) => write!(f, "I`{}`", i),
            Token::Operator(i) => write!(f, "o`{}`", i),
            Token::Keyword(k) => write!(f, "k`{}`", k),
            Token::Error(e) => write!(f, "{:?}", e),
            Token::EOF => write!(f, "EOF"),
        }
    }
}
