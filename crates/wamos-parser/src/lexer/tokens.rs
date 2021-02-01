use ast::literal::StringLiteral;
use ast::name::{Ident, Operator, UpperIdent};
use ast::{LexError, Spanned, TextRange, Token, TokenData};
use logos::Lexer;
use string_interner::StringInterner;

use super::numbers;
use super::syntax::{parse_keyword, IToken};

pub(super) fn lex<'a>(
    text: &'a str,
    interner: &mut StringInterner,
) -> Vec<Spanned<Token<'a>>> {
    let mut was_word = false;
    let mut v: Vec<Spanned<Token<'a>>> = Vec::new();

    for (t, span) in Lexer::<IToken>::new(text).spanned() {
        let span = TextRange::from(span);

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
            IToken::StringLit(s) => TokenData::StringLit(StringLiteral::new(s, interner)),
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
                    prev.span.extend_until(span.end()),
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
