#![allow(dead_code)]

use ast::expr::Expr;
use ast::item::Item;
use ast::token::{Operator, Token, TokenData};
use ast::{Spanned, TextRange};
use validation::{Validate, ValidationError};

pub use self::formatting::ToBeauty;

pub mod expr;
pub mod formatting;
mod formatting_impl;
mod helpers;
pub mod items;
pub mod patterns;
mod validation;

type Tokens<'a> = &'a [Token<'a>];
type ParseResult<T> = Result<Option<Spanned<T>>, Error>;
type LexerMut<'a, 'b, 'c> = &'a mut Lexer<'b, 'c>;

#[derive(Debug, Clone)]
struct Lexer<'a, 'b> {
    tokens: &'a [Spanned<Token<'b>>],
}

impl<'a, 'b> Lexer<'a, 'b> {
    fn from_tokens(tokens: &'a [Spanned<Token<'b>>]) -> Self { Self { tokens } }

    /// Returns `Some(span)` and advances the lexer if the next token matches
    /// `elem`
    #[must_use]
    fn eat(&mut self, token: impl Into<TokenData>) -> Option<TextRange> {
        let (next, rest) = self.tokens.split_first().unwrap();
        if token.into() == next.data() {
            self.tokens = rest;
            Some(next.span)
        } else {
            None
        }
    }

    fn expect(&mut self, token: impl Into<TokenData>) -> Result<TextRange, Error> {
        let expected = token.into();
        let got = self.peek().data();
        if expected == got {
            Ok(self.next().span)
        } else {
            Err(Error::ExpectedGot(expected, got))
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
            || (self.tokens.len() == 1 && self.tokens[0].data() == TokenData::EOF)
        {
            Ok(())
        } else {
            Err(Error::RemainingTokens(
                self.tokens.iter().map(|t| t.map_ref(|t| t.data())).collect(),
            ))
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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("There are remaining tokens that could not be parsed: {0:?}")]
    RemainingTokens(Vec<Spanned<TokenData>>),

    #[error("Expected {0}")]
    Expected(&'static str),

    #[error("Expected {0:?}, got {1:?}")]
    ExpectedGot(TokenData, TokenData),

    #[error("Expected {0}, got {1:?}")]
    ExpectedGot2(&'static str, TokenData),

    #[error("Expected {0}, got {1:?}")]
    ExpectedGot3(&'static str, Expr),

    #[error("Expected {0}, got {1}")]
    ExpectedGot4(&'static str, &'static str),

    #[error(
        "Operators are not allowed here: {0:?}\n  tip: Wrap the operator in braces, \
         e.g. `{{+}}`"
    )]
    OperatorInsteadOfOperand(Operator),

    #[error("{0}")]
    ValidationError(#[from] ValidationError),
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
            None => Err(Error::ExpectedGot2(expect, lexer.peek().data())),
        }
    }
}

#[test]
fn run_parser_tests() {
    use std::ffi::OsStr;
    use std::fs::{read_to_string, File};
    use std::io::Write;

    for file in std::fs::read_dir("./tests").unwrap() {
        let path = file.unwrap().path();
        if path.is_file() && path.extension() == Some(OsStr::new("wa")) {
            let content: String = read_to_string(&path).unwrap();
            let content = content.trim_end();

            let lexed = lexer::lex(content);
            assert_eq!(lexed.errors(), vec![]);

            let actual = match parse(lexed.tokens()) {
                Ok(items) => format!("{:#?}", items),
                Err(err) => {
                    eprintln!("{}", content);
                    panic!("{}", err);
                }
            };
            let actual = actual.trim_end();

            let ast_path = path.with_extension("ast");
            if ast_path.exists() {
                let expected: String = read_to_string(ast_path).unwrap();
                let expected = expected.trim_end();

                if expected != actual {
                    let changes = difference::Changeset::new(expected, &actual, "\n");
                    eprintln!("{}", changes);
                    eprintln!("Input:\n{}", content);
                    panic!(
                        "{} differences between expected and actual output",
                        changes.distance
                    );
                }
            } else {
                let mut file = File::create(ast_path).unwrap();
                file.write_all(actual.as_bytes()).unwrap();
                file.write_all(b"\n").unwrap();
                file.flush().unwrap();
            }
        }
    }
}
