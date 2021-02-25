use ast::pattern::Pattern;
use ast::token::{Ident, Keyword};

use crate::{uoret, Parse};

impl Parse for Pattern {
    fn parse(lexer: crate::LexerMut) -> crate::ParseResult<Self> {
        let kw = uoret!(lexer.eat(Keyword::Let));
        let ident = Ident::parse_expect(lexer, "identifier")?;
        Ok(Some(kw.merge(ident.span).embed(Pattern::Binding(ident.inner))))
    }
}
