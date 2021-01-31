use anyhow::{Context, Result};
use std::borrow::Cow;

use super::tokens::{LexError, TokenData};


/// Supported literals are
///
/// * Signed integer (Int)
/// * Unsigned integer (UInt)
/// * Float (Number)
///
/// # Grammar
///
/// ```no_test
/// SIGN  := '+' | '-'
/// E     := 'e' | 'E'
///
/// BIN_DIGIT := '0' | '1'
/// OCT_DIGIT := '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7'
/// DEC_DIGIT := OCT_DIGIT | '8' | '9'
/// HEX_DIGIT := DEC_DIGIT | 'a' | 'b' | 'c' | 'd' | 'e' | 'f'
///                        | 'A' | 'B' | 'C' | 'D' | 'E' | 'F'
///
/// BIN_SEQUENCE := BIN_DIGIT (BIN_DIGIT | '_')*
/// OCT_SEQUENCE := OCT_DIGIT (OCT_DIGIT | '_')*
/// DEC_SEQUENCE := DEC_DIGIT (DEC_DIGIT | '_')*
/// HEX_SEQUENCE := HEX_DIGIT (HEX_DIGIT | '_')*
///
/// BINARY      := SIGN? '0b' BIN_SEQUENCE
/// OCTAL       := SIGN? '0o' OCT_SEQUENCE
/// HEXADECIMAL := SIGN? '0x' HEX_SEQUENCE
/// DECIMAL     := SIGN? DEC_SEQUENCE
///
/// EXPONENT    := E SIGN? DEC_SEQUENCE
/// FLOAT       := SIGN? DEC_SEQUENCE '.' DEC_SEQUENCE EXPONENT?
///              | SIGN? DEC_SEQUENCE EXPONENT
///              | '.' DEC_SEQUENCE EXPONENT?
/// ```
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NumberLiteral {
    Int(i64),
    UInt(u64),
    Float(f64),
}

trait Int: Copy + 'static {
    fn zero() -> Self;
    fn mul(self, factor: u32) -> Result<Self>;
    fn add(self, summand: u32) -> Result<Self>;
    fn sub(self, summand: u32) -> Result<Self>;
}

macro_rules! impl_int {
    ($( $t:ty )*) => {
        $(
            impl Int for $t {
                #[inline(always)]
                fn zero() -> Self { 0 }

                #[inline(always)]
                fn mul(self, factor: u32) -> Result<Self> {
                    self.checked_mul(factor as Self).context("Number overflowed")
                }

                #[inline(always)]
                fn add(self, summand: u32) -> Result<Self> {
                    self.checked_add(summand as Self).context("Number overflowed")
                }

                #[inline(always)]
                fn sub(self, summand: u32) -> Result<Self> {
                    self.checked_sub(summand as Self).context("Number overflowed")
                }
            }
        )*
    }
}

impl_int!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128);

fn parse_int_digits<N: Int>(negative: bool, text: &str, radix: u32) -> Result<N> {
    let chars = text.chars().filter(|&c| c != '_').map(|c| {
        c.to_digit(radix).with_context(|| format!("Illegal char {:?} in number", c))
    });

    let mut num = N::zero();
    if negative {
        for digit in chars {
            num = num.mul(radix)?.sub(digit?)?;
        }
    } else {
        for digit in chars {
            num = num.mul(radix)?.add(digit?)?;
        }
    }
    Ok(num)
}

fn parse_exp(text: &str) -> Result<i32, ()> {
    Ok(match text.chars().next() {
        Some('+') => parse_int_digits(false, &text[1..], 10).map_err(|_| ())?,
        Some('-') => parse_int_digits(true, &text[1..], 10).map_err(|_| ())?,
        _ => parse_int_digits(false, text, 10).map_err(|_| ())?,
    })
}

fn parse_at_dot(text: &str) -> Result<f64, ()> {
    let text = if text.contains('_') {
        Cow::Owned(text.chars().filter(|&c| c != '_').collect())
    } else {
        Cow::Borrowed(text)
    };
    text.parse().map_err(|_| ())
}

pub(crate) fn leading_dot(input: &str) -> Result<NumberLiteral, ()> {
    let exp = input.find(|c: char| c == 'e' || c == 'E');

    let num = if let Some(exp_index) = exp {
        let exp = parse_exp(&input[exp_index + 1..])?;
        let num = parse_at_dot(&input[..exp_index])?;
        num * 10f64.powi(exp)
    } else {
        parse_at_dot(input)?
    };
    if num.is_finite() {
        Ok(NumberLiteral::Float(num))
    } else {
        Err(())
    }
}

