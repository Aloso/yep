#![allow(dead_code)]

use ast::item::Item;
use ast::token::Token;
use ast::{Spanned, TextRange};
pub use error::Error;
use validation::Validate;

pub use self::formatting::ToBeauty;

pub mod error;
pub mod expr;
pub mod formatting;
mod formatting_impl;
mod helpers;
pub mod items;
pub mod patterns;
mod validation;

#[cfg(test)]
mod tests;

type Tokens<'a> = &'a [Token];
type ParseResult<T> = Result<Option<Spanned<T>>, Error>;
type LexerMut<'a, 'b> = &'a mut Lexer<'b>;

#[derive(Debug, Clone)]
struct Lexer<'a> {
    tokens: &'a [Spanned<Token>],
}

impl<'a> Lexer<'a> {
    fn from_tokens(tokens: &'a [Spanned<Token>]) -> Self { Self { tokens } }

    /// Returns `Some(span)` and advances the lexer if the next token matches
    /// `elem`
    #[must_use]
    fn eat(&mut self, token: impl Into<Token>) -> Option<TextRange> {
        let (next, rest) = self.tokens.split_first().unwrap();
        if token.into() == **next {
            self.tokens = rest;
            Some(next.span)
        } else {
            None
        }
    }

    fn expect(&mut self, token: impl Into<Token>) -> Result<TextRange, Error> {
        let expected = token.into();
        let got = self.peek();
        if &expected == got {
            Ok(self.next().span)
        } else {
            Err(Error::ExpectedGot(expected, got.clone()))
        }
    }

    /// Return the next token and advance the lexer
    fn next(&mut self) -> Spanned<Token> {
        let (next, rest) = self.tokens.split_first().unwrap();
        self.tokens = rest;
        next.clone()
    }

    /// Return the next token _without_ advancing the lexer
    fn peek(&self) -> &Token { &self.tokens[0].inner }

    fn len(&self) -> usize { self.tokens.len() }

    fn finish(&mut self) -> Result<(), Error> {
        if self.tokens.is_empty()
            || (self.tokens.len() == 1 && *self.tokens[0] == Token::Eof)
        {
            Ok(())
        } else {
            Err(Error::RemainingTokens(self.tokens.to_vec()))
        }
    }

    pub fn parse_items(&'a mut self) -> Result<Vec<Spanned<Item>>, Error> {
        let mut results = Vec::new();
        while let Some(result) = Item::parse(self)? {
            results.push(result);
        }
        self.finish()?;
        results.validate(())?;
        Ok(results)
    }
}

pub fn parse(tokens: &[Spanned<Token>]) -> Result<Vec<Spanned<Item>>, Error> {
    Lexer::from_tokens(tokens).parse_items()
}

trait Parse: Sized {
    fn parse(lexer: LexerMut) -> ParseResult<Self>;

    #[inline]
    fn parse_expect(
        lexer: LexerMut,
        expect: &'static str,
    ) -> Result<Spanned<Self>, Error> {
        match Self::parse(lexer)? {
            Some(result) => Ok(result),
            None => Err(Error::ExpectedGot2(expect, lexer.peek().clone())),
        }
    }
}
