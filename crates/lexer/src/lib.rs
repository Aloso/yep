mod numbers;
mod syntax;
#[cfg(test)]
mod tests;
mod tokens;

use std::fmt;

use ast::token::Token;
use ast::{LexError, Spanned};

pub fn lex(text: &str) -> Program {
    let tokens = tokens::lex(text);
    Program { tokens }
}

pub struct Program {
    tokens: Vec<Spanned<Token>>,
}

impl Program {
    pub fn token_len(&self) -> usize { self.tokens.len() }

    pub fn tokens(&self) -> &[Spanned<Token>] { &self.tokens }

    pub fn errors(&self) -> Vec<Spanned<LexError>> {
        let mut lex_errors = Vec::new();
        for t in self.tokens() {
            if let Some(e) = t.lex_error() {
                lex_errors.push(t.span.embed(e));
            }
        }
        lex_errors
    }

    pub fn no_eof(&mut self) {
        match self.tokens.pop() {
            Some(t) if *t != Token::Eof => self.tokens.push(t),
            _ => {}
        }
    }
}

impl From<Vec<Spanned<Token>>> for Program {
    fn from(tokens: Vec<Spanned<Token>>) -> Self { Program { tokens } }
}

impl fmt::Debug for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        let multi_line = matches!(f.align(), Some(fmt::Alignment::Left));
        if multi_line {
            f.write_str("\n")?;
        }

        for (i, t) in self.tokens.iter().enumerate() {
            if multi_line {
                writeln!(f, "    {:?}", t)?;
            } else {
                if i != 0 {
                    f.write_str(" ")?;
                }
                write!(f, "{:?}", t)?;
            }
        }

        f.write_str("]")?;
        Ok(())
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, t) in self.tokens.iter().enumerate() {
            if i != 0 {
                f.write_str(" ")?;
            }
            match &**t {
                Token::Punct(p) => write!(f, "{}", p)?,
                Token::StringLit(l) => write!(f, "{}", l)?,
                Token::NumberLit(l) => write!(f, "{}", l)?,
                Token::Ident(i) => write!(f, "{}", i)?,
                Token::UpperIdent(u) => write!(f, "{}", u)?,
                Token::Operator(o) => write!(f, "{}", o)?,
                Token::Keyword(k) => write!(f, "{}", k)?,
                Token::Error(e) => write!(f, "{}", e)?,
                Token::Eof => write!(f, "EOF")?,
            }
        }
        Ok(())
    }
}
