use std::iter::Peekable;

use crate::lexer::{Ident, Keyword, NumberLiteral, Operator, Punctuation, TokenData, UpperIdent};
use crate::uoret;

use super::helpers::*;
use super::items::*;
use super::patterns::Pattern;
use super::{Error, LexerMut, Parse, ParseResult};

#[derive(Debug, Clone)]
pub enum Expr {
    Invokable(Invokable),
    Literal(Literal),
    ParenCall(ParenCall),
    MemberCall(MemberCall),
    Operation(Operation),
    ShortcircuitingOp(ScOperation),
    Assignment(Assignment),
    TypeAscription(TypeAscription),
    Statement(Box<Expr>),
    Lambda(Lambda),
    Block(Block),
    Tuple(Parens),
    Empty(Empty),

    Declaration(Declaration),
    Case(Case),
}

#[derive(Debug, Clone)]
pub struct Invokable {
    pub name: Name,
    pub generics: Vec<TypeArgument>,
}

#[derive(Debug, Clone)]
pub enum Literal {
    NumberLit(NumberLiteral),
}

#[derive(Debug, Clone)]
pub struct ParenCall {
    pub receiver: Box<Expr>,
    pub args: Option<Vec<FunCallArgument>>,
}

#[derive(Debug, Clone)]
pub struct MemberCall {
    pub receiver: Box<Expr>,
    pub member: Invokable,
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub operator: Operator,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

/// Short-circuiting
#[derive(Debug, Clone)]
pub struct ScOperation {
    pub operator: ScOperator,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

/// Short-circuiting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScOperator {
    And,
    Or,
}

#[derive(Debug, Clone)]
pub struct TypeAscription {
    pub expr: Box<Expr>,
    pub ty: NamedType,
}

#[derive(Debug, Clone)]
pub struct Lambda {
    pub args: Vec<LambdaArgument>,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub exprs: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct Parens {
    pub exprs: Vec<FunCallArgument>,
}

#[derive(Debug, Clone, Copy)]
pub struct Empty;

#[derive(Debug, Clone)]
pub struct Declaration {
    pub decl_kind: DeclKind,
    pub name: Ident,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Case {
    pub expr: Box<Expr>,
    pub match_arms: Vec<MatchArm>,
}

#[derive(Debug, Clone)]
pub struct FunCallArgument {
    pub name: Option<Ident>,
    pub expr: Expr,
}

#[derive(Debug, Clone, Copy)]
pub enum DeclKind {
    Let,
    Var,
}

#[derive(Debug, Clone)]
pub struct LambdaArgument {
    pub name: Ident,
    pub ty: Option<NamedType>,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub expr: Expr,
}

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
            let expr = match parts.pop().unwrap() {
                ExprPart::Literal(o) => Expr::Literal(o),
                ExprPart::Invokable(o) => Expr::Invokable(o),
                ExprPart::Lambda(o) => Expr::Lambda(o),
                ExprPart::Block(o) => Expr::Block(o),
                ExprPart::Parens(o) => Expr::Tuple(o),
                ExprPart::And | ExprPart::Or | ExprPart::Dot | ExprPart::Equals => return Ok(None),
            };
            Some(expr)
        } else {
            Some(pratt_parser(&mut parts.into_iter().peekable(), 0)?)
        })
    }
}

#[rustfmt::skip]
macro_rules! invk {
    ($s:ident($p:pat)) => {
        Invokable { name: Name::$s($p), .. }
    };
}

