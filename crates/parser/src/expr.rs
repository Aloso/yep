use std::iter::Peekable;

use ast::expr::*;
use ast::item::{Name, NamedType};
use ast::token::{
    Ident, Keyword, NumberLiteral, Operator, Punctuation, StringLiteral, TokenData,
    UpperIdent,
};
use ast::Spanned;

use crate::uoret;

use super::helpers::*;
use super::{Error, LexerMut, Parse, ParseResult};

impl Parse for Expr {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let mut parts = Vec::new();

        let mut len = lexer.len();
        while let Some(part) = ExprPart::parse(lexer)? {
            parts.push(part);
            if lexer.len() == len {
                return Err(Error::ExpectedGot2("expression", lexer.peek().data()));
            }
            len = lexer.len();
        }
        Ok(if parts.is_empty() {
            None
        } else if parts.len() == 1 {
            let (expr, span) = parts.pop().unwrap().into_inner();
            let expr_data = match expr {
                ExprPart::Literal(o) => Expr::Literal(o),
                ExprPart::Invokable(o) => Expr::Invokable(o),
                ExprPart::Lambda(o) => Expr::Lambda(o),
                ExprPart::Block(o) => Expr::Block(o),
                ExprPart::Parens(o) => Expr::Tuple(o),
                ExprPart::And | ExprPart::Or | ExprPart::Dot | ExprPart::Equals => {
                    return Ok(None)
                }
            };
            Some(span.embed(expr_data))
        } else {
            let expr = pratt_parser(&mut parts.into_iter().peekable(), 0)?;
            Some(expr)
        })
    }
}

/// <https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html>
fn pratt_parser(
    expr_parts: &mut Peekable<impl Iterator<Item = Spanned<ExprPart>>>,
    min_bp: u8,
) -> Result<Spanned<Expr>, Error> {
    fn postfix_binding_power(op: &ExprPart) -> Option<(u8, ())> {
        match op.kind() {
            ExprPartKind::InvokableType => Some((11, ())),
            ExprPartKind::Parens => Some((9, ())),
            _ => None,
        }
    }

    fn infix_binding_power(op: &ExprPart) -> Option<(u8, u8)> {
        match op.kind() {
            ExprPartKind::Dot => Some((13, 14)),
            ExprPartKind::InvokableOperator => Some((7, 8)),
            ExprPartKind::And => Some((5, 6)),
            ExprPartKind::Or => Some((3, 4)),
            ExprPartKind::Equals => Some((2, 1)),
            _ => None,
        }
    }

    let lhs = expr_parts.next().ok_or(Error::Expected("expression"))?;
    let mut lhs = lhs.span.embed(lhs.inner.into_operand()?);

    loop {
        let op = match expr_parts.peek() {
            None => break,
            Some(op) => op,
        };
        op.assert_is_operator(&lhs.inner)?;

        if let Some((l_bp, ())) = postfix_binding_power(&op.inner) {
            if l_bp < min_bp {
                break;
            }
            let op = expr_parts.next().unwrap();
            let lhs_span = lhs.span;

            let lhs_data = match op.inner {
                ExprPart::Parens(tuple) => Expr::ParenCall(ParenCall {
                    receiver: Box::new(lhs),
                    args: Some(tuple.into_fun_call_args()),
                }),
                ExprPart::Invokable(Invokable { name, generics: args }) => {
                    match name.into_inner() {
                        (Name::Type(name), name_span) => {
                            let name = Spanned::new(name, name_span);
                            Expr::TypeAscription(TypeAscription {
                                expr: Box::new(lhs),
                                ty: NamedType { name, args },
                            })
                        }
                        (t, _) => panic!("Unexpected token {:?}", t),
                    }
                }
                t => panic!("Unexpected token {:?}", t),
            };
            lhs = lhs_span.merge(op.span).embed(lhs_data);
            continue;
        }

        if let Some((l_bp, r_bp)) = infix_binding_power(op) {
            if l_bp < min_bp {
                break;
            }
            let op = expr_parts.next().unwrap();

            let rhs = pratt_parser(expr_parts, r_bp)?;
            lhs = op.inner.into_operation(lhs, rhs)?;
            continue;
        }

        break;
    }

    Ok(lhs)
}

impl Parse for Literal {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        or2(
            map(NumberLiteral::parse, Literal::NumberLit),
            map(StringLiteral::parse, Literal::StringLit),
        )(lexer)
    }
}

impl Parse for Invokable {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let name = uoret!(Name::parse(lexer)?);
        let generics = parse_type_arguments(lexer)?;
        let span = name.span.merge_if(&generics);
        let generics = generics.unwrap_or_default();
        Ok(Some(span.embed(Invokable { name, generics })))
    }
}

impl Parse for Operator {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        Ok(match lexer.peek().data() {
            TokenData::Operator(o) => Some(lexer.next().span.embed(o)),
            _ => None,
        })
    }
}

impl Parse for Lambda {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let args = uoret!(enclose_multiple(
            LambdaArgument::parse,
            Punctuation::Pipe,
            Punctuation::Comma,
            Punctuation::Pipe,
            true,
        )(lexer)?);

