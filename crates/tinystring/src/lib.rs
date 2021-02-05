use std::hash::Hash;
use std::ops::Deref;
use std::{fmt, str};


/// An immutable string that stored inline on the heap,
/// if it is at most 23 bytes long. Otherwise, it is allocated
/// on the heap.
///
/// ## Example
///
/// ```
/// # use tinystring::TinyString;
/// let short = TinyString::new("hello world!"); // on the stack
/// let long = TinyString::new("This is a really long string"); // on the heap
/// ```
#[derive(Clone)]
pub struct TinyString {
    inner: TinyStringInner,
}

#[derive(Clone)]
enum TinyStringInner {
    /// SAFETY: This array must contain valid UTF-8
    Stack([u8; 22], u8),
    Heap(Box<str>),
}

impl From<&str> for TinyString {
    fn from(s: &str) -> Self {
        let len = s.len();
        let inner = if len <= 22 {
            // SAFETY: The NUL byte is valid UTF-8.
            let mut data = [0; 22];

            // SAFETY: Assuming that `s` is valid UTF-8, `data` is, too
            data[0..len].copy_from_slice(s.as_bytes());

            TinyStringInner::Stack(data, len as u8)
        } else {
            TinyStringInner::Heap(s.to_string().into_boxed_str())
        };
        TinyString { inner }
    }
}

impl Deref for TinyString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match &self.inner {
            TinyStringInner::Stack(s, l) => {
                let slice = &s[0..*l as usize];
                // SAFETY: This is sound, assuming that `TinyStringInner`s safety
                // invariants are upheld
                unsafe { str::from_utf8_unchecked(slice) }
            }
            TinyStringInner::Heap(b) => &**b,
        }
    }
}


impl TinyString {
    /// Borrow the string as a `&str`.
    pub fn as_str(&self) -> &str { &**self }

    /// Create a new `TinyString` from a `&str`
    pub fn new(s: &str) -> Self { TinyString::from(s) }
}

impl Default for TinyString {
    fn default() -> Self { TinyString::from("") }
}

impl AsRef<str> for TinyString {
    fn as_ref(&self) -> &str { &**self }
}

impl From<TinyString> for String {
    fn from(s: TinyString) -> Self {
        match s.inner {
            TinyStringInner::Stack(..) => (*s).to_string(),
            TinyStringInner::Heap(b) => b.into_string(),
        }
    }
}

impl fmt::Display for TinyString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl fmt::Debug for TinyString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl PartialEq for TinyString {
    fn eq(&self, other: &Self) -> bool { **self == **other }
}
impl Eq for TinyString {}

impl PartialOrd for TinyString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (**self).partial_cmp(&**other)
    }
}

impl Ord for TinyString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { (**self).cmp(&**other) }
}

impl Hash for TinyString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { (**self).hash(state) }
}

#[test]
fn test_size() {
    assert_eq!(std::mem::size_of::<TinyString>(), 24);
}
