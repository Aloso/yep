#[derive(thiserror::Error, Debug, Copy, Clone, PartialEq, Eq)]
pub enum LexError {
    #[error("Unexpected token")]
    Unexpected,
    #[error("No whitespace")]
    NoWS,
    #[error("Whitespace")]
    WS,

    #[error("Invalid number token")]
    InvalidNum,
    #[error("Number too large")]
    NumberOverflow,
    #[error("Invalid char {0:?} in number literal")]
    InvalidCharInNum(char),
}
