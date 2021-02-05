mod error;
mod keyword;
mod literal;
mod name;
mod punct;
mod spanned;
mod text_range;

pub mod expr;
pub mod item;
pub mod pattern;
pub mod token;

pub use error::LexError;
pub use spanned::Spanned;
pub use text_range::TextRange;

// TODO: Use something like smallvec or tinyvec instead
pub type SpannedList<T> = Box<[Spanned<T>]>;

pub use tinystring::TinyString;
