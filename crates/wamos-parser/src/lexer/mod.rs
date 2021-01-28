mod idents;
pub(super) mod numbers;
mod syntax;
mod tokens;

use std::fmt;

use string_interner::StringInterner;

pub use idents::{Ident, Operator, UpperIdent};
pub use numbers::NumberLiteral;
pub use syntax::{Keyword, Punctuation};
pub use tokens::{LexError, LifelessToken, Token, TokenData, TokenKind};

pub fn lex(text: &str) -> Program<'_> {
    let mut interner = StringInterner::new();
    let tokens = tokens::lex(text, &mut interner);
    Program { text, tokens }
}

pub fn lex_with_interner<'a>(text: &'a str, interner: &'a mut StringInterner) -> Program<'a> {
    let tokens = tokens::lex(text, interner);
    Program { text, tokens }
}

pub struct Program<'a> {
    text: &'a str,
    tokens: Vec<Token<'a>>,
}

impl<'a> Program<'a> {
    pub fn token_len(&self) -> usize { self.tokens.len() }

    pub fn tokens(&'a self) -> &'a [Token<'a>] { &self.tokens }

    pub fn text(&self) -> &str { self.text }

    pub fn errors(&self) -> Vec<LexError> {
        let mut lex_errors = Vec::new();
        for t in self.tokens() {
            if let Some(e) = t.lex_error() {
                lex_errors.push(e);
            }
        }
        lex_errors
    }

    pub fn with_tokens(&'a self, tokens: &'a [Token<'a>]) -> Self {
        Program {
            tokens: tokens.to_vec(),
            text: self.text,
        }
    }

    pub fn with_lifeless_tokens(&'a self, tokens: &'a [LifelessToken]) -> Self {
        Program {
            tokens: tokens.iter().map(|l| l.to_static_token()).collect(),
            text: self.text,
        }
    }

    pub fn no_eof(&mut self) {
        match self.tokens.pop() {
            Some(t) if t.data != TokenData::EOF => self.tokens.push(t),
            _ => {}
        }
    }
}

impl fmt::Debug for Program<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        let multi_line = matches!(f.align(), Some(fmt::Alignment::Left));
        if multi_line {
            f.write_str("\n")?;
        }

        for (i, t) in self.tokens.iter().enumerate() {
            if multi_line {
                f.write_str("    ")?;
                t.debug(self.text, f)?;
                f.write_str("\n")?;
            } else {
                if i != 0 {
                    f.write_str(" ")?;
                }
                t.debug(self.text, f)?;
            }
        }

        f.write_str("]")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    macro_rules! assert_list {
        ($input:expr, $fmt:literal) => {{
            let p = super::lex($input);
            let formatted = format!("{:<#?}", p);
            assert!(formatted == $fmt, "{}  doesn't equal  {}", formatted, $fmt);
        }};
    }

    #[test]
    fn test_tokens() {
        assert_list!(
            "hello, world!",
            r#"[
    i`hello`
    `,`
    i`world!`
    EOF
]"#
        );
    }

    #[test]
    fn test_function() {
        assert_list!(
            "\
# This is a
# very cool function!
fun foo() Bool {
    let x-y = 5 Int;
    var test! = \\ x-y >gt 42;
    test!()
}",
            r#"[
    k`fun`
    i`foo`
    `(`
    `)`
    I`Bool`
    `{`
    k`let`
    i`x-y`
    `=`
    Int(5)@`5`
    I`Int`
    `;`
    k`var`
    i`test!`
    `=`
    `\`
    i`x-y`
    o`>gt`
    Int(42)@`42`
    `;`
    i`test!`
    `(`
    `)`
    `}`
    EOF
]"#
        );
    }

    #[test]
    fn test_types() {
        assert_list!(
            "class Foo(x Int, y Int)

            class Bar[T](x T, y T)

            impl[T Clone] Clone for Bar[T] {
                fun clone(self) Bar[T] {
                    Bar(.x.clone, .y.clone)
                }
            }",
            r#"[
    k`class`
    I`Foo`
    `(`
    i`x`
    I`Int`
    `,`
    i`y`
    I`Int`
    `)`
    k`class`
    I`Bar`
    `[`
    I`T`
    `]`
    `(`
    i`x`
    I`T`
    `,`
    i`y`
    I`T`
    `)`
    k`impl`
    `[`
    I`T`
    I`Clone`
    `]`
    I`Clone`
    k`for`
    I`Bar`
    `[`
    I`T`
    `]`
    `{`
    k`fun`
    i`clone`
    `(`
    i`self`
    `)`
    I`Bar`
    `[`
    I`T`
    `]`
    `{`
    I`Bar`
    `(`
    `.`
    i`x`
    `.`
    i`clone`
    `,`
    `.`
    i`y`
    `.`
    i`clone`
    `)`
    `}`
    `}`
    EOF
]"#
        )
    }
}