/// <https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html>
fn pratt_parser(
    expr_parts: &mut Peekable<impl Iterator<Item = ExprPart>>,
    min_bp: u8,
) -> Result<Expr, Error> {
    fn postfix_binding_power(op: &ExprPart) -> Option<(u8, ())> {
        match op {
            ExprPart::Invokable(invk!(Type(_))) => Some((11, ())),
            ExprPart::Parens(_) => Some((9, ())),
            _ => None,
        }
    }

    fn infix_binding_power(op: &ExprPart) -> Option<(u8, u8)> {
        match op {
            ExprPart::Dot => Some((13, 14)),
            ExprPart::Invokable(invk!(Operator(_))) => Some((7, 8)),
            ExprPart::And => Some((5, 6)),
            ExprPart::Or => Some((3, 4)),
            ExprPart::Equals => Some((2, 1)),
            _ => None,
        }
    }

    let mut lhs = expr_parts
        .next()
        .ok_or(Error::Expected("expression"))?
        .into_operand()?;

    loop {
        let op = match expr_parts.peek() {
            None => break,
            Some(op) => op,
        };
        op.assert_is_operator(&lhs)?;

        if let Some((l_bp, ())) = postfix_binding_power(op) {
            if l_bp < min_bp {
                break;
            }
            let op = expr_parts.next().unwrap();

            lhs = match op {
                ExprPart::Parens(tuple) => Expr::ParenCall(ParenCall {
                    receiver: Box::new(lhs),
                    args: Some(tuple.into_fun_call_args()),
                }),
                ExprPart::Invokable(Invokable {
                    name: Name::Type(name),
                    generics: args,
                }) => Expr::TypeAscription(TypeAscription {
                    expr: Box::new(lhs),
                    ty: NamedType { name, args },
                }),
                t => panic!("Unexpected token {:?}", t),
            };
            continue;
        }

        if let Some((l_bp, r_bp)) = infix_binding_power(op) {
            if l_bp < min_bp {
                break;
            }
            let op = expr_parts.next().unwrap();

            let rhs = pratt_parser(expr_parts, r_bp)?;
            lhs = op.into_operation(lhs, rhs)?;
            continue;
        }

        break;
    }

    Ok(lhs)
}

impl Parse for Literal {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        Ok(NumberLiteral::parse(lexer)?.map(Literal::NumberLit))
    }
}

impl Parse for Invokable {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let name = uoret!(Name::parse(lexer)?);
        let generics = parse_type_arguments(lexer)?.unwrap_or_default();
        Ok(Some(Invokable { name, generics }))
    }
}

impl Parse for Operator {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        Ok(match lexer.peek().data() {
            TokenData::Operator(o) => {
                lexer.next();
                Some(o)
            }
            _ => None,
        })
    }
}

impl Parse for Lambda {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        uoret!(lexer.eat(Punctuation::Pipe));

        let args =
            vec_separated(lexer, LambdaArgument::parse, Punctuation::Comma)?.unwrap_or_default();

        lexer.expect(Punctuation::Pipe)?;

        let body = or2(map(Block::parse, Expr::Block), Expr::parse)(lexer)?;
        let body = Box::new(body.ok_or_else(|| todo!())?);

        Ok(Some(Lambda { args, body }))
    }
}

impl Parse for Block {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        uoret!(lexer.eat(Punctuation::OpenBrace));

        let mut exprs =
            vec_separated(lexer, Expr::parse, Punctuation::Semicolon)?.unwrap_or_default();

        if lexer.eat(Punctuation::Semicolon).is_some() {
            exprs.push(Expr::Empty(Empty));
        }

        lexer.expect(Punctuation::CloseBrace)?;
        Ok(Some(Block { exprs }))
    }
}

impl Parse for Parens {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        uoret!(lexer.eat(Punctuation::OpenParen));

        let exprs =
            vec_separated(lexer, FunCallArgument::parse, Punctuation::Comma)?.unwrap_or_default();

        if !exprs.is_empty() {
            let _ = lexer.eat(Punctuation::Comma);
        }

        lexer.expect(Punctuation::CloseParen)?;
        Ok(Some(Parens { exprs }))
    }
}

impl Parens {
    fn into_fun_call_args(self) -> Vec<FunCallArgument> { self.exprs }
}

impl Parse for LambdaArgument {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let name = uoret!(Ident::parse(lexer)?);
        let ty = NamedType::parse(lexer)?;
        Ok(Some(LambdaArgument { name, ty }))
    }
}

impl Parse for Declaration {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let decl_kind = uoret!(DeclKind::parse(lexer)?);
        let name = Ident::parse_expect(lexer, "variable name")?;
        lexer.expect(Punctuation::Equals)?;
        let value = Box::new(Expr::parse_expect(lexer, "expression")?);

        Ok(Some(Declaration {
            decl_kind,
            name,
            value,
        }))
    }
}

