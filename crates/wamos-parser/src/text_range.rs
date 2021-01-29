use std::ops::{Deref, Index, Range};
use std::{cmp::Ordering, convert::TryInto, fmt};

use fmt::Debug;

use crate::parser::expr::Expr;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct TextRange {
    start: u32,
    end: u32,
}

impl TextRange {
    pub fn new(start: u32, end: u32) -> Self {
        assert!(start <= end);
        TextRange { start, end }
    }

    pub fn start(&self) -> u32 { self.start }

    pub fn end(&self) -> u32 { self.end }

    pub fn extend_until(&self, end: u32) -> Self { TextRange::new(self.start, end) }

    #[must_use]
    pub fn merge(&self, other: Self) -> Self {
        TextRange::new(self.start.min(other.start), self.end.max(other.end))
    }

    pub fn merge_if<T>(&self, other: &Option<(T, Self)>) -> Self {
        match other {
            Some((_, other)) => self.merge(*other),
            None => *self,
        }
    }

    pub fn merge_if_expr(&self, other: &Option<Expr>) -> Self {
        match other {
            Some(e) => self.merge(e.span()),
            None => *self,
        }
    }
}

impl fmt::Display for TextRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl From<Range<usize>> for TextRange {
    fn from(r: Range<usize>) -> Self {
        TextRange::new(r.start.try_into().unwrap(), r.end.try_into().unwrap())
    }
}

impl From<TextRange> for Range<usize> {
    fn from(r: TextRange) -> Self {
        Range { start: r.start.try_into().unwrap(), end: r.end.try_into().unwrap() }
    }
}

impl PartialOrd for TextRange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self.end <= other.start {
            Some(Ordering::Less)
        } else if self.start >= other.end {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
}

impl Index<TextRange> for str {
    type Output = str;

    fn index(&self, index: TextRange) -> &Self::Output {
        &self[index.start as usize..index.end as usize]
    }
}


#[derive(Copy, Clone)]
pub struct Spanned<T> {
    inner: T,
    span: TextRange,
}

impl<T> Spanned<T> {
    pub fn new(inner: T, span: TextRange) -> Self { Self { inner, span } }

    pub fn span(&self) -> TextRange { self.span }

    pub fn inner(&self) -> &T { &self.inner }

    pub fn into_inner(self) -> (T, TextRange) { (self.inner, self.span) }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target { &self.inner }
}

impl<T: fmt::Debug> fmt::Debug for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)?;
        write!(f, " @ ")?;
        fmt::Debug::fmt(&self.span, f)
    }
}

impl<T> From<(T, TextRange)> for Spanned<T> {
    fn from((inner, span): (T, TextRange)) -> Self { Self { inner, span } }
}
