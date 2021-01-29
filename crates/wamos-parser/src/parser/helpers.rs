use super::{
    items::{GenericParam, TypeArgument},
    Error,
};
use super::{LexerMut, Parse, ParseResult};

use crate::lexer::{Keyword, Punctuation, TokenData};
use crate::text_range::TextRange;

/// unwrap or return
#[macro_export]
macro_rules! uoret {
    ($e:expr) => {
        match $e {
            Some(inner) => inner,
            None => return Ok(None),
        }
    };
}

impl Parse for Punctuation {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        Ok(match lexer.peek().data() {
            TokenData::Punct(p) => {
                let span = lexer.next().span();
                Some((p, span))
            }
            _ => None,
        })
    }
}

impl Parse for Keyword {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        Ok(match lexer.peek().data() {
            TokenData::Keyword(kw) => {
                let span = lexer.next().span();
                Some((kw, span))
            }
            _ => None,
        })
    }
}

pub(super) fn map<T, U>(
    f: impl Fn(LexerMut) -> ParseResult<T>,
    x: impl Fn(T) -> U + Copy,
) -> impl Fn(LexerMut) -> ParseResult<U> {
    move |lexer| Ok(f(lexer)?.map(|(t, span)| (x(t), span)))
}

pub(super) fn map2<T, U>(
    f: impl Fn(LexerMut) -> ParseResult<T>,
    x: impl Fn(T, TextRange) -> (U, TextRange) + Copy,
) -> impl Fn(LexerMut) -> ParseResult<U> {
    move |lexer| Ok(f(lexer)?.map(|(t, span)| x(t, span)))
}

pub(super) fn or2<T>(
    f1: impl FnOnce(LexerMut) -> ParseResult<T>,
    f2: impl FnOnce(LexerMut) -> ParseResult<T>,
) -> impl FnOnce(LexerMut) -> ParseResult<T> {
    move |lexer| {
        Ok(match f1(lexer)? {
            Some(result) => Some(result),
            None => f2(lexer)?,
        })
    }
}

pub(super) fn or3<T>(
    f1: impl FnOnce(LexerMut) -> ParseResult<T>,
    f2: impl FnOnce(LexerMut) -> ParseResult<T>,
    f3: impl FnOnce(LexerMut) -> ParseResult<T>,
) -> impl FnOnce(LexerMut) -> ParseResult<T> {
    or2(f1, or2(f2, f3))
}

pub(super) fn or6<T>(
    f1: impl FnOnce(LexerMut) -> ParseResult<T>,
    f2: impl FnOnce(LexerMut) -> ParseResult<T>,
    f3: impl FnOnce(LexerMut) -> ParseResult<T>,
    f4: impl FnOnce(LexerMut) -> ParseResult<T>,
    f5: impl FnOnce(LexerMut) -> ParseResult<T>,
    f6: impl FnOnce(LexerMut) -> ParseResult<T>,
) -> impl FnOnce(LexerMut) -> ParseResult<T> {
    or2(or3(f1, f2, f3), or3(f4, f5, f6))
}

pub(super) fn expect2(
    expected: impl Into<TokenData>,
) -> impl FnOnce(LexerMut) -> ParseResult<()> {
    move |lexer: LexerMut| {
        Ok(if lexer.peek().data() == expected.into() {
            let span = lexer.next().span();
            Some(((), span))
        } else {
            None
        })
    }
}

pub(super) fn vec_separated<T>(
    lexer: LexerMut,
    mut f: impl FnMut(LexerMut) -> ParseResult<T>,
    separator: impl Into<TokenData> + Clone,
) -> ParseResult<Vec<(T, TextRange)>> {
    let mut lexer_clone = lexer.clone();
    let first = uoret!(f(&mut lexer_clone)?);
    let mut span = first.1;
    let mut results = vec![first];
    loop {
        if lexer_clone.eat(separator.clone()).is_some() {
            if let Some(next) = f(&mut lexer_clone)? {
                span = span.merge(next.1);
                results.push(next);
                continue;
            }
        }
        break;
    }
    *lexer = lexer_clone;
    Ok(Some((results, span)))
}

pub(super) fn enclosed<T>(
    parser: impl FnOnce(LexerMut) -> ParseResult<T>,
    left: impl Into<TokenData> + Clone,
    right: impl Into<TokenData> + Clone,
    on_error: impl FnOnce() -> Error,
) -> impl FnOnce(LexerMut) -> ParseResult<T> {
    move |lexer| {
        let span1: TextRange = uoret!(lexer.eat(left.clone()));
        let (inner, _) = match parser(lexer)? {
            Some(inner) => inner,
            None => return Err(on_error()),
        };
        let span2 = lexer.expect(right)?;

        Ok(Some((inner, span1.merge(span2))))
    }
}

pub(super) fn enclose_multiple<T>(
    parser: impl Fn(LexerMut) -> ParseResult<T> + Clone,
    left: impl Into<TokenData> + Clone,
    separator: impl Into<TokenData> + Clone,
    right: impl Into<TokenData> + Clone,
    trailing_separator: bool,
) -> impl FnOnce(LexerMut) -> ParseResult<Vec<(T, TextRange)>> {
    let parser_inner = move |lexer: LexerMut| {
        let items = vec_separated(lexer, parser.clone(), separator.clone())?;
        match items {
            Some(items) => {
                if trailing_separator && lexer.peek().data() == separator.into() {
                    lexer.next();
                }
                Ok(Some(items))
            }
            None => Ok(Some(Default::default())),
        }
    };
    enclosed(parser_inner, left, right, || {
        panic!("inner parser in enclose_multiple returned None")
    })
}

pub(super) fn enclose_multiple_expect<T>(
    parser: impl Fn(LexerMut) -> ParseResult<T> + Clone,
    left: impl Into<TokenData> + Clone,
    separator: impl Into<TokenData> + Clone,
    right: impl Into<TokenData> + Clone,
    trailing_separator: bool,
) -> impl FnOnce(LexerMut) -> Result<(Vec<(T, TextRange)>, TextRange), Error> {
    move |lexer| {
        let parser =
            enclose_multiple(parser, left.clone(), separator, right, trailing_separator);
        match parser(lexer)? {
            Some(res) => Ok(res),
            None => Err(Error::ExpectedGot(left.into(), lexer.peek().data())),
        }
    }
}

pub(super) fn parse_generics(
    lexer: LexerMut,
) -> ParseResult<Vec<(GenericParam, TextRange)>> {
    enclose_multiple(
        GenericParam::parse,
        Punctuation::OpenBracket,
        Punctuation::Comma,
        Punctuation::CloseBracket,
        true,
    )(lexer)
}

pub(super) fn parse_type_arguments(
    lexer: LexerMut,
) -> ParseResult<Vec<(TypeArgument, TextRange)>> {
    enclose_multiple(
        TypeArgument::parse,
        Punctuation::OpenBracket,
        Punctuation::Comma,
        Punctuation::CloseBracket,
        true,
    )(lexer)
}
