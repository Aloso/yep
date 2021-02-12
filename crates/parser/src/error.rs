use ast::expr::Expr;
use ast::token::{Operator, Token};
use ast::Spanned;

use crate::validation::ValidationError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("There are remaining tokens that could not be parsed: {0:?}")]
    RemainingTokens(Vec<Spanned<Token>>),

    #[error("Expected {0}")]
    Expected(&'static str),

    #[error("Expected {0:?}, got {1:?}")]
    ExpectedGot(Token, Token),

    #[error("Expected {0}, got {1:?}")]
    ExpectedGot2(&'static str, Token),

    #[error("Expected {0}, got {1:?}")]
    ExpectedGot3(&'static str, Expr),

    #[error("Expected {0}, got {1}")]
    ExpectedGot4(&'static str, &'static str),

    #[error(
        "Operators are not allowed here: {0:?}\n  tip: Wrap the operator in braces, \
         e.g. `{{+}}`"
    )]
    OperatorInsteadOfOperand(Operator),

    #[error("{0}")]
    ValidationError(#[from] ValidationError),
}
