use ast::expr::Expr;
use ast::item::Item;
use ast::name::Operator;
use ast::{Spanned, TextRange, Token, TokenData};

pub use self::formatting::FancyFormat;

pub mod expr;
pub mod formatting;
mod formatting_impl;
mod helpers;
pub mod items;
pub mod patterns;

type Tokens<'a> = &'a [Token<'a>];
type ParseResult<T> = Result<Option<Spanned<T>>, Error>;
type LexerMut<'a, 'b, 'c> = &'a mut Lexer<'b, 'c>;

#[derive(Debug, Clone)]
struct Lexer<'a, 'b> {
    tokens: &'a [Spanned<Token<'b>>],
}

impl<'a, 'b> Lexer<'a, 'b> {
    fn from_tokens(tokens: &'a [Spanned<Token<'b>>]) -> Self { Self { tokens } }

    /// Returns `Some(())` and advances the lexer if the next token matches
    /// `elem`
    #[must_use]
    fn eat(&mut self, token: impl Into<TokenData>) -> Option<TextRange> {
        let (next, rest) = self.tokens.split_first().unwrap();
        if token.into() == next.data() {
            self.tokens = rest;
            Some(next.span)
        } else {
            None
        }
    }

    fn expect(&mut self, token: impl Into<TokenData>) -> Result<TextRange, Error> {
        let expected = token.into();
        let got = self.peek().data();
        if expected == got {
            Ok(self.next().span)
        } else {
            Err(Error::ExpectedGot(expected, got))
        }
    }

    /// Return the next token and advance the lexer
    fn next(&mut self) -> Spanned<Token> {
        let (&next, rest) = self.tokens.split_first().unwrap();
        self.tokens = rest;
        next
    }

    /// Return the next token _without_ advancing the lexer
    fn peek(&self) -> &Token { &self.tokens[0].inner }

    fn len(&self) -> usize { self.tokens.len() }

    fn finish(&mut self) -> Result<(), Error> {
        if self.tokens.is_empty()
            || (self.tokens.len() == 1 && self.tokens[0].data() == TokenData::EOF)
        {
            Ok(())
        } else {
            Err(Error::RemainingTokens(
                self.tokens.iter().map(|t| t.map_ref(|t| t.data())).collect(),
            ))
        }
    }

    pub fn parse_items(&'a mut self) -> Result<Vec<Spanned<Item>>, Error> {
        let mut results = Vec::new();
        while let Some(result) = Item::parse(self)? {
            results.push(result);
        }
        self.finish()?;
        Ok(results)
    }
}

pub fn parse(tokens: &[Spanned<Token>]) -> Result<Vec<Spanned<Item>>, Error> {
    Lexer::from_tokens(tokens).parse_items()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("There are remaining tokens that could not be parsed: {0:?}")]
    RemainingTokens(Vec<Spanned<TokenData>>),

    #[error("Expected {0}")]
    Expected(&'static str),

    #[error("Expected {0:?}, got {1:?}")]
    ExpectedGot(TokenData, TokenData),

    #[error("Expected {0}, got {1:?}")]
    ExpectedGot2(&'static str, TokenData),

    #[error("Expected {0}, got {1:?}")]
    ExpectedGot3(&'static str, Expr),

    #[error("Expected {0}, got {1}")]
    ExpectedGot4(&'static str, &'static str),

    #[error(
        "Operators are not allowed here: {0:?}\n  tip: Wrap the operator in braces, \
         e.g. `{{+}}`"
    )]
    OperatorInsteadOfOperand(Operator),
}

trait Parse: Sized {
    fn parse(lexer: LexerMut) -> ParseResult<Self>;

    #[inline]
    fn parse_expect(
        lexer: LexerMut,
        expect: &'static str,
    ) -> Result<Spanned<Self>, Error> {
        match Self::parse(lexer)? {
            Some(result) => Ok(result),
            None => Err(Error::ExpectedGot2(expect, lexer.peek().data())),
        }
    }
}


#[cfg(test)]
mod tests {
    fn parsed_equals(code: &str, out: &str) {
        let lexed = crate::lexer::lex(code);
        assert_eq!(lexed.errors(), vec![]);
        match super::parse(lexed.tokens()) {
            Ok(items) => {
                let actual = format!("{:#?}", items);
                let changes = difference::Changeset::new(&actual, out, "\n");
                if changes.distance > 0 {
                    eprintln!("{}", changes);
                    panic!(
                        "{} differences between expected and actual output",
                        changes.distance
                    );
                }
            }
            Err(err) => {
                panic!("{}", err);
            }
        }
    }