pub(crate) fn float(input: &str) -> Result<NumberLiteral, ()> {
    let input = input.trim_end_matches('_');
    if input.ends_with(|c: char| c == 'e' || c == 'E' || c == '.') {
        return Err(());
    }
    let exp = input.find(|c: char| c == 'e' || c == 'E');
    let num: f64 = if let Some(exp_index) = exp {
        let exp = parse_exp(&input[exp_index + 1..])?;
        let num: String = input[..exp_index].chars().filter(|&c| c != '_').collect();
        format!("{}e{}", num, exp).parse().map_err(|_| ())?
    } else {
        let num: String = input.chars().filter(|&c| c != '_').collect();
        num.parse().map_err(|_| ())?
    };
    if num.is_finite() {
        Ok(NumberLiteral::Float(num))
    } else {
        Err(())
    }
}

fn int_with_radix(
    input: &str,
    radix_width: usize,
    radix: u32,
) -> Result<NumberLiteral, ()> {
    Ok(match input.chars().next() {
        Some('-') => {
            let text = input[radix_width + 1..].trim_start_matches('_');
            if text.is_empty() {
                return Err(());
            }
            NumberLiteral::Int(parse_int_digits(true, text, radix).map_err(|_| ())?)
        }
        Some('+') => {
            let text = input[radix_width + 1..].trim_start_matches('_');
            if text.is_empty() {
                return Err(());
            }
            NumberLiteral::UInt(parse_int_digits(false, text, radix).map_err(|_| ())?)
        }
        _ => {
            let text = input[radix_width..].trim_start_matches('_');
            if text.is_empty() {
                return Err(());
            }
            NumberLiteral::Int(parse_int_digits(false, text, radix).map_err(|_| ())?)
        }
    })
}

pub(crate) fn hex(input: &str) -> Result<NumberLiteral, ()> {
    int_with_radix(input, 2, 16)
}

pub(crate) fn oct(input: &str) -> Result<NumberLiteral, ()> {
    int_with_radix(input, 2, 8)
}

pub(crate) fn bin(input: &str) -> Result<NumberLiteral, ()> {
    int_with_radix(input, 2, 2)
}

pub(crate) fn dec(input: &str) -> Result<NumberLiteral, ()> {
    int_with_radix(input, 0, 10)
}

