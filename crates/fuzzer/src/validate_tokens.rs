use ast::token::{Punctuation, Token};
use ast::Spanned;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Bracket {
    EOF,
    Round,
    Square,
    Curly,
    Pipe,
}

struct OpenBrackets {
    inner: Vec<Bracket>,
}
impl OpenBrackets {
    fn new() -> Self { OpenBrackets { inner: vec![Bracket::EOF] } }

    fn last(&self) -> Result<Bracket, &'static str> {
        if self.inner.is_empty() {
            Err("no open brackets")
        } else {
            Ok(self.inner[self.inner.len() - 1])
        }
    }

    fn push(&mut self, bracket: Bracket) { self.inner.push(bracket); }

    fn pop(&mut self) { self.inner.pop(); }

    fn pop_exact(&mut self, bracket: Bracket) -> Result<(), &'static str> {
        if self.last()? == bracket {
            self.inner.pop();
            Ok(())
        } else {
            Err("Unexpected closing bracket")
        }
    }

    fn expect_empty(&self) -> Result<(), &'static str> {
        if self.inner.is_empty() {
            Ok(())
        } else {
            Err("remaining open brackets")
        }
    }
}

pub fn is_balanced(tokens: &[Spanned<Token>]) -> Result<(), &'static str> {
    let mut open_brackets = OpenBrackets::new();
    let mut eof = false;

    for token in tokens {
        match &**token {
            Token::Punct(p) => match p {
                Punctuation::Pipe => {
                    if open_brackets.last()? == Bracket::Pipe {
                        open_brackets.pop();
                    } else {
                        open_brackets.push(Bracket::Pipe);
                    }
                }
                Punctuation::OpenParen => open_brackets.push(Bracket::Round),
                Punctuation::CloseParen => open_brackets.pop_exact(Bracket::Round)?,
                Punctuation::OpenBracket => open_brackets.push(Bracket::Square),
                Punctuation::CloseBracket => open_brackets.pop_exact(Bracket::Square)?,
                Punctuation::OpenBrace => open_brackets.push(Bracket::Curly),
                Punctuation::CloseBrace => open_brackets.pop_exact(Bracket::Curly)?,
                _ => {}
            },
            Token::EOF => {
                if eof {
                    return Err("token after EOF");
                } else {
                    eof = true;
                }
                open_brackets.pop_exact(Bracket::EOF)?;
            }
            _ => {}
        }
    }
    open_brackets.expect_empty()
}
