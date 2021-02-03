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
    pub fn token_len(&self) -> usize { self.tokens.len() }

    pub fn tokens(&'a self) -> &'a [Spanned<Token<'a>>] { &self.tokens }

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

#[test]
fn run_test_files() {
    use std::ffi::OsStr;
    use std::fs::{read_to_string, File};
    use std::io::Write;

    for file in std::fs::read_dir("./tests").unwrap() {
        let path = file.unwrap().path();
        if path.is_file() && path.extension() == Some(OsStr::new("wa")) {
            let content: String = read_to_string(&path).unwrap();
            let content = content.trim_end();

            let p = crate::lex(&content);
            let formatted = format!("{:<#?}", p);

            let tokens_path = path.with_extension("tokens");
            if tokens_path.exists() {
                let expected: String = read_to_string(tokens_path).unwrap();
                let expected = expected.trim_end();

                if expected != formatted {
                    let changes = difference::Changeset::new(expected, &formatted, "\n");
                    eprintln!("{}", changes);
                    eprintln!("Input:\n{}", content);
                    panic!(
                        "{} differences between expected and actual output",
                        changes.distance
                    );
                }
            } else {
                let mut file = File::create(tokens_path).unwrap();
                file.write_all(formatted.as_bytes()).unwrap();
                file.write_all(b"\n").unwrap();
                file.flush().unwrap();
            }
        }
    }
}