        let body = or2(map(Block::parse, Expr::Block), Expr::parse)(lexer)?;
        let body = Box::new(body.ok_or(Error::Expected("block or expression"))?);

        let span = args.span.merge(body.span);

        Ok(Some(span.embed(Lambda { args, body })))
    }
}

impl Parse for Block {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let span1 = uoret!(lexer.eat(Punctuation::OpenBrace));

        let exprs = vec_separated(lexer, Expr::parse, Punctuation::Semicolon)?
            .unwrap_or_default()
            .inner;

        let ends_with_semicolon = lexer.eat(Punctuation::Semicolon).is_some();

        let span2 = lexer.expect(Punctuation::CloseBrace)?;
        Ok(Some(span1.merge(span2).embed(Block { exprs, ends_with_semicolon })))
    }
}

impl Parse for Parens {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let span1 = uoret!(lexer.eat(Punctuation::OpenParen));

        let exprs = vec_separated(lexer, FunCallArgument::parse, Punctuation::Comma)?
            .unwrap_or_default()
            .inner;

        if !exprs.is_empty() {
            let _ = lexer.eat(Punctuation::Comma);
        }

        let span2 = lexer.expect(Punctuation::CloseParen)?;
        Ok(Some(span1.merge(span2).embed(Parens { exprs })))
    }
}

impl Parse for LambdaArgument {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let name = uoret!(Ident::parse(lexer)?);
        let ty = NamedType::parse(lexer)?;
        let span = name.span.merge_if(&ty);
        Ok(Some(span.embed(LambdaArgument { name, ty })))
    }
}

impl Parse for Declaration {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let (decl_kind, span) = uoret!(DeclKind::parse(lexer)?).into_inner();
        let name = Ident::parse_expect(lexer, "variable name")?;
        lexer.expect(Punctuation::Equals)?;
        let value = Box::new(Expr::parse_expect(lexer, "expression")?);
        let span = span.merge(value.span);

        Ok(Some(span.embed(Declaration { decl_kind, name, value })))
    }
}

impl Parse for DeclKind {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let decl_kind = match lexer.peek().data() {
            TokenData::Keyword(Keyword::Let) => DeclKind::Let,
            TokenData::Keyword(Keyword::Var) => DeclKind::Var,
            _ => return Ok(None),
        };
        Ok(Some(lexer.next().span.embed(decl_kind)))
    }
}

impl Parse for FunCallArgument {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        fn parse_with_name(lexer: LexerMut) -> ParseResult<FunCallArgument> {
            let mut lexer_clone = lexer.clone();

            let name = uoret!(Ident::parse(&mut lexer_clone)?);
            uoret!(lexer_clone.eat(Punctuation::Colon));
            let expr = Expr::parse_expect(&mut lexer_clone, "expression")?;

            *lexer = lexer_clone;
            let span = name.span.merge(expr.span);
            Ok(Some(span.embed(FunCallArgument { name: Some(name), expr })))
        }

        fn wrap_expr(expr: Spanned<Expr>) -> Spanned<FunCallArgument> {
            expr.span.embed(FunCallArgument { name: None, expr })
        }

        or2(parse_with_name, map2(Expr::parse, wrap_expr))(lexer)
    }
}

impl Parse for StringLiteral {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        Ok(match lexer.peek().data() {
            TokenData::StringLit(s) => Some(lexer.next().span.embed(s)),
            _ => None,
        })
    }
}

impl Parse for NumberLiteral {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        Ok(match lexer.peek().data() {
            TokenData::NumberLit(n) => Some(lexer.next().span.embed(n)),
            _ => None,
        })
    }
}

impl Parse for Ident {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        Ok(match lexer.peek().data() {
            TokenData::Ident(i) => Some(lexer.next().span.embed(i)),
            _ => None,
        })
    }
}

impl Parse for UpperIdent {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        Ok(match lexer.peek().data() {
            TokenData::UpperIdent(i) => Some(lexer.next().span.embed(i)),
            _ => None,
        })
    }
}

#[derive(Debug, Clone)]
pub(super) enum ExprPart {
    Literal(Literal),
    Invokable(Invokable),
    Lambda(Lambda),
    Block(Block),
    Parens(Parens),
    And,
    Or,
    Dot,
    Equals,
}

pub(super) enum ExprPartKind {
    StringLit,
    NumberLit,
    InvokableIdent,
    InvokableType,
    InvokableOperator,
    Lambda,
    Block,
    Parens,
    And,
    Or,
    Dot,
    Equals,
}

impl Parse for ExprPart {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        #[allow(clippy::unnecessary_wraps)]
        fn parse_and_or_dot_equals(lexer: LexerMut) -> ParseResult<ExprPart> {
            let part = match lexer.peek().data() {
                TokenData::Keyword(Keyword::And) => ExprPart::And,
                TokenData::Keyword(Keyword::Or) => ExprPart::Or,
                TokenData::Punct(Punctuation::Dot) => ExprPart::Dot,
                TokenData::Punct(Punctuation::Equals) => ExprPart::Equals,
                _ => return Ok(None),
            };
            Ok(Some(lexer.next().span.embed(part)))
        }

