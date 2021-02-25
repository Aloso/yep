#[derive(thiserror::Error, Debug, Copy, Clone, PartialEq, Eq)]
pub enum LexError {
    #[error("Unexpected token")]
    Unexpected,
    #[error("Missing whitespace")]
    NoWs,
    #[error("Unexpected whitespace")]
    Ws,

    #[error("Invalid number token")]
    InvalidNum,
    #[error("Number too large")]
    NumberOverflow,
    #[error("Invalid char {0:?} in number literal")]
    InvalidCharInNum(char),
}

#[cfg(feature = "fuzz")]
impl arbitrary::Arbitrary for LexError {
    fn arbitrary(_: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        Ok(LexError::Ws)
    }
}
