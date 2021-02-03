mod numbers;
mod syntax;
mod tokens;

use std::fmt;
use string_interner::StringInterner;

use ast::token::{Token, TokenData};
use ast::{LexError, Spanned};

pub fn lex(text: &str) -> Program<'_> {
    let mut interner = StringInterner::new();
    let tokens = tokens::lex(text, &mut interner);
    Program { text, tokens }
}

pub fn lex_with_interner<'a>(
    text: &'a str,
    interner: &'a mut StringInterner,
) -> Program<'a> {
    let tokens = tokens::lex(text, interner);
    Program { text, tokens }
}

pub struct Program<'a> {
    text: &'a str,
    tokens: Vec<Spanned<Token<'a>>>,
}

impl<'a> Program<'a> {
    pub fn token_len(&self) -> usize {
        self.tokens.len()
    }

    pub fn tokens(&'a self) -> &'a [Spanned<Token<'a>>] {
        &self.tokens
    }

    pub fn text(&self) -> &str {
        self.text
    }

    pub fn errors(&self) -> Vec<LexError> {
        let mut lex_errors = Vec::new();
        for t in self.tokens() {
            if let Some(e) = t.lex_error() {
                lex_errors.push(e);
            }
        }
        lex_errors
    }

    pub fn with_tokens(&'a self, tokens: &'a [Spanned<Token<'a>>]) -> Self {
        Program { tokens: tokens.to_vec(), text: self.text }
    }

    pub fn with_lifeless_tokens(&'a self, tokens: &'a [Spanned<TokenData>]) -> Self {
        Program {
            tokens: tokens.iter().map(|l| Token::new(l.inner, l.span)).collect(),
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
    i`hello` @ 0..5
    `,` @ 5..6
    i`world!` @ 7..13
    EOF @ 13..13
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
    k`fun` @ 34..37
    i`foo` @ 38..41
    `(` @ 41..42
    `)` @ 42..43
    I`Bool` @ 44..48
    `{` @ 49..50
    k`let` @ 55..58
    i`x-y` @ 59..62
    `=` @ 63..64
    Int(5)@`5` @ 65..66
    I`Int` @ 67..70
    `;` @ 70..71
    k`var` @ 76..79
    i`test!` @ 80..85
    `=` @ 86..87
    `\` @ 88..89
    i`x-y` @ 90..93
    o`>gt` @ 94..97
    Int(42)@`42` @ 98..100
    `;` @ 100..101
    i`test!` @ 106..111
    `(` @ 111..112
    `)` @ 112..113
    `}` @ 114..115
    EOF @ 115..115
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
    k`class` @ 0..5
    I`Foo` @ 6..9
    `(` @ 9..10
    i`x` @ 10..11
    I`Int` @ 12..15
    `,` @ 15..16
    i`y` @ 17..18
    I`Int` @ 19..22
    `)` @ 22..23
    k`class` @ 37..42
    I`Bar` @ 43..46
    `[` @ 46..47
    I`T` @ 47..48
    `]` @ 48..49
    `(` @ 49..50
    i`x` @ 50..51
    I`T` @ 52..53
    `,` @ 53..54
    i`y` @ 55..56
    I`T` @ 57..58
    `)` @ 58..59
    k`impl` @ 73..77
    `[` @ 77..78
    I`T` @ 78..79
    I`Clone` @ 80..85
    `]` @ 85..86
    I`Clone` @ 87..92
    k`for` @ 93..96
    I`Bar` @ 97..100
    `[` @ 100..101
    I`T` @ 101..102
    `]` @ 102..103
    `{` @ 104..105
    k`fun` @ 122..125
    i`clone` @ 126..131
    `(` @ 131..132
    i`self` @ 132..136
    `)` @ 136..137
    I`Bar` @ 138..141
    `[` @ 141..142
    I`T` @ 142..143
    `]` @ 143..144
    `{` @ 145..146
    I`Bar` @ 167..170
    `(` @ 170..171
    `.` @ 171..172
    i`x` @ 172..173
    `.` @ 173..174
    i`clone` @ 174..179
    `,` @ 179..180
    `.` @ 181..182
    i`y` @ 182..183
    `.` @ 183..184
    i`clone` @ 184..189
    `)` @ 189..190
    `}` @ 207..208
    `}` @ 221..222
    EOF @ 222..222
]"#
        )
    }
}