        or6(
            map(Literal::parse, ExprPart::Literal),
            map(Invokable::parse, ExprPart::Invokable),
            map(Lambda::parse, ExprPart::Lambda),
            map(Block::parse, ExprPart::Block),
            map(Parens::parse, ExprPart::Parens),
            parse_and_or_dot_equals,
        )(lexer)
    }
}

impl ExprPart {
    fn kind(&self) -> ExprPartKind {
        match self {
            ExprPart::Literal(Literal::StringLit(_)) => ExprPartKind::StringLit,
            ExprPart::Literal(Literal::NumberLit(_)) => ExprPartKind::NumberLit,
            ExprPart::Invokable(i) => match *i.name {
                Name::Operator(_) => ExprPartKind::InvokableOperator,
                Name::Ident(_) => ExprPartKind::InvokableIdent,
                Name::Type(_) => ExprPartKind::InvokableType,
            },
            ExprPart::Lambda(_) => ExprPartKind::Lambda,
            ExprPart::Block(_) => ExprPartKind::Block,
            ExprPart::Parens(_) => ExprPartKind::Parens,
            ExprPart::And => ExprPartKind::And,
            ExprPart::Or => ExprPartKind::Or,
            ExprPart::Dot => ExprPartKind::Dot,
            ExprPart::Equals => ExprPartKind::Equals,
        }
    }

    fn into_operand(self) -> Result<Expr, Error> {
        Ok(match self {
            ExprPart::Literal(l) => Expr::Literal(l),
            ExprPart::Invokable(n) => Expr::Invokable(n),
            ExprPart::Lambda(l) => Expr::Lambda(l),
            ExprPart::Block(b) => Expr::Block(b),
            ExprPart::Parens(p) => Expr::Tuple(p),
            ExprPart::And => return Err(Error::ExpectedGot4("operand", "`and`")),
            ExprPart::Or => return Err(Error::ExpectedGot4("operand", "`or`")),
            ExprPart::Dot => return Err(Error::ExpectedGot4("operand", "`.`")),
            ExprPart::Equals => return Err(Error::ExpectedGot4("operand", "`=`")),
        })
    }

    fn assert_is_operator(&self, lhs: &Expr) -> Result<(), Error> {
        match self {
            ExprPart::Parens(_) | ExprPart::Dot | ExprPart::Equals => Ok(()),

            ExprPart::Invokable(i) => match *i.name {
                Name::Operator(_) | Name::Type(_) => validate_operand(lhs),
                Name::Ident(_) => {
                    Err(Error::ExpectedGot3("operator", Expr::Invokable(i.clone())))
                }
            },

            ExprPart::And | ExprPart::Or => validate_operand(lhs),

            ExprPart::Lambda(l) => {
                Err(Error::ExpectedGot3("operator", Expr::Lambda(l.clone())))
            }

            ExprPart::Block(b) => {
                Err(Error::ExpectedGot3("operator", Expr::Block(b.clone())))
            }

            ExprPart::Literal(l) => {
                Err(Error::ExpectedGot3("operator", Expr::Literal(*l)))
            }
        }
    }

    fn into_operation(
        self,
        lhs: Spanned<Expr>,
        rhs: Spanned<Expr>,
    ) -> Result<Spanned<Expr>, Error> {
        let span = lhs.span.merge(rhs.span);
        let data = match self {
            ExprPart::Invokable(i) => match *i.name {
                Name::Operator(operator) => {
                    validate_operand(&lhs.inner)?;
                    validate_operand(&rhs.inner)?;
                    Expr::Operation(Operation {
                        operator,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    })
                }
                _ => panic!("Unexpected name, expected operator, found {:?}", i),
            },
            ExprPart::And => {
                validate_operand(&lhs.inner)?;
                validate_operand(&rhs.inner)?;
                Expr::ShortcircuitingOp(ScOperation {
                    operator: ScOperator::And,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                })
            }
            ExprPart::Or => {
                validate_operand(&lhs.inner)?;
                validate_operand(&rhs.inner)?;
                Expr::ShortcircuitingOp(ScOperation {
                    operator: ScOperator::Or,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                })
            }
            ExprPart::Dot => Expr::MemberCall(MemberCall {
                member: match rhs.into_inner() {
                    (Expr::Invokable(i), _) => i,
                    (e, _) => return Err(Error::ExpectedGot3("name", e)),
                },
                receiver: Box::new(lhs),
            }),
            ExprPart::Equals => {
                validate_operand(&lhs.inner)?;
                Expr::Assignment(Assignment { lhs: Box::new(lhs), rhs: Box::new(rhs) })
            }
            e => panic!("Expected name, infix operator, `.` or `=`, got {:?}", e),
        };
        Ok(span.embed(data))
    }
}

fn validate_operand(expr: &Expr) -> Result<(), Error> {
    if let Some(op) = expr.to_operator() {
        Err(Error::OperatorInsteadOfOperand(op))
    } else {
        Ok(())
    }
}
