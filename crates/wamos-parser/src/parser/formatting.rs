use string_interner::StringInterner;

fn do_indent(buf: &mut String, indent: usize) { buf.extend((0..indent).map(|_| ' ')); }


pub trait FancyFormat {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner);

    fn is_single_line(&self) -> bool { false }

    fn is_empty(&self) -> bool { false }

    fn fmt(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        if !self.is_empty() {
            self.fmt_impl(buf, indent, interner);
        }
    }

    fn to_string(&self, interner: &StringInterner) -> String {
        let mut buf = String::new();
        self.fmt(&mut buf, 0, interner);
        buf
    }
}

impl<T: FancyFormat + ?Sized> FancyFormat for &T {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        (*self).fmt_impl(buf, indent, interner)
    }

    fn is_single_line(&self) -> bool { (*self).is_single_line() }

    fn is_empty(&self) -> bool { (*self).is_empty() }

    fn fmt(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        (*self).fmt(buf, indent, interner)
    }
}

impl FancyFormat for &'_ str {
    fn fmt_impl(&self, buf: &mut String, _indent: usize, _interner: &StringInterner) {
        buf.push_str(self)
    }

    fn is_single_line(&self) -> bool { true }

    fn is_empty(&self) -> bool { false }
}

/// Value, transformation, is_single_line
pub(crate) struct FancyWrap<T, F>(pub T, pub F, pub bool);


impl<T: Copy, U: FancyFormat, F: Fn(T) -> U> FancyFormat for FancyWrap<T, F> {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        let u = self.1(self.0);
        u.fmt_impl(buf, indent, interner);
        if u.is_single_line() && !self.is_single_line() {
            buf.push('\n');
        }
    }

    fn is_single_line(&self) -> bool { self.2 }

    fn is_empty(&self) -> bool { self.1(self.0).is_empty() }
}

pub(crate) struct FancyList<'a, T>(pub &'a [T]);

impl<T: FancyFormat> FancyFormat for FancyList<'_, T> {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        if self.is_single_line() {
            self.0[0].fmt(buf, indent, interner);
        } else {
            for (i, x) in self.0.iter().filter(|&x| !x.is_empty()).enumerate() {
                if i > 0 {
                    do_indent(buf, indent);
                }
                x.fmt(buf, indent, interner);
                if x.is_single_line() {
                    buf.push('\n');
                }
            }
        }
    }

    fn is_empty(&self) -> bool { self.0.is_empty() }

    fn is_single_line(&self) -> bool { self.0.len() == 1 && self.0[0].is_single_line() }
}

impl<T: FancyFormat> FancyFormat for Vec<T> {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        FancyList(self.as_slice()).fmt_impl(buf, indent, interner)
    }

    fn is_single_line(&self) -> bool { self.len() == 1 && self[0].is_single_line() }

    fn is_empty(&self) -> bool { self.is_empty() }
}

pub(crate) struct FancyKV<K: FancyFormat, V: FancyFormat>(pub K, pub V);

impl<K: FancyFormat, V: FancyFormat> FancyFormat for FancyKV<K, V> {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        if self.is_single_line() {
            self.0.fmt(buf, indent, interner);
            buf.push_str(": ");
            self.1.fmt(buf, indent, interner);
        } else {
            self.0.fmt(buf, indent, interner);
            if self.0.is_single_line() {
                buf.push('\n');
            }
            do_indent(buf, indent + 3);
            self.1.fmt(buf, indent + 3, interner);
            if self.1.is_single_line() {
                buf.push('\n');
            }
        }
    }

    fn is_single_line(&self) -> bool { self.0.is_single_line() && self.1.is_single_line() }

    fn is_empty(&self) -> bool { self.0.is_single_line() && self.1.is_empty() }
}

impl<T: FancyFormat> FancyFormat for Option<T> {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        match self {
            Some(v) => v.fmt_impl(buf, indent, interner),
            None => {}
        }
    }

    fn is_single_line(&self) -> bool {
        match self {
            Some(v) => v.is_single_line(),
            None => false,
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Some(v) => v.is_empty(),
            None => true,
        }
    }
}

impl<T: FancyFormat> FancyFormat for Box<T> {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        (**self).fmt_impl(buf, indent, interner)
    }

    fn is_single_line(&self) -> bool { (**self).is_single_line() }

    fn is_empty(&self) -> bool { (**self).is_empty() }

    fn fmt(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        (**self).fmt(buf, indent, interner)
    }
}

pub(crate) fn dyn_list<'a>(items: &'a [&'a dyn FancyFormat]) -> FancyList<&'a dyn FancyFormat> {
    FancyList(items)
}

#[macro_export]
macro_rules! key_values {
    ($name:literal { $( $e:expr ),* $(,)? }) => {
        $crate::parser::formatting::FancyKV(
            $name,
            $crate::parser::formatting::dyn_list(&[ $( &$e ),* ]),
        )
    };
}

#[test]
fn test_formatting() {
    fn test<T: FancyFormat>(s: T, expected: &'static str) {
        let interner = StringInterner::new();
        let mut buf = String::new();
        s.fmt(&mut buf, 0, &interner);
        assert_eq!(buf.as_str(), expected)
    }

    let short_list = FancyList(&["A", "B"]);
    let short_list2 = FancyList(&["C", "D"]);
    let short_list3 = FancyList(&["E"]);
    let empty_list = FancyList::<&str>(&[]);

    test(FancyKV("Foo", "Bar"), "Foo: Bar");
    test(
        FancyKV("Foo", FancyWrap("  Bar", str::trim, false)),
        "Foo\n   Bar\n",
    );
    test(FancyKV("Foo", empty_list), "");
    test(FancyKV("Foo", &short_list), "Foo\n   A\n   B\n");
    test(FancyKV("Foo", &short_list3), "Foo: E");
    test(
        FancyKV("Foo", dyn_list(&[&"A", &FancyKV("Bar", short_list2)])),
        "Foo\n   A\n   Bar\n      C\n      D\n",
    );
    test(
        FancyKV("Foo", dyn_list(&[&"A", &FancyKV("Bar", &short_list3)])),
        "Foo\n   A\n   Bar: E\n",
    );
    test(
        FancyKV("Foo", FancyKV("Bar", FancyKV("Baz", short_list))),
        "Foo\n   Bar\n      Baz\n         A\n         B\n",
    );
    test(
        FancyKV("Foo", FancyKV("Bar", FancyKV("Baz", short_list3))),
        "Foo: Bar: Baz: E",
    );
}