pub(super) fn parse_number(input: &str) -> TokenData {
    if input.starts_with('.') {
        leading_dot(input)
            .map(TokenData::NumberLit)
            .unwrap_or(TokenData::Error(LexError::InvalidNum))
    } else {
        let without_sign =
            input.strip_prefix(|c: char| c == '+' || c == '-').unwrap_or(input);
        if without_sign.starts_with('0') {
            if let Some(x) = without_sign.chars().nth(1) {
                match x {
                    'x' | 'X' => {
                        return hex(input)
                            .map(TokenData::NumberLit)
                            .unwrap_or(TokenData::Error(LexError::InvalidNum))
                    }
                    'b' | 'B' => {
                        return bin(input)
                            .map(TokenData::NumberLit)
                            .unwrap_or(TokenData::Error(LexError::InvalidNum))
                    }
                    'o' | 'O' => {
                        return oct(input)
                            .map(TokenData::NumberLit)
                            .unwrap_or(TokenData::Error(LexError::InvalidNum))
                    }
                    _ => {}
                }
            }
        }
        if without_sign.contains(|c: char| c == '.' || c == 'e' || c == 'E') {
            float(input)
                .map(TokenData::NumberLit)
                .unwrap_or(TokenData::Error(LexError::InvalidNum))
        } else {
            dec(input)
                .map(TokenData::NumberLit)
                .unwrap_or(TokenData::Error(LexError::InvalidNum))
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::lexer;
    use crate::lexer::tokens::TokenData;

    use super::NumberLiteral::{self as Num, *};

    use anyhow::{bail, Result};
    use assert_matches::assert_matches;
    use std::str::FromStr;

    impl FromStr for Num {
        type Err = anyhow::Error;
        fn from_str(text: &str) -> Result<Self> {
            let mut program = lexer::lex(text);
            program.no_eof();
            if program.token_len() != 1 {
                bail!("expected exactly 1 token, got {:#?}", program);
            }
            match program.tokens()[0].data() {
                TokenData::NumberLit(lit) => Ok(lit),
                _ => bail!("expected number, got {:#?}", program),
            }
        }
    }

    macro_rules! assert_ok {
        ($s:literal, $p:pat $(if $e:expr)? $(,)?) => {
            match $s.parse::<Num>() {
                Ok($p) $(if $e)? => {}
                Err(e) => panic!("{}", e),
                p => panic!(
                    "Assertion failed:\n  \
                    expected: {}\n  \
                    got:      {:?}",
                    stringify!($p $(if $e)?), p
                ),
            }
        };
    }

    macro_rules! assert_err {
        ($s:literal, $e:expr $(,)?) => {
            match $s.parse::<Num>() {
                Ok(v) => panic!(
                    "Assertion failed:\n  \
                    expected: Err({})\n  \
                    got:      {:?}",
                    stringify!($e), v
                ),
                Err(s) if &ToString::to_string(&s) == $e => {},
                Err(s) => panic!(
                    "Assertion failed:\n  \
                    expected: Err({})\n  \
                    got:      Err({})",
                    stringify!($e), s
                ),
            }
            assert_matches!($s.parse::<Num>(), Err(s) if &ToString::to_string(&s) == $e);
        };
    }

    fn approx(left: f64, right: f64) -> bool {
        (left / right - 1.0).abs() < 0.000000000001
    }

    #[test]
    fn decimal_ints() {
        assert_ok!("0", Int(0));
        assert_ok!("-0", Int(0));
        assert_ok!("+0", UInt(0));
        assert_ok!("00010", Int(10));
        assert_ok!("100", Int(100));
        assert_ok!("+123", UInt(123));
        assert_ok!("-234", Int(-234));
        assert_ok!("1_2__34_", Int(1234));
        assert_ok!("+18_446_744_073_709_551_615", UInt(u64::MAX),);
    }

    #[test]
    fn bin_oct_hex_ints() {
        assert_ok!("0b0", Int(0));
        assert_ok!("+0o0", UInt(0));
        assert_ok!("-0x0", Int(0));
        assert_ok!("0xFFEF", Int(0xFFEF));
        assert_ok!("-0xffef", Int(-0xffef));
        assert_ok!("0b10101010", Int(0b10101010));
        assert_ok!("+0o2575751", UInt(0o2575751));
    }

    #[test]
    fn floats() {
        assert_ok!(".0", Float(n) if n == 0.0);
        assert_ok!("0.0", Float(n) if n == 0.0);
        assert_ok!(".0e0", Float(n) if n == 0.0);
        assert_ok!(".0E1", Float(n) if n == 0.0e1);
        assert_ok!("0.0e0", Float(n) if n == 0.0);
        assert_ok!("0.0_1", Float(n) if approx(n, 0.01));
        assert_ok!("+2.2e2", Float(n) if approx(n, 2.2e2));
        assert_ok!("-2.2e2", Float(n) if approx(n, -2.2e2));
        assert_ok!("2_.2_e2_", Float(n) if approx(n, 2.2e2));
        assert_ok!("12345.12345E234", Float(n) if approx(n, 12345.12345e234));
        assert_ok!(".12345e234", Float(n) if approx(n, 0.12345e234));
    }

    #[test]
    fn invalid_integers() {
        assert_err!("+-0", "expected number, got [InvalidNum@`+-0` @ 0..3]");
        assert_err!("--0", "expected number, got [InvalidNum@`--0` @ 0..3]");
        assert_err!("", "expected exactly 1 token, got []");
        assert_err!("+", "expected number, got [o`+` @ 0..1]");
        assert_err!("_", "expected number, got [`_` @ 0..1]");
        assert_err!("0x_", "expected number, got [InvalidNum@`0x_` @ 0..3]");
        assert_err!("0xF_G", "expected number, got [InvalidNum@`0xF_G` @ 0..5]");
        assert_err!("0b012", "expected number, got [InvalidNum@`0b012` @ 0..5]");
    }

    #[test]
    fn invalid_floats() {
        assert_err!(".", "expected number, got [`.` @ 0..1]");
        assert_err!(
            "._1",
            "expected exactly 1 token, got [`.` @ 0..1 InvalidNum@`_1` @ 1..3]"
        );
        assert_err!(
            "_.1",
            "expected exactly 1 token, got [`_` @ 0..1 Float(0.1)@`.1` @ 1..3]"
        );
        assert_err!("-.1", "expected number, got [NoWS@`-.1` @ 0..3]");
        assert_err!("1e", "expected number, got [InvalidNum@`1e` @ 0..2]");
        assert_err!("1e__", "expected number, got [InvalidNum@`1e__` @ 0..4]");
        assert_err!("1e_+1", "expected number, got [InvalidNum@`1e_+1` @ 0..5]");
        assert_err!(
            "0._1",
            "expected exactly 1 token, got [Int(0)@`0` @ 0..1 `.` @ 1..2 \
             InvalidNum@`_1` @ 2..4]"
        );
        assert_err!(
            ".12345e2345",
            "expected number, got [InvalidNum@`.12345e2345` @ 0..11]",
        );
        assert_err!("0f.1", "expected number, got [InvalidNum@`0f.1` @ 0..4]");
    }
}
