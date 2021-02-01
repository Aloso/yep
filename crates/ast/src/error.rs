#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LexError {
    Unexpected,
    InvalidNum,
    NoWS,
    WS,
}
