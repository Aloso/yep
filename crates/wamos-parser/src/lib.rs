#![allow(dead_code)]

pub mod lexer;
pub mod parser;
mod spanned;
mod text_range;

pub use spanned::{Spanned, SpannedList};
pub use string_interner::StringInterner;
pub use text_range::TextRange;
