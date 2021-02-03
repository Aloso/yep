use ast::token::{NumberLiteral, StringLiteral};
use ast::{DefaultSymbol, Spanned, StringInterner};

#[derive(Debug, Clone)]
pub struct Beauty {
    pub(super) data: BeautyData,
    // 0, 1 or many
    pub(super) num: u8,
}

impl Beauty {
    pub(super) fn kv(key: &'static str, value: Beauty) -> Self {
        let num = value.num;
        let data = BeautyData::KV { key, value: Box::new(value) };
        Beauty { data, num }
    }

    pub(super) fn kvs(key: &'static str, values: Vec<Beauty>) -> Self {
        let value = Beauty::list(values);
        let num = value.num;
        let data = BeautyData::KV { key, value: Box::new(value) };
        Beauty { data, num }
    }

    pub(super) fn list(values: Vec<Beauty>) -> Self {
        let mut num = 0;
        for b in &values {
            num += b.num;
            if num > 1 {
                break;
            }
        }
        Beauty { data: BeautyData::List(values), num }
    }
}

#[derive(Debug, Clone)]
pub(super) enum BeautyData {
    List(Vec<Beauty>),
    Str(&'static str),
    String(StringLiteral),
    Number(NumberLiteral),
    Interned(DefaultSymbol),
    KV { key: &'static str, value: Box<Beauty> },
    Empty,
}


pub trait ToBeauty {
    fn to_beauty(&self) -> Beauty;

    fn to_beauty_string(&self, interner: &StringInterner) -> String {
        fn do_indent(buf: &mut String, indent: u32) {
            buf.extend((0..indent).map(|_| ' '));
        }

        fn to_beauty_string(
            b: &Beauty,
            buf: &mut String,
            indent: u32,
            interner: &StringInterner,
        ) {
            if b.num == 0 {
                return;
            }
            match &b.data {
                BeautyData::List(l) => {
                    if b.num == 1 {
                        let v = l.iter().find(|&x| x.num > 0).unwrap();
                        to_beauty_string(v, buf, indent, interner);
                    } else {
                        for (i, x) in l.iter().filter(|&x| x.num > 0).enumerate() {
                            if i > 0 {
                                do_indent(buf, indent);
                            }
                            to_beauty_string(x, buf, indent, interner);
                            if x.num == 1 {
                                buf.push('\n');
                            }
                        }
                    }
                }
                BeautyData::Str(s) => buf.push_str(s),
                BeautyData::String(s) => {
                    buf.push_str("StringLiteral: ");
                    buf.push_str(interner.resolve(s.symbol()).unwrap());
                }
                BeautyData::Number(n) => match n {
                    NumberLiteral::Int(x) => buf.push_str(&format!("Int: {}", x)),
                    NumberLiteral::UInt(x) => buf.push_str(&format!("UInt: {}", x)),
                    NumberLiteral::Float(x) => buf.push_str(&format!("Float: {}", x)),
                },
                BeautyData::Interned(i) => buf.push_str(interner.resolve(*i).unwrap()),
                BeautyData::KV { key, value } => {
                    if b.num == 1 {
                        buf.push_str(key);
                        buf.push_str(": ");
                        to_beauty_string(value, buf, indent, interner);
                    } else {
                        buf.push_str(key);
                        buf.push('\n');
                        do_indent(buf, indent + 3);
                        to_beauty_string(value, buf, indent + 3, interner);
                        if value.num == 1 {
                            buf.push('\n');
                        }
                    }
                }
                BeautyData::Empty => {}
            }
        }

        let mut buf = String::new();
        to_beauty_string(&self.to_beauty(), &mut buf, 0, interner);
        buf
    }
}


impl ToBeauty for &'static str {
    fn to_beauty(&self) -> Beauty { Beauty { data: BeautyData::Str(*self), num: 1 } }
}

impl<'a, T: ToBeauty + ?Sized> From<&'a T> for Beauty {
    fn from(f: &T) -> Self { f.to_beauty() }
}

impl<T: ToBeauty> ToBeauty for Spanned<T> {
    fn to_beauty(&self) -> Beauty { self.inner.to_beauty() }
}

impl<T: ToBeauty + ?Sized> ToBeauty for Box<T> {
    fn to_beauty(&self) -> Beauty { (&**self).to_beauty() }
}

impl<T: ToBeauty> ToBeauty for [T] {
    fn to_beauty(&self) -> Beauty {
        Beauty::list(self.iter().map(ToBeauty::to_beauty).collect())
    }
}

impl<T: ToBeauty> ToBeauty for Vec<T> {
    fn to_beauty(&self) -> Beauty {
        Beauty::list(self.iter().map(ToBeauty::to_beauty).collect())
    }
}

impl<'a, T: ToBeauty + ?Sized> ToBeauty for &'a T {
    fn to_beauty(&self) -> Beauty { (*self).to_beauty() }
}

impl ToBeauty for bool {
    fn to_beauty(&self) -> Beauty {
        match *self {
            true => "true".to_beauty(),
            false => "false".to_beauty(),
        }
    }
}

impl From<Vec<Beauty>> for Beauty {
    fn from(x: Vec<Beauty>) -> Self {
        let mut num = 0;
        for b in &x {
            num += b.num;
            if num > 1 {
                break;
            }
        }
        Beauty { data: BeautyData::List(x), num }
    }
}

impl<T: ToBeauty> ToBeauty for Option<T> {
    fn to_beauty(&self) -> Beauty {
        match self {
            Some(x) => x.to_beauty(),
            None => Beauty { data: BeautyData::Empty, num: 0 },
        }
    }
}


#[test]
fn test_formatting() {
    struct KV(&'static str, Beauty);

    impl ToBeauty for KV {
        fn to_beauty(&self) -> Beauty { Beauty::kv(self.0, self.1.clone()) }
    }

    fn kv(k: &'static str, v: impl ToBeauty) -> KV { KV(k, v.to_beauty()) }

    macro_rules! list {
        ($( $it:expr ),* $(,)?) => {
            vec![ $( Box::new($it) as Box<dyn ToBeauty> ),* ]
        }
    }

    macro_rules! test {
        ($s:expr, $expected:expr $(,)?) => {{
            let output = $s.to_beauty_string(&StringInterner::new());
            assert_eq!(output.as_str(), $expected)
        }};
    }

    test!(kv("Foo", "Bar"), "Foo: Bar");
    test!(kv("Foo", &[] as &[bool]), "");
    test!(kv("Foo", list!["A", "B"]), "Foo\n   A\n   B\n");
    test!(kv("Foo", list!["E"]), "Foo: E");
    test!(
        kv("Foo", list!["A", kv("Bar", list!["C", "D"])]),
        "Foo\n   A\n   Bar\n      C\n      D\n",
    );
    test!(kv("Foo", list!["A", kv("Bar", list!["E"])]), "Foo\n   A\n   Bar: E\n");
    test!(
        kv("Foo", kv("Bar", kv("Baz", list!["A", "B"]))),
        "Foo\n   Bar\n      Baz\n         A\n         B\n",
    );
    test!(kv("Foo", kv("Bar", kv("Baz", list!["E"]))), "Foo: Bar: Baz: E");
}
