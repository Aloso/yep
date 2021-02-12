use arbitrary::{Result, Unstructured};
use ast::token::{Ident, Keyword, Punctuation, Token, UpperIdent};
use ast::TextRange;
use lexer::Program;

mod validate_tokens;

pub fn get_tokens(data: &[u8]) -> Result<Program> {
    let mut unstructured = Unstructured::new(data);

    let b: bool = unstructured.arbitrary()?;

    let mut program = unstructured
        .arbitrary_take_rest_iter::<Token>()?
        .enumerate()
        .map(|(i, t)| Ok(TextRange::from(i..i + 1).embed(t?)))
        .filter(|r| {
            r.as_ref()
                .map(|r| !matches!(**r, Token::Error(_) | Token::EOF))
                .unwrap_or(true)
        })
        .collect::<Result<Vec<_>>>()?;

    if b {
        program.push(TextRange::from(0..0).embed(Token::Keyword(Keyword::Fun)));
        program.push(TextRange::from(0..0).embed(Token::Ident(Ident::new("f"))));
        program.push(TextRange::from(0..0).embed(Token::Punct(Punctuation::OpenParen)));
        program.push(TextRange::from(0..0).embed(Token::Punct(Punctuation::CloseParen)));
        program.push(
            TextRange::from(0..0).embed(Token::UpperIdent(UpperIdent::new("Unit"))),
        );
        program.push(TextRange::from(0..0).embed(Token::Punct(Punctuation::OpenBrace)));
        program.rotate_right(6);

        program.push(TextRange::from(0..0).embed(Token::Punct(Punctuation::CloseBrace)));
    }

    program.push(TextRange::new(0, 0).embed(Token::EOF));

    Ok(program.into())
}

pub use validate_tokens::is_balanced;
