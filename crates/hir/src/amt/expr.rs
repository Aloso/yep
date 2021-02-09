use ast::{Spanned, SpannedList};

use crate::arena::Idx;

use super::literal::{NumberLiteral, StringLiteral};
use super::name::{Ident, Operator};
use super::pattern::Pattern;
use super::types::{NamedType, TypeArgument};
use super::Name;

#[derive(Clone)]
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
    Match(Match),
}

#[derive(Clone)]
pub struct Invokable {
    pub name: Spanned<Name>,
    pub generics: Spanned<SpannedList<TypeArgument>>,
}

#[derive(Clone)]
pub enum Literal {
    NumberLit(NumberLiteral),
    StringLit(StringLiteral),
}

#[derive(Clone)]
pub struct ParenCall {
    pub receiver: Spanned<Idx<Expr>>,
    pub args: Option<SpannedList<FunCallArgument>>,
}

#[derive(Clone)]
pub struct MemberCall {
    pub receiver: Spanned<Idx<Expr>>,
    pub member: Invokable,
}

#[derive(Clone)]
pub struct Operation {
    pub operator: Operator,
    pub lhs: Spanned<Idx<Expr>>,
    pub rhs: Spanned<Idx<Expr>>,
}

/// Short-circuiting
#[derive(Clone)]
pub struct ScOperation {
    pub operator: ScOperator,
    pub lhs: Spanned<Idx<Expr>>,
    pub rhs: Spanned<Idx<Expr>>,
}

#[derive(Clone)]
pub struct Assignment {
    pub lhs: Spanned<Idx<Expr>>,
    pub rhs: Spanned<Idx<Expr>>,
}

/// Short-circuiting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScOperator {
    And,
    Or,
}

/// 56 bytes
#[derive(Clone)]
pub struct TypeAscription {
    pub expr: Box<Spanned<Expr>>,
    pub ty: NamedType,
}

#[derive(Clone)]
pub struct Lambda {
    pub args: Spanned<SpannedList<LambdaArgument>>,
    pub body: Box<Spanned<Expr>>,
}

#[derive(Clone)]
pub struct Block {
    pub exprs: SpannedList<Expr>,
    pub ends_with_semicolon: bool,
}

#[derive(Clone)]
pub struct Parens {
    pub exprs: SpannedList<FunCallArgument>,
}

#[derive(Clone, Copy)]
pub struct Empty;

#[derive(Clone)]
pub struct Declaration {
    pub decl_kind: DeclKind,
    pub name: Spanned<Ident>,
    pub value: Box<Spanned<Expr>>,
}

#[derive(Clone)]
pub struct Match {
    pub expr: Box<Spanned<Expr>>,
    pub match_arms: Vec<MatchArm>,
}

#[derive(Clone)]
pub struct FunCallArgument {
    pub name: Option<Spanned<Ident>>,
    pub expr: Spanned<Expr>,
}

#[derive(Clone, Copy)]
pub enum DeclKind {
    Let,
    Var,
}

#[derive(Clone)]
pub struct LambdaArgument {
    pub name: Spanned<Ident>,
    pub ty: Option<Spanned<NamedType>>,
}

#[derive(Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub expr: Expr,
}
