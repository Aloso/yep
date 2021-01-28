use crate::lexer::{LexError, LifelessToken, Token, TokenData};

pub use self::formatting::FancyFormat;
use self::items::Item;

mod expr;
mod formatting;
mod formatting_impl;
mod helpers;
mod items;
mod patterns;

type Tokens<'a> = &'a [Token<'a>];
type ParseResult<T> = Result<Option<T>, Error>;
type LexerMut<'a, 'b, 'c> = &'a mut Lexer<'b, 'c>;

#[derive(Debug, Clone)]
struct Lexer<'a, 'b> {
    tokens: &'a [Token<'b>],
}

impl<'a, 'b> Lexer<'a, 'b> {
    fn from_tokens(tokens: &'a [Token<'b>]) -> Self { Self { tokens } }

    /// Returns `Some(())` and advances the lexer if the next token matches
    /// `elem`
    #[must_use]
    fn eat(&mut self, token: impl Into<TokenData>) -> Option<()> {
        let (next, rest) = self.tokens.split_first().unwrap();
        if token.into() == next.data() {
            self.tokens = rest;
            Some(())
        } else {
            None
        }
    }

    fn expect(&mut self, token: impl Into<TokenData>) -> Result<(), Error> {
        let expected = token.into();
        let got = self.peek().data();
        if expected == got {
            self.next();
            Ok(())
        } else {
            Err(Error::ExpectedGot(expected, got))
        }
    }

    /// Return the next token and advance the lexer
    fn next(&mut self) -> TokenData {
        let (next, rest) = self.tokens.split_first().unwrap();
        self.tokens = rest;
        next.data()
    }

    /// Return the next token _without_ advancing the lexer
    fn peek(&self) -> &Token { &self.tokens[0] }

    fn finish(&mut self) -> Result<(), Error> {
        if self.tokens.is_empty()
            || (self.tokens.len() == 1 && self.tokens[0].data() == TokenData::EOF)
        {
            Ok(())
        } else {
            Err(Error::RemainingTokens(
                self.tokens.iter().map(Token::lifeless).collect(),
            ))
        }
    }

    pub fn parse_items(&'a mut self) -> Result<Vec<Item>, Error> {
        let mut results = Vec::new();
        while let Some(result) = Item::parse(self)? {
            results.push(result);
        }
        self.finish()?;
        Ok(results)
    }
}

pub fn parse(tokens: &[Token]) -> Result<Vec<Item>, Error> {
    Lexer::from_tokens(tokens).parse_items()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid character: {0:?}")]
    LexErrors(Vec<LexError>),

    #[error("There are remaining tokens that could not be parsed: {0:?}")]
    RemainingTokens(Vec<LifelessToken>),

    #[error("Expected {0:?}, got {1:?}")]
    ExpectedGot(TokenData, TokenData),

    #[error("Expected {0}, got {1:?}")]
    ExpectedGot2(&'static str, TokenData),
}

trait Parse: Sized {
    fn parse(lexer: LexerMut) -> ParseResult<Self>;

    #[inline]
    fn parse_expect(lexer: LexerMut, expect: &'static str) -> Result<Self, Error> {
        match Self::parse(lexer)? {
            Some(result) => Ok(result),
            None => Err(Error::ExpectedGot2(expect, lexer.peek().data())),
        }
    }
}
