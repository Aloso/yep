use std::cmp::Ordering;
use std::convert::TryInto;
use std::fmt;
use std::ops::{Index, Range};

use crate::Spanned;


#[derive(Clone, Copy, PartialEq, Eq, Default, Hash)]
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

    pub fn merge_if<T>(&self, other: &Option<Spanned<T>>) -> Self {
        match other {
            Some(t) => self.merge(t.span),
            None => *self,
        }
    }

    pub fn embed<T>(self, inner: T) -> Spanned<T> { Spanned::new(inner, self) }
}

impl fmt::Debug for TextRange {
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
