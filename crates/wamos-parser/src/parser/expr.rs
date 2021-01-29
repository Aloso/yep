use std::fmt;
use std::iter::Peekable;

use crate::text_range::TextRange;
use crate::uoret;
use crate::{
    lexer::{
        Ident, Keyword, NumberLiteral, Operator, Punctuation, StringLiteral, TokenData,
        UpperIdent,
    },
    text_range::Spanned,
};

use super::helpers::*;
use super::items::*;
use super::patterns::Pattern;
use super::{Error, LexerMut, Parse, ParseResult};

#[derive(Debug, Clone)]
pub enum ExprData {
    Invokable(Invokable),
    Literal(Literal),
    ParenCall(ParenCall),
    MemberCall(MemberCall),
    Operation(Operation),
    ShortcircuitingOp(ScOperation),
    Assignment(Assignment),
    TypeAscription(TypeAscription),
    Statement(Expr),
    Lambda(Lambda),
    Block(Block),
    Tuple(Parens),
    Empty(Empty),

    Declaration(Declaration),
    Case(Case),
}

#[derive(Clone)]
pub struct Expr {
    pub(super) inner: Box<Spanned<ExprData>>,
}

impl From<Spanned<ExprData>> for Expr {
    fn from(e: Spanned<ExprData>) -> Self { Expr { inner: Box::new(e) } }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.inner.fmt(f) }
}

impl Expr {
    pub fn new(data: ExprData, span: TextRange) -> Self {
        Self { inner: Box::new(Spanned::new(data, span)) }
    }

    pub fn inner(&self) -> &ExprData { self.inner.inner() }

    pub fn span(&self) -> TextRange { self.inner.span() }

    pub fn into_inner(self) -> (ExprData, TextRange) { self.inner.into_inner() }
}

#[derive(Debug, Clone)]
pub struct Invokable {
    pub name: (Name, TextRange),
    pub generics: Vec<(TypeArgument, TextRange)>,
}

#[derive(Debug, Copy, Clone)]
pub enum Literal {
    NumberLit(NumberLiteral),
    StringLit(StringLiteral),
}

#[derive(Debug, Clone)]
pub struct ParenCall {
    pub receiver: Expr,
    pub args: Option<Vec<(FunCallArgument, TextRange)>>,
}

#[derive(Debug, Clone)]
pub struct MemberCall {
    pub receiver: Expr,
    pub member: Invokable,
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub operator: Operator,
    pub lhs: Expr,
    pub rhs: Expr,
}

/// Short-circuiting
#[derive(Debug, Clone)]
pub struct ScOperation {
    pub operator: ScOperator,
    pub lhs: Expr,
    pub rhs: Expr,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub lhs: Expr,
    pub rhs: Expr,
}

/// Short-circuiting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScOperator {
    And,
    Or,
}

#[derive(Debug, Clone)]
pub struct TypeAscription {
    pub expr: Expr,
    pub ty: NamedType,
}

#[derive(Debug, Clone)]
pub struct Lambda {
    pub args: (Vec<(LambdaArgument, TextRange)>, TextRange),
    pub body: Expr,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub exprs: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct Parens {
    pub exprs: Vec<(FunCallArgument, TextRange)>,
}

#[derive(Debug, Clone, Copy)]
pub struct Empty;

#[derive(Debug, Clone)]
pub struct Declaration {
    pub decl_kind: DeclKind,
    pub name: Ident,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct Case {
    pub expr: Expr,
    pub match_arms: Vec<MatchArm>,
}

#[derive(Debug, Clone)]
pub struct FunCallArgument {
    pub name: Option<(Ident, TextRange)>,
    pub expr: Spanned<ExprData>,
}

#[derive(Debug, Clone, Copy)]
pub enum DeclKind {
    Let,
    Var,
}

#[derive(Debug, Clone)]
pub struct LambdaArgument {
    pub name: (Ident, TextRange),
    pub ty: Option<(NamedType, TextRange)>,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub expr: ExprData,
}

impl Parse for ExprData {
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
            let (expr, span) = parts.pop().unwrap();
            let expr = match expr {
                ExprPart::Literal(o) => ExprData::Literal(o),
                ExprPart::Invokable(o) => ExprData::Invokable(o),
                ExprPart::Lambda(o) => ExprData::Lambda(o),
                ExprPart::Block(o) => ExprData::Block(o),
                ExprPart::Parens(o) => ExprData::Tuple(o),
                ExprPart::And | ExprPart::Or | ExprPart::Dot | ExprPart::Equals => {
                    return Ok(None)
                }
            };
            Some((expr, span))
        } else {
            let expr = pratt_parser(&mut parts.into_iter().peekable(), 0)?;
            Some(expr.into_inner())
        })
    }
}

macro_rules! invk {
    ($s1:ident($p:pat)) => {
        Invokable { name: (Name::$s1($p), _), .. }
    };
    ($s1:ident($p:pat), $s2:pat) => {
        Invokable { name: (Name::$s1($p), $s2), .. }
    };
}

