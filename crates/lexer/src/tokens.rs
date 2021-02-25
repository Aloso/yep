use ast::token::{Ident, Operator, StringLiteral, Token, UpperIdent};
use ast::{LexError, Spanned, TextRange};
use logos::Lexer;

use super::numbers;
use super::syntax::{parse_keyword, IToken};

pub(super) fn lex(text: &str) -> Vec<Spanned<Token>> {
    let mut was_word = false;
    let mut v: Vec<Spanned<Token>> = Vec::new();

    for (t, span) in Lexer::<IToken>::new(text).spanned() {
        let span = TextRange::from(span);

        let data = match t {
            IToken::Word(word) => {
                if word.starts_with(|c: char| c.is_ascii_lowercase()) {
                    parse_keyword(word)
                        .map(Token::Keyword)
                        .unwrap_or_else(|| Token::Ident(Ident::new(word)))
                } else if word.starts_with(|c: char| c.is_ascii_uppercase()) {
                    Token::UpperIdent(UpperIdent::new(word))
                } else if word.contains(|c: char| c.is_ascii_digit()) {
                    Token::Error(LexError::InvalidNum)
                } else {
                    Token::Operator(Operator::new(word))
                }
            }
            IToken::NumberLit(input) => numbers::parse_number(input),
            IToken::StringLit(s) => Token::StringLit(StringLiteral::new(s)),
            IToken::Punct(p) => Token::Punct(p),
            IToken::Error => Token::Error(LexError::Unexpected),
            IToken::Ws => Token::Error(LexError::Ws),
        };
        if let Token::Error(LexError::Ws) = data {
            was_word = false;
        } else {
            let is_word = matches!(
                data,
                Token::NumberLit(_)
                    | Token::Ident(_)
                    | Token::UpperIdent(_)
                    | Token::Operator(_)
                    | Token::Keyword(_)
            );
            if was_word && is_word {
                let prev = v.pop().unwrap();
                let no_ws = Token::Error(LexError::NoWs);
                v.push(prev.span.extend_until(span.end()).embed(no_ws));
            } else {
                was_word = is_word;
                v.push(span.embed(data));
            }
        }
    }
    v.push(TextRange::from(text.len()..text.len()).embed(Token::Eof));
    v
}
