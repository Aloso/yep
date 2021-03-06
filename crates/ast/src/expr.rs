use std::fmt;

use crate::item::{Name, NamedType, TypeArgument};
use crate::name::Operator;
use crate::pattern::Pattern;
use crate::token::{Ident, NumberLiteral, StringLiteral};
use crate::{Spanned, SpannedList};

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
    Match(Match),
}

impl Expr {
    pub fn to_operator(&self) -> Option<Operator> {
        match self {
            Expr::Invokable(i) => match &*i.name {
                Name::Operator(o) => Some(o.clone()),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn kind(&self) -> ExprKind {
        match self {
            Expr::Invokable(_) => ExprKind::Invokable,
            Expr::Literal(_) => ExprKind::Literal,
            Expr::ParenCall(_) => ExprKind::ParenCall,
            Expr::MemberCall(_) => ExprKind::MemberCall,
            Expr::Operation(_) => ExprKind::Operation,
            Expr::ShortcircuitingOp(_) => ExprKind::ShortcircuitingOp,
            Expr::Assignment(_) => ExprKind::Assignment,
            Expr::TypeAscription(_) => ExprKind::TypeAscription,
            Expr::Statement(_) => ExprKind::Statement,
            Expr::Lambda(_) => ExprKind::Lambda,
            Expr::Block(_) => ExprKind::Block,
            Expr::Tuple(_) => ExprKind::Tuple,
            Expr::Empty(_) => ExprKind::Empty,
            Expr::Declaration(_) => ExprKind::Declaration,
            Expr::Match(_) => ExprKind::Match,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExprKind {
    Invokable,
    Literal,
    ParenCall,
    MemberCall,
    Operation,
    ShortcircuitingOp,
    Assignment,
    TypeAscription,
    Statement,
    Lambda,
    Block,
    Tuple,
    Empty,
    Declaration,
    Match,
}

#[derive(Debug, Clone)]
pub struct Invokable {
    pub name: Spanned<Name>,
    pub generics: Spanned<SpannedList<TypeArgument>>,
}

#[derive(Clone)]
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
pub struct Match {
    pub expr: Box<Spanned<Expr>>,
    pub match_arms: SpannedList<MatchArm>,
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
    pub pattern: Spanned<Pattern>,
    pub expr: Spanned<Expr>,
}

#[derive(Debug, Clone)]
pub struct MatchBody {
    pub arms: SpannedList<MatchArm>,
}
