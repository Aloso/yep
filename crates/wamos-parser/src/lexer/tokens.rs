use std::{fmt, marker::PhantomData, ops::Range};

use logos::Lexer;
use string_interner::StringInterner;

use super::syntax::{parse_keyword, IToken};
use super::{numbers, Ident, Keyword, NumberLiteral, Operator, Punctuation, UpperIdent};

#[derive(Clone)]
pub struct Token<'a> {
    pub(super) data: TokenData,
    span: Range<usize>,
    lt: PhantomData<&'a str>,
}

pub struct LifelessToken {
    pub data: TokenData,
    pub span: Range<usize>,
}

impl Token<'_> {
    pub fn lifeless(&self) -> LifelessToken {
        LifelessToken { data: self.data, span: self.span.clone() }
    }
}

impl LifelessToken {
    pub fn to_static_token(&self) -> Token<'static> {
        Token { data: self.data, span: self.span.clone(), lt: PhantomData }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenData {
    Punct(Punctuation),
    NumberLit(NumberLiteral),
    Ident(Ident),
    UpperIdent(UpperIdent),
    Operator(Operator),
    Keyword(Keyword),
    Error(LexError),
    EOF,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LexError {
    Unexpected,
    InvalidNum,
    NoWS,
    WS,
}

impl<'a> Token<'a> {
    fn new(data: TokenData, span: Range<usize>) -> Self {
        Self { data, span, lt: PhantomData }
    }

    pub fn data(&self) -> TokenData { self.data }

    pub fn kind(&self) -> TokenKind {
        match self.data {
            TokenData::Punct(_) => TokenKind::Punct,
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

    pub fn span(&self) -> Range<usize> { self.span.clone() }

    pub fn debug(&self, text: &str, f: &mut fmt::Formatter) -> fmt::Result {
        let text = &text[self.span.clone()];
        if f.alternate() {
            match &self.data {
                TokenData::Punct(_) => write!(f, "`{}`", text),
                TokenData::NumberLit(n) => write!(f, "{:?}@`{}`", n, text),
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
    NumberLit,
    Ident,
    UpperIdent,
    Operator,
    Keyword,
    Error,
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

pub(super) fn lex<'a>(text: &'a str, interner: &mut StringInterner) -> Vec<Token<'a>> {
    let mut was_word = false;
    let mut v: Vec<Token<'a>> = Vec::new();

    for (t, span) in Lexer::<IToken>::new(text).spanned() {
        let data = match t {
            IToken::Word(word) => {
                if word.starts_with(|c: char| c.is_ascii_lowercase()) {
                    parse_keyword(word)
                        .map(TokenData::Keyword)
                        .unwrap_or_else(|| TokenData::Ident(Ident::new(word, interner)))
                } else if word.starts_with(|c: char| c.is_ascii_uppercase()) {
                    TokenData::UpperIdent(UpperIdent::new(word, interner))
                } else if word.contains(|c: char| c.is_ascii_digit()) {
                    TokenData::Error(LexError::InvalidNum)
                } else {
                    TokenData::Operator(Operator::new(word, interner))
                }
            }
            IToken::NumberLit(input) => numbers::parse_number(input),
            IToken::Punct(p) => TokenData::Punct(p),
            IToken::Error => TokenData::Error(LexError::Unexpected),
            IToken::WS => TokenData::Error(LexError::WS),
        };
        if let TokenData::Error(LexError::WS) = data {
            was_word = false;
        } else {
            let is_word = matches!(
                data,
                TokenData::NumberLit(_)
                    | TokenData::Ident(_)
                    | TokenData::UpperIdent(_)
                    | TokenData::Operator(_)
                    | TokenData::Keyword(_)
            );
            if was_word && is_word {
                let prev = v.pop().unwrap();
                v.push(Token::new(
                    TokenData::Error(LexError::NoWS),
                    prev.span.start..span.end,
                ));
            } else {
                was_word = is_word;
                v.push(Token::new(data, span));
            }
        }
    }
    v.push(Token::new(TokenData::EOF, text.len()..text.len()));
    v
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
        write!(f, "{}..{} @ {:?}", self.span.start, self.span.end, &self.data)
    }
}

impl fmt::Debug for LifelessToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{} @ {:?}", self.span.start, self.span.end, &self.data)
    }
}