/// <https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html>
fn pratt_parser(
    expr_parts: &mut Peekable<impl Iterator<Item = (ExprPart, TextRange)>>,
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

    let (lhs, lhs_span) = expr_parts.next().ok_or(Error::Expected("expression"))?;
    let mut lhs = lhs.into_operand(lhs_span)?;

    loop {
        let (op, _) = match expr_parts.peek() {
            None => break,
            Some(op) => op,
        };
        op.assert_is_operator(&lhs)?;

        if let Some((l_bp, ())) = postfix_binding_power(op) {
            if l_bp < min_bp {
                break;
            }
            let (op, op_span) = expr_parts.next().unwrap();
            let lhs_span = lhs.span();

            let lhs_data = match op {
                ExprPart::Parens(tuple) => ExprData::ParenCall(ParenCall {
                    receiver: lhs,
                    args: Some(tuple.into_fun_call_args()),
                }),
                ExprPart::Invokable(Invokable {
                    name: (Name::Type(name), name_span),
                    generics: args,
                }) => ExprData::TypeAscription(TypeAscription {
                    expr: lhs,
                    ty: NamedType { name: (name, name_span), args },
                }),
                t => panic!("Unexpected token {:?}", t),
            };
            lhs = Expr::new(lhs_data, lhs_span.merge(op_span));
            continue;
        }

        if let Some((l_bp, r_bp)) = infix_binding_power(op) {
            if l_bp < min_bp {
                break;
            }
            let (op, _) = expr_parts.next().unwrap();

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
        or2(
            map(NumberLiteral::parse, Literal::NumberLit),
            map(StringLiteral::parse, Literal::StringLit),
        )(lexer)
    }
}

impl Parse for Invokable {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let name = uoret!(Name::parse(lexer)?);
        let (generics, gen_span) = parse_type_arguments(lexer)?.unwrap_or_default();
        let span = name.1.merge(gen_span);
        Ok(Some((Invokable { name, generics }, span)))
    }
}

impl Parse for Operator {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        Ok(match lexer.peek().data() {
            TokenData::Operator(o) => {
                let span = lexer.next().span();
                Some((o, span))
            }
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

        let body = or2(map(Block::parse, ExprData::Block), ExprData::parse)(lexer)?;
        let body: Expr = Spanned::from(body.ok_or_else(|| todo!())?).into();

        let span = args.1.merge(body.span());

        Ok(Some((Lambda { args, body }, span)))
    }
}

impl Parse for Block {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let span1 = uoret!(lexer.eat(Punctuation::OpenBrace));

        let (mut exprs, _) =
            vec_separated(lexer, ExprData::parse, Punctuation::Semicolon)?
                .map(|(v, s)| {
                    let v = v
                        .into_iter()
                        .map(Spanned::from)
                        .map(Expr::from)
                        .collect::<Vec<Expr>>();
                    (v, s)
                })
                .unwrap_or_default();

        if let Some(span) = lexer.eat(Punctuation::Semicolon) {
            exprs.push(Expr::new(ExprData::Empty(Empty), span));
        }

        let span2 = lexer.expect(Punctuation::CloseBrace)?;
        Ok(Some((Block { exprs }, span1.merge(span2))))
    }
}

impl Parse for Parens {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let span1 = uoret!(lexer.eat(Punctuation::OpenParen));

        let (exprs, _) =
            vec_separated(lexer, FunCallArgument::parse, Punctuation::Comma)?
                .unwrap_or_default();

        if !exprs.is_empty() {
            let _ = lexer.eat(Punctuation::Comma);
        }

        let span2 = lexer.expect(Punctuation::CloseParen)?;
        Ok(Some((Parens { exprs }, span1.merge(span2))))
    }
}

impl Parens {
    fn into_fun_call_args(self) -> Vec<(FunCallArgument, TextRange)> { self.exprs }
}

impl Parse for LambdaArgument {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let name = uoret!(Ident::parse(lexer)?);
        let ty = NamedType::parse(lexer)?;
        let span = name.1.merge_if(&ty);
        Ok(Some((LambdaArgument { name, ty }, span)))
    }
}

impl Parse for Declaration {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let (decl_kind, span) = uoret!(DeclKind::parse(lexer)?);
        let (name, _) = Ident::parse_expect(lexer, "variable name")?;
        lexer.expect(Punctuation::Equals)?;
        let value: Expr =
            Spanned::from(ExprData::parse_expect(lexer, "expression")?).into();
        let span = span.merge(value.span());

        Ok(Some((Declaration { decl_kind, name, value }, span)))
    }
}

impl Parse for DeclKind {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let decl_kind = match lexer.peek().data() {
            TokenData::Keyword(Keyword::Let) => DeclKind::Let,
            TokenData::Keyword(Keyword::Var) => DeclKind::Var,
            _ => return Ok(None),
        };
        let span = lexer.next().span();
        Ok(Some((decl_kind, span)))
    }
}

impl Parse for FunCallArgument {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        fn parse_with_name(lexer: LexerMut) -> ParseResult<FunCallArgument> {
            let mut lexer_clone = lexer.clone();

