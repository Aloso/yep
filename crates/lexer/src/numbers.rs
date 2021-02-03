use std::borrow::Cow;

use ast::token::{NumberLiteral, TokenData};
use ast::LexError;

trait Int: Copy + 'static {
    fn zero() -> Self;
    fn mul(self, factor: u32) -> Result<Self, LexError>;
    fn add(self, summand: u32) -> Result<Self, LexError>;
    fn sub(self, summand: u32) -> Result<Self, LexError>;
}

macro_rules! impl_int {
    ($( $t:ty )*) => {
        $(
            impl Int for $t {
                #[inline(always)]
                fn zero() -> Self { 0 }

                #[inline(always)]
                fn mul(self, factor: u32) -> Result<Self, LexError> {
                    self.checked_mul(factor as Self).ok_or(LexError::NumberOverflow)
                }

                #[inline(always)]
                fn add(self, summand: u32) -> Result<Self, LexError> {
                    self.checked_add(summand as Self).ok_or(LexError::NumberOverflow)
                }

                #[inline(always)]
                fn sub(self, summand: u32) -> Result<Self, LexError> {
                    self.checked_sub(summand as Self).ok_or(LexError::NumberOverflow)
                }
            }
        )*
    }
}

impl_int!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128);

fn parse_int_digits<N: Int>(
    negative: bool,
    text: &str,
    radix: u32,
) -> Result<N, LexError> {
    let chars = text
        .chars()
        .filter(|&c| c != '_')
        .map(|c| c.to_digit(radix).ok_or(LexError::InvalidCharInNum(c)));

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

fn parse_exp(text: &str) -> Result<i32, LexError> {
    Ok(match text.chars().next() {
        Some('+') => parse_int_digits(false, &text[1..], 10)?,
        Some('-') => parse_int_digits(true, &text[1..], 10)?,
        _ => parse_int_digits(false, text, 10)?,
    })
}

fn parse_at_dot(text: &str) -> Result<f64, LexError> {
    let text = if text.contains('_') {
        Cow::Owned(text.chars().filter(|&c| c != '_').collect())
    } else {
        Cow::Borrowed(text)
    };
    text.parse().map_err(|_| LexError::InvalidNum)
}

pub(crate) fn leading_dot(input: &str) -> Result<NumberLiteral, LexError> {
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
        Err(LexError::NumberOverflow)
    }
}

pub(crate) fn float(input: &str) -> Result<NumberLiteral, LexError> {
    let input = input.trim_end_matches('_');
    if input.ends_with(|c: char| c == 'e' || c == 'E' || c == '.') {
        return Err(LexError::InvalidNum);
    }
    let exp = input.find(|c: char| c == 'e' || c == 'E');
    let num: f64 = if let Some(exp_index) = exp {
        let exp = parse_exp(&input[exp_index + 1..])?;
        let num: String = input[..exp_index].chars().filter(|&c| c != '_').collect();
        format!("{}e{}", num, exp).parse().map_err(|_| LexError::InvalidNum)?
    } else {
        let num: String = input.chars().filter(|&c| c != '_').collect();
        num.parse().map_err(|_| LexError::InvalidNum)?
    };
    if num.is_finite() {
        Ok(NumberLiteral::Float(num))
    } else {
        Err(LexError::NumberOverflow)
    }
}

fn int_with_radix(
    input: &str,
    radix_width: usize,
    radix: u32,
) -> Result<NumberLiteral, LexError> {
    Ok(match input.chars().next() {
        Some('-') => {
            let text = input[radix_width + 1..].trim_start_matches('_');
            if text.is_empty() {
                return Err(LexError::InvalidNum);
            }
            NumberLiteral::Int(parse_int_digits(true, text, radix)?)
        }
        Some('+') => {
            let text = input[radix_width + 1..].trim_start_matches('_');
            if text.is_empty() {
                return Err(LexError::InvalidNum);
            }
            NumberLiteral::UInt(parse_int_digits(false, text, radix)?)
        }
        _ => {
            let text = input[radix_width..].trim_start_matches('_');
            if text.is_empty() {
                return Err(LexError::InvalidNum);
            }
            NumberLiteral::Int(parse_int_digits(false, text, radix)?)
        }
    })
}

pub(crate) fn hex(input: &str) -> Result<NumberLiteral, LexError> {
    int_with_radix(input, 2, 16)
}

pub(crate) fn oct(input: &str) -> Result<NumberLiteral, LexError> {
    int_with_radix(input, 2, 8)
}

pub(crate) fn bin(input: &str) -> Result<NumberLiteral, LexError> {
    int_with_radix(input, 2, 2)
}

pub(crate) fn dec(input: &str) -> Result<NumberLiteral, LexError> {
    int_with_radix(input, 0, 10)
}

pub(super) fn parse_number(input: &str) -> TokenData {
    if input.starts_with('.') {
        into_token_data(leading_dot(input))
    } else {
        let without_sign =
            input.strip_prefix(|c: char| c == '+' || c == '-').unwrap_or(input);
        if without_sign.starts_with('0') {
            if let Some(x) = without_sign.chars().nth(1) {
                match x {
                    'x' | 'X' => return into_token_data(hex(input)),
                    'b' | 'B' => return into_token_data(bin(input)),
                    'o' | 'O' => return into_token_data(oct(input)),
                    _ => {}
                }
            }
        }
        if without_sign.contains(|c: char| c == '.' || c == 'e' || c == 'E') {
            into_token_data(float(input))
        } else {
            into_token_data(dec(input))
        }
    }
}

fn into_token_data(result: Result<NumberLiteral, LexError>) -> TokenData {
    result.map(TokenData::NumberLit).unwrap_or_else(TokenData::Error)
}
