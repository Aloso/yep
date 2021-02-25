use ast::token::{Keyword, Punctuation};
use logos::Logos;

/// Intermediate token type for lexing
#[derive(Logos)]
pub(super) enum IToken<'a> {
    #[regex(r"([ \t\n\f]+|#.*)+")]
    Ws,

    #[token(".", |_| Punctuation::Dot)]
    #[token(",", |_| Punctuation::Comma)]
    #[token(":", |_| Punctuation::Colon)]
    #[token(";", |_| Punctuation::Semicolon)]
    #[token("=", |_| Punctuation::Equals)]
    #[token("&", |_| Punctuation::And)]
    #[token("|", |_| Punctuation::Pipe)]
    #[token("\\", |_| Punctuation::Backslash)]
    #[token("@", |_| Punctuation::At)]
    #[token("_", |_| Punctuation::Underscore)]
    #[token("(", |_| Punctuation::OpenParen)]
    #[token(")", |_| Punctuation::CloseParen)]
    #[token("[", |_| Punctuation::OpenBracket)]
    #[token("]", |_| Punctuation::CloseBracket)]
    #[token("{", |_| Punctuation::OpenBrace)]
    #[token("}", |_| Punctuation::CloseBrace)]
    Punct(Punctuation),

    #[regex(
        r"[+-]?\d[a-zA-Z_+\-*/%~<>=!0-9]*(\.\d[a-zA-Z_+\-*/%~<>=!?0-9]*)?",
        priority = 2
    )]
    #[regex(r"\.\d[a-zA-Z_+\-*/%~<>=!?0-9]*")]
    NumberLit(&'a str),

    #[regex(r#""([^"\\]|\\.)*""#)]
    StringLit(&'a str),

    #[regex(r"[a-zA-Z_+\-*/%~<>=!?][a-zA-Z_+\-*/%~<>=!?0-9]*", priority = 1)]
    Word(&'a str),

    #[error]
    Error,
}


pub(super) fn parse_keyword(s: &str) -> Option<Keyword> {
    Some(match s {
        "and" => Keyword::And,
        "match" => Keyword::Match,
        "class" => Keyword::Class,
        "enum" => Keyword::Enum,
        "use" => Keyword::Use,
        "for" => Keyword::For,
        "fun" => Keyword::Fun,
        "impl" => Keyword::Impl,
        "let" => Keyword::Let,
        "not" => Keyword::Not,
        "or" => Keyword::Or,
        "type" => Keyword::Type,
        "var" => Keyword::Var,
        _ => return None,
    })
}