            let name = uoret!(Ident::parse(&mut lexer_clone)?);
            uoret!(lexer_clone.eat(Punctuation::Colon));
            let expr = ExprData::parse_expect(&mut lexer_clone, "expression")?;

            *lexer = lexer_clone;
            let span = name.1.merge(expr.1);
            Ok(Some((FunCallArgument { name: Some(name), expr: expr.into() }, span)))
        }

        fn wrap_expr(expr: ExprData, span: TextRange) -> (FunCallArgument, TextRange) {
            (FunCallArgument { name: None, expr: Spanned::new(expr, span) }, span)
        }

        or2(parse_with_name, map2(ExprData::parse, wrap_expr))(lexer)
    }
}

impl Parse for StringLiteral {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        Ok(match lexer.peek().data() {
            TokenData::StringLit(s) => {
                let span = lexer.next().span();
                Some((s, span))
            }
            _ => None,
        })
    }
}

impl Parse for NumberLiteral {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        Ok(match lexer.peek().data() {
            TokenData::NumberLit(n) => {
                let span = lexer.next().span();
                Some((n, span))
            }
            _ => None,
        })
    }
}

impl Parse for Ident {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        Ok(match lexer.peek().data() {
            TokenData::Ident(i) => {
                let span = lexer.next().span();
                Some((i, span))
            }
            _ => None,
        })
    }
}

impl Parse for UpperIdent {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        Ok(match lexer.peek().data() {
            TokenData::UpperIdent(i) => {
                let span = lexer.next().span();
                Some((i, span))
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
            let span = lexer.next().span();
            Ok(Some((part, span)))
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
#[allow(unreachable_code, clippy::diverging_sub_expression)]
impl ExprPart {
    fn into_operand(self, span: TextRange) -> Result<Expr, Error> {
        let expr_data = match self {
            ExprPart::Literal(l) => ExprData::Literal(l),
            ExprPart::Invokable(n) => ExprData::Invokable(n),
            ExprPart::Lambda(l) => ExprData::Lambda(l),
            ExprPart::Block(b) => ExprData::Block(b),
            ExprPart::Parens(p) => ExprData::Tuple(p),
            ExprPart::And | ExprPart::Or | ExprPart::Dot | ExprPart::Equals => {
                return Err(todo!())
            }
        };
        Ok(Expr::new(expr_data, span))
    }

    fn assert_is_operator(&self, lhs: &Expr) -> Result<(), Error> {
        match self {
            ExprPart::Parens(_) | ExprPart::Dot | ExprPart::Equals => Ok(()),

            ExprPart::Invokable(invk!(Operator(_)))
            | ExprPart::Invokable(invk!(Type(_)))
            | ExprPart::And
            | ExprPart::Or => validate_operand(lhs.inner()),

            ExprPart::Invokable(i @ invk!(Ident(_))) => {
                Err(Error::ExpectedGot3("operator", ExprData::Invokable(i.clone())))
            }

            ExprPart::Lambda(l) => {
                Err(Error::ExpectedGot3("operator", ExprData::Lambda(l.clone())))
            }

            ExprPart::Block(b) => {
                Err(Error::ExpectedGot3("operator", ExprData::Block(b.clone())))
            }

            ExprPart::Literal(l) => {
                Err(Error::ExpectedGot3("operator", ExprData::Literal(*l)))
            }
        }
    }

    fn into_operation(self, lhs: Expr, rhs: Expr) -> Result<Expr, Error> {
        let span = lhs.span().merge(rhs.span());
        let data = match self {
            ExprPart::Invokable(invk!(Operator(operator))) => {
                validate_operand(lhs.inner())?;
                validate_operand(rhs.inner())?;
                ExprData::Operation(Operation { operator, lhs, rhs })
            }
            ExprPart::And => {
                validate_operand(lhs.inner())?;
                validate_operand(rhs.inner())?;
                ExprData::ShortcircuitingOp(ScOperation {
                    operator: ScOperator::And,
                    lhs,
                    rhs,
                })
            }
            ExprPart::Or => {
                validate_operand(lhs.inner())?;
                validate_operand(rhs.inner())?;
                ExprData::ShortcircuitingOp(ScOperation {
                    operator: ScOperator::Or,
                    lhs,
                    rhs,
                })
            }
            ExprPart::Dot => ExprData::MemberCall(MemberCall {
                member: match rhs.into_inner() {
                    (ExprData::Invokable(i), _) => i,
                    _ => return Err(todo!()),
                },
                receiver: lhs,
            }),
            ExprPart::Equals => {
                validate_operand(lhs.inner())?;
                ExprData::Assignment(Assignment { lhs, rhs })
            }
            _ => return Err(todo!()),
        };
        Ok(Expr::new(data, span))
    }
}

impl ExprData {
    fn to_operator(&self) -> Option<Operator> {
        match *self {
            ExprData::Invokable(Invokable { name: (Name::Operator(o), _), .. }) => {
                Some(o)
            }
            _ => None,
        }
    }
}

fn validate_operand(expr: &ExprData) -> Result<(), Error> {
    if let Some(op) = expr.to_operator() {
        Err(Error::OperatorInsteadOfOperand(op))
    } else {
        Ok(())
    }
}