impl Parse for DeclKind {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let decl_kind = match lexer.peek().data() {
            TokenData::Keyword(Keyword::Let) => DeclKind::Let,
            TokenData::Keyword(Keyword::Var) => DeclKind::Var,
            _ => return Ok(None),
        };
        lexer.next();
        Ok(Some(decl_kind))
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
            Ok(Some(FunCallArgument {
                name: Some(name),
                expr,
            }))
        }

        fn wrap_expr(expr: Expr) -> FunCallArgument { FunCallArgument { name: None, expr } }

        or2(parse_with_name, map(Expr::parse, wrap_expr))(lexer)
    }
}

impl Parse for NumberLiteral {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        Ok(match lexer.peek().data() {
            TokenData::NumberLit(n) => {
                lexer.next();
                Some(n)
            }
            _ => None,
        })
    }
}

impl Parse for Ident {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        Ok(match lexer.peek().data() {
            TokenData::Ident(i) => {
                lexer.next();
                Some(i)
            }
            _ => None,
        })
    }
}

impl Parse for UpperIdent {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        Ok(match lexer.peek().data() {
            TokenData::UpperIdent(i) => {
                lexer.next();
                Some(i)
            }
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
            lexer.next();
            Ok(Some(part))
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

// TODO: remove attribute
#[allow(unreachable_code)]
impl ExprPart {
    fn into_operand(self) -> Result<Expr, Error> {
        Ok(match self {
            ExprPart::Literal(l) => Expr::Literal(l),
            ExprPart::Invokable(n) => Expr::Invokable(n),
            ExprPart::Lambda(l) => Expr::Lambda(l),
            ExprPart::Block(b) => Expr::Block(b),
            ExprPart::Parens(p) => Expr::Tuple(p),
            ExprPart::And | ExprPart::Or | ExprPart::Dot | ExprPart::Equals => return Err(todo!()),
        })
    }

    fn assert_is_operator(&self, lhs: &Expr) -> Result<(), Error> {
        match self {
            ExprPart::Parens(_) | ExprPart::Dot | ExprPart::Equals => Ok(()),

            ExprPart::Invokable(invk!(Operator(_)))
            | ExprPart::Invokable(invk!(Type(_)))
            | ExprPart::And
            | ExprPart::Or => validate_operand(lhs),

            ExprPart::Invokable(i @ invk!(Ident(_))) => {
                Err(Error::ExpectedGot3("operator", Expr::Invokable(i.clone())))
            }

            ExprPart::Lambda(l) => Err(Error::ExpectedGot3("operator", Expr::Lambda(l.clone()))),

            ExprPart::Block(b) => Err(Error::ExpectedGot3("operator", Expr::Block(b.clone()))),

            ExprPart::Literal(l) => Err(Error::ExpectedGot3("operator", Expr::Literal(l.clone()))),
        }
    }

    fn into_operation(self, lhs: Expr, rhs: Expr) -> Result<Expr, Error> {
        Ok(match self {
            ExprPart::Invokable(invk!(Operator(operator))) => {
                validate_operand(&lhs)?;
                validate_operand(&rhs)?;
                Expr::Operation(Operation {
                    operator,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                })
            }
            ExprPart::And => {
                validate_operand(&lhs)?;
                validate_operand(&rhs)?;
                Expr::ShortcircuitingOp(ScOperation {
                    operator: ScOperator::And,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                })
            }
            ExprPart::Or => {
                validate_operand(&lhs)?;
                validate_operand(&rhs)?;
                Expr::ShortcircuitingOp(ScOperation {
                    operator: ScOperator::Or,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                })
            }
            ExprPart::Dot => Expr::MemberCall(MemberCall {
                member: match rhs {
                    Expr::Invokable(i) => i,
                    _ => return Err(todo!()),
                },
                receiver: Box::new(lhs),
            }),
            ExprPart::Equals => {
                validate_operand(&lhs)?;
                Expr::Assignment(Assignment {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                })
            }
            _ => return Err(todo!()),
        })
    }
}

impl Expr {
    fn to_operator(&self) -> Option<Operator> {
        match *self {
            Expr::Invokable(Invokable {
                name: Name::Operator(o),
                ..
            }) => Some(o),
            _ => None,
        }
    }
}

fn validate_operand(expr: &Expr) -> Result<(), Error> {
    if let Some(op) = expr.to_operator() {
        Err(Error::OperatorInsteadOfOperand(op))
    } else {
        Ok(())
    }
}
