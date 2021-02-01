use std::fmt;

use crate::item::{Name, NamedType, TypeArgument};
use crate::name::Operator;
use crate::pattern::Pattern;
use crate::{Ident, NumberLiteral, Spanned, SpannedList, StringLiteral};

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
    Statement(Box<Spanned<Expr>>),
    Lambda(Lambda),
    Block(Block),
    Tuple(Parens),
    Empty(Empty),

    Declaration(Declaration),
    Case(Case),
}

impl Expr {
    pub fn to_operator(&self) -> Option<Operator> {
        match self {
            Expr::Invokable(i) => match *i.name {
                Name::Operator(o) => Some(o),
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Invokable {
    pub name: Spanned<Name>,
    pub generics: Spanned<SpannedList<TypeArgument>>,
}

#[derive(Copy, Clone)]
pub enum Literal {
    NumberLit(NumberLiteral),
    StringLit(StringLiteral),
}

impl fmt::Debug for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::NumberLit(x) => write!(f, "{:?}", x),
            Literal::StringLit(x) => write!(f, "{:?}", x),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParenCall {
    pub receiver: Box<Spanned<Expr>>,
    pub args: Option<SpannedList<FunCallArgument>>,
}

#[derive(Debug, Clone)]
pub struct MemberCall {
    pub receiver: Box<Spanned<Expr>>,
    pub member: Invokable,
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub operator: Operator,
    pub lhs: Box<Spanned<Expr>>,
    pub rhs: Box<Spanned<Expr>>,
}

/// Short-circuiting
#[derive(Debug, Clone)]
pub struct ScOperation {
    pub operator: ScOperator,
    pub lhs: Box<Spanned<Expr>>,
    pub rhs: Box<Spanned<Expr>>,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub lhs: Box<Spanned<Expr>>,
    pub rhs: Box<Spanned<Expr>>,
}

/// Short-circuiting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScOperator {
    And,
    Or,
}

/// 56 bytes
#[derive(Debug, Clone)]
pub struct TypeAscription {
    pub expr: Box<Spanned<Expr>>,
    pub ty: NamedType,
}

#[derive(Debug, Clone)]
pub struct Lambda {
    pub args: Spanned<SpannedList<LambdaArgument>>,
    pub body: Box<Spanned<Expr>>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub exprs: SpannedList<Expr>,
    pub ends_with_semicolon: bool,
}

#[derive(Debug, Clone)]
pub struct Parens {
    pub exprs: SpannedList<FunCallArgument>,
}

impl Parens {
    pub fn into_fun_call_args(self) -> SpannedList<FunCallArgument> { self.exprs }
}

#[derive(Debug, Clone, Copy)]
pub struct Empty;

#[derive(Debug, Clone)]
pub struct Declaration {
    pub decl_kind: DeclKind,
    pub name: Spanned<Ident>,
    pub value: Box<Spanned<Expr>>,
}

#[derive(Debug, Clone)]
pub struct Case {
    pub expr: Box<Spanned<Expr>>,
    pub match_arms: Vec<MatchArm>,
}

#[derive(Debug, Clone)]
pub struct FunCallArgument {
    pub name: Option<Spanned<Ident>>,
    pub expr: Spanned<Expr>,
}

#[derive(Debug, Clone, Copy)]
pub enum DeclKind {
    Let,
    Var,
}

#[derive(Debug, Clone)]
pub struct LambdaArgument {
    pub name: Spanned<Ident>,
    pub ty: Option<Spanned<NamedType>>,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub expr: Expr,
}
