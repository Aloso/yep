use std::fmt;
use std::ops::Deref;

use crate::TextRange;

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct Spanned<T> {
    pub inner: T,
    pub span: TextRange,
}

impl<T> Spanned<T> {
    pub fn new(inner: T, span: TextRange) -> Self { Self { inner, span } }

    pub fn into_inner(self) -> (T, TextRange) { (self.inner, self.span) }

    pub fn map<F: FnMut(T) -> U, U>(self, mut f: F) -> Spanned<U> {
        Spanned::new(f(self.inner), self.span)
    }

    pub fn map_ref<F: FnMut(&T) -> U, U>(&self, mut f: F) -> Spanned<U> {
        Spanned::new(f(&self.inner), self.span)
    }
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
