use super::items::{GenericParam, TypeArgument};
use super::{Error, LexerMut, Parse, ParseResult};

use crate::lexer::{Keyword, Punctuation, TokenData};

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
                lexer.next();
                Some(p)
            }
            _ => None,
        })
    }
}

impl Parse for Keyword {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        Ok(match lexer.peek().data() {
            TokenData::Keyword(kw) => {
                lexer.next();
                Some(kw)
            }
            _ => None,
        })
    }
}

pub(super) fn map<T, U>(
    f: impl Fn(LexerMut) -> ParseResult<T>,
    x: impl Fn(T) -> U + Copy,
) -> impl Fn(LexerMut) -> ParseResult<U> {
    move |lexer| Ok(f(lexer)?.map(x))
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

pub(super) fn wrap<T, U>(value: ParseResult<T>, f: impl Fn(T) -> U) -> ParseResult<U> {
    Ok(match value? {
        Some(t) => Some(f(t)),
        None => None,
    })
}

pub(super) fn expect2(expected: impl Into<TokenData>) -> impl FnOnce(LexerMut) -> ParseResult<()> {
    move |lexer: LexerMut| {
        Ok(if lexer.peek().data() == expected.into() {
            lexer.next();
            Some(())
        } else {
            None
        })
    }
}

pub(super) fn vec_separated<T>(
    lexer: LexerMut,
    mut f: impl FnMut(LexerMut) -> ParseResult<T>,
    separator: impl Into<TokenData> + Clone,
) -> ParseResult<Vec<T>> {
    let mut lexer_clone = lexer.clone();
    let first = uoret!(f(&mut lexer_clone)?);
    let mut results = vec![first];
    loop {
        if lexer_clone.eat(separator.clone()).is_some() {
            if let Some(next) = f(&mut lexer_clone)? {
                results.push(next);
                continue;
            }
        }
        break;
    }
    *lexer = lexer_clone;
    Ok(Some(results))
}

pub(super) fn vec_separated_opt<T>(
    lexer: LexerMut,
    f: impl FnMut(LexerMut) -> ParseResult<T>,
    separator: impl Into<TokenData> + Clone,
) -> Result<Vec<T>, Error> {
    Ok(vec_separated(lexer, f, separator)?.unwrap_or_default())
}

pub(super) fn parse_generics(lexer: LexerMut) -> ParseResult<Vec<GenericParam>> {
    uoret!(lexer.eat(Punctuation::OpenBracket));
    let generics =
        vec_separated(lexer, GenericParam::parse, Punctuation::Comma)?.unwrap_or_default();
    lexer.expect(Punctuation::CloseBracket)?;
    Ok(Some(generics))
}

pub(super) fn parse_type_arguments(lexer: LexerMut) -> ParseResult<Vec<TypeArgument>> {
    uoret!(lexer.eat(Punctuation::OpenBracket));
    let args = vec_separated(lexer, TypeArgument::parse, Punctuation::Comma)?.unwrap_or_default();
    lexer.expect(Punctuation::CloseBracket)?;
    Ok(Some(args))
}
