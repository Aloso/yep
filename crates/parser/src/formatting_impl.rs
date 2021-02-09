use super::formatting::{Beauty, BeautyData, ToBeauty};
use ast::expr::*;
use ast::item::*;
use ast::token::*;
use ast::TinyString;

macro_rules! beauty_impl {
    (struct $name:ident { $($field:ident),* $(,)? }) => {
        impl ToBeauty for $name {
            fn to_beauty(&self) -> Beauty {
                Beauty::kvs(
                    stringify!($name),
                    vec![ $( Beauty::kv(stringify!($field), Beauty::from(&self.$field)) ),* ]
                )
            }
        }
    };
    (enum $name:ident { $($variant:ident),* $(,)? }) => {
        impl ToBeauty for $name {
            fn to_beauty(&self) -> Beauty {
                match self {
                    $( $name::$variant(f) => f.into(), )*
                }
            }
        }
    };
}

impl ToBeauty for NumberLiteral {
    fn to_beauty(&self) -> Beauty { Beauty { data: BeautyData::Number(*self), num: 1 } }
}

impl ToBeauty for StringLiteral {
    fn to_beauty(&self) -> Beauty {
        Beauty { data: BeautyData::String(self.clone()), num: 1 }
    }
}

impl ToBeauty for DeclKind {
    fn to_beauty(&self) -> Beauty {
        match self {
            DeclKind::Let => "Let".to_beauty(),
            DeclKind::Var => "Var".to_beauty(),
        }
    }
}

impl ToBeauty for ScOperator {
    fn to_beauty(&self) -> Beauty {
        match self {
            ScOperator::And => "And".to_beauty(),
            ScOperator::Or => "Or".to_beauty(),
        }
    }
}

impl ToBeauty for TinyString {
    fn to_beauty(&self) -> Beauty {
        Beauty { data: BeautyData::Name(self.clone()), num: 1 }
    }
}

impl ToBeauty for Ident {
    fn to_beauty(&self) -> Beauty { Beauty::kv("Ident", self.inner().to_beauty()) }
}

impl ToBeauty for UpperIdent {
    fn to_beauty(&self) -> Beauty { Beauty::kv("UpperIdent", self.inner().to_beauty()) }
}

impl ToBeauty for Operator {
    fn to_beauty(&self) -> Beauty { Beauty::kv("Operator", self.inner().to_beauty()) }
}

beauty_impl! {
    enum Item { Function, Class, Enum, Impl, Use }
}

beauty_impl! {
    struct Function { name, generics, args, return_ty, body }
}

beauty_impl! {
    struct Class { name, generics, fields }
}

beauty_impl! {
    struct Enum { name, generics, variants }
}

beauty_impl! {
    struct ClassField { name, ty, default }
}

beauty_impl! {
    struct EnumVariant { name, arguments }
}

beauty_impl! {
    struct Impl { generics, r#trait, ty, items }
}

beauty_impl! {
    struct Use { path, wildcard }
}

beauty_impl! {
    enum Name { Ident, Type, Operator }
}

beauty_impl! {
    struct GenericParam { name, bounds }
}

impl ToBeauty for TypeBound {
    fn to_beauty(&self) -> Beauty { match *self {} }
}

beauty_impl! {
    struct FunArgument { name, ty, default }
}

beauty_impl! {
    struct NamedType { name, args }
}

impl ToBeauty for TypeArgument {
    fn to_beauty(&self) -> Beauty {
        match self {
            TypeArgument::Type(f) => f.into(),
            TypeArgument::Wildcard => "Wildcard".to_beauty(),
        }
    }
}

beauty_impl! {
    enum Expr {
        Invokable, Literal, ParenCall, MemberCall, Operation,
        ShortcircuitingOp, Assignment, TypeAscription, Lambda,
        Block, Empty, Declaration, Match, Statement, Tuple
    }
}

beauty_impl! {
    struct Invokable { name, generics }
}

beauty_impl! {
    enum Literal { NumberLit, StringLit }
}

beauty_impl! {
    struct ParenCall { receiver, args }
}

beauty_impl! {
    struct MemberCall { receiver, member }
}

beauty_impl! {
    struct Operation { operator, lhs, rhs }
}

beauty_impl! {
    struct ScOperation { operator, lhs, rhs }
}

beauty_impl! {
    struct Assignment { lhs, rhs }
}

beauty_impl! {
    struct TypeAscription { ty, expr }
}

beauty_impl! {
    struct Lambda { args, body }
}

beauty_impl! {
    struct Block { exprs, ends_with_semicolon }
}

beauty_impl! {
    struct Parens { exprs }
}

beauty_impl! {
    struct FunCallArgument { name, expr }
}

beauty_impl! {
    struct LambdaArgument { name, ty }
}

impl ToBeauty for Empty {
    fn to_beauty(&self) -> Beauty { "Empty".to_beauty() }
}

beauty_impl! {
    struct Declaration { decl_kind, name, value }
}

beauty_impl! {
    struct Match { expr, /* match_arms */ }
}
