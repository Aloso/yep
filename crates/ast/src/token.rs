use std::fmt;
use std::marker::PhantomData;

pub use crate::keyword::Keyword;
pub use crate::literal::{NumberLiteral, StringLiteral};
pub use crate::name::{Ident, Operator, UpperIdent};
pub use crate::punct::Punctuation;
use crate::{LexError, Spanned, TextRange};

#[derive(Copy, Clone)]
pub struct Token<'a> {
    pub data: TokenData,
    lt: PhantomData<&'a str>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenData {
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

impl From<Punctuation> for TokenData {
    fn from(x: Punctuation) -> Self { TokenData::Punct(x) }
}
impl From<NumberLiteral> for TokenData {
    fn from(x: NumberLiteral) -> Self { TokenData::NumberLit(x) }
}
impl From<Ident> for TokenData {
    fn from(x: Ident) -> Self { TokenData::Ident(x) }
}
impl From<UpperIdent> for TokenData {
    fn from(x: UpperIdent) -> Self { TokenData::UpperIdent(x) }
}
impl From<Operator> for TokenData {
    fn from(x: Operator) -> Self { TokenData::Operator(x) }
}
impl From<Keyword> for TokenData {
    fn from(x: Keyword) -> Self { TokenData::Keyword(x) }
}
impl From<LexError> for TokenData {
    fn from(x: LexError) -> Self { TokenData::Error(x) }
}

impl<'a> Token<'a> {
    pub fn new(data: TokenData, span: impl Into<TextRange>) -> Spanned<Self> {
        Spanned::new(Self { data, lt: PhantomData }, span.into())
    }

    pub fn data(&self) -> TokenData { self.data }

    pub fn kind(&self) -> TokenKind {
        match self.data {
            TokenData::Punct(_) => TokenKind::Punct,
            TokenData::StringLit(_) => TokenKind::StringLit,
            TokenData::NumberLit(_) => TokenKind::NumberLit,
            TokenData::Ident(_) => TokenKind::Ident,
            TokenData::UpperIdent(_) => TokenKind::UpperIdent,
            TokenData::Operator(_) => TokenKind::Operator,
            TokenData::Keyword(_) => TokenKind::Keyword,
            TokenData::Error(_) => TokenKind::Error,
            TokenData::EOF => TokenKind::EOF,
        }
    }

    pub fn lex_error(&self) -> Option<LexError> {
        match self.data {
            TokenData::Error(e) => Some(e),
            _ => None,
        }
    }

    pub fn debug(&self, text: &str, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            match &self.data {
                TokenData::Punct(_) => write!(f, "`{}`", text),
                TokenData::NumberLit(n) => write!(f, "{:?}@`{}`", n, text),
                TokenData::StringLit(_) => write!(f, "s`{}`", text),
                TokenData::Ident(_) => write!(f, "i`{}`", text),
                TokenData::UpperIdent(_) => write!(f, "I`{}`", text),
                TokenData::Operator(_) => write!(f, "o`{}`", text),
                TokenData::Keyword(_) => write!(f, "k`{}`", text),
                TokenData::Error(e) => write!(f, "{:?}@`{}`", e, text),
                TokenData::EOF => write!(f, "EOF"),
            }
        } else {
            write!(f, "{}", text)
        }
    }

    pub fn debug_to_string(&self, text: &str, alternate: bool) -> String {
        if alternate {
            format!("{:#?}", TokenFormatting { token: self, text })
        } else {
            format!("{:?}", TokenFormatting { token: self, text })
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

struct TokenFormatting<'a> {
    token: &'a Token<'a>,
    text: &'a str,
}

impl fmt::Debug for TokenFormatting<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.token.debug(self.text, f)
    }
}

impl fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.data, f)
    }
}

impl Spanned<Token<'_>> {
    pub fn debug(&self, text: &str, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.debug(&text[self.span], f)?;
        if f.alternate() {
            write!(f, " @ {:?}", self.span)?;
        }
        Ok(())
    }

    pub fn debug_to_string(&self, text: &str, alternate: bool) -> String {
        let text = &text[self.span];
        if alternate {
            format!("{:#?}", TokenFormatting { token: self, text })
        } else {
            format!("{:?}", TokenFormatting { token: self, text })
        }
    }
}