    #[test]
    fn test() {
        parsed_equals(
            "fun foo[T](x List[T], y) List[T] {
    x = x.map(y) List[T];
    assert(x.len != 0);
    x
}",
            "[
    Function(
        Function {
            name: Ident(
                Ident #1,
            ) @ 4..7,
            generics: [
                GenericParam {
                    name: UpperIdent #2 @ 8..9,
                    bounds: [],
                } @ 8..9,
            ] @ 7..10,
            args: [
                FunArgument {
                    name: Ident #3,
                    ty: Some(
                        NamedType {
                            name: UpperIdent #4 @ 13..17,
                            args: [
                                Type(
                                    NamedType {
                                        name: UpperIdent #2 @ 18..19,
                                        args: [] @ 0..0,
                                    },
                                ) @ 18..19,
                            ] @ 17..20,
                        } @ 13..20,
                    ),
                    default: None,
                } @ 11..20,
                FunArgument {
                    name: Ident #5,
                    ty: None,
                    default: None,
                } @ 22..23,
            ] @ 10..24,
            return_ty: Some(
                NamedType {
                    name: UpperIdent #4 @ 25..29,
                    args: [
                        Type(
                            NamedType {
                                name: UpperIdent #2 @ 30..31,
                                args: [] @ 0..0,
                            },
                        ) @ 30..31,
                    ] @ 29..32,
                } @ 25..32,
            ),
            body: Some(
                Block {
                    exprs: [
                        Assignment(
                            Assignment {
                                lhs: Invokable(
                                    Invokable {
                                        name: Ident(
                                            Ident #3,
                                        ) @ 39..40,
                                        generics: [] @ 0..0,
                                    },
                                ) @ 39..40,
                                rhs: TypeAscription(
                                    TypeAscription {
                                        expr: ParenCall(
                                            ParenCall {
                                                receiver: MemberCall(
                                                    MemberCall {
                                                        receiver: Invokable(
                                                            Invokable {
                                                                name: Ident(
                                                                    Ident #3,
                                                                ) @ 43..44,
                                                                generics: [] @ 0..0,
                                                            },
                                                        ) @ 43..44,
                                                        member: Invokable {
                                                            name: Ident(
                                                                Ident #6,
                                                            ) @ 45..48,
                                                            generics: [] @ 0..0,
                                                        },
                                                    },
                                                ) @ 43..48,
                                                args: Some(
                                                    [
                                                        FunCallArgument {
                                                            name: None,
                                                            expr: Invokable(
                                                                Invokable {
                                                                    name: Ident(
                                                                        Ident #5,
                                                                    ) @ 49..50,
                                                                    generics: [] @ 0..0,
                                                                },
                                                            ) @ 49..50,
                                                        } @ 49..50,
                                                    ],
                                                ),
                                            },
                                        ) @ 43..51,
                                        ty: NamedType {
                                            name: UpperIdent #4 @ 52..56,
                                            args: [
                                                Type(
                                                    NamedType {
                                                        name: UpperIdent #2 @ 57..58,
                                                        args: [] @ 0..0,
                                                    },
                                                ) @ 57..58,
                                            ] @ 56..59,
                                        },
                                    },
                                ) @ 43..59,
                            },
                        ) @ 39..59,
                        ParenCall(
                            ParenCall {
                                receiver: Invokable(
                                    Invokable {
                                        name: Ident(
                                            Ident #7,
                                        ) @ 65..71,
                                        generics: [] @ 0..0,
                                    },
                                ) @ 65..71,
                                args: Some(
                                    [
                                        FunCallArgument {
                                            name: None,
                                            expr: Operation(
                                                Operation {
                                                    operator: Operator #9,
                                                    lhs: MemberCall(
                                                        MemberCall {
                                                            receiver: Invokable(
                                                                Invokable {
                                                                    name: Ident(
                                                                        Ident #3,
                                                                    ) @ 72..73,
                                                                    generics: [] @ 0..0,
                                                                },
                                                            ) @ 72..73,
                                                            member: Invokable {
                                                                name: Ident(
                                                                    Ident #8,
                                                                ) @ 74..77,
                                                                generics: [] @ 0..0,
                                                            },
                                                        },
                                                    ) @ 72..77,
                                                    rhs: Literal(
                                                        Int(0),
                                                    ) @ 81..82,
                                                },
                                            ) @ 72..82,
                                        } @ 72..82,
                                    ],
                                ),
                            },
                        ) @ 65..83,
                        Invokable(
                            Invokable {
                                name: Ident(
                                    Ident #3,
                                ) @ 89..90,
                                generics: [] @ 0..0,
                            },
                        ) @ 89..90,
                    ],
                    ends_with_semicolon: false,
                } @ 33..92,
            ),
        },
    ) @ 0..92,
]",
        );
    }
}
