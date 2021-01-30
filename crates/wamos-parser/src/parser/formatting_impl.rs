use string_interner::StringInterner;

use crate::lexer::{Ident, Operator, StringLiteral, UpperIdent};
use crate::{key_values, lexer::NumberLiteral};

use super::expr::*;
use super::formatting::{FancyFormat, FancyKV, FancyList};
use super::items::*;

macro_rules! impl_fancy_format_struct {
    ($name:ident : $s:literal { $( $key:literal => $value:ident ),* $(,)? }) => {
        impl FancyFormat for $name {
            fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
                key_values!($s {
                    $( FancyKV($key, &self.$value) ),*
                })
                .fmt(buf, indent, interner)
            }

            fn is_single_line(&self) -> bool {
                let mut single_lines = 0;
                $(
                    if !self.$value.is_empty() {
                        if self.$value.is_single_line() {
                            single_lines += 1;
                        } else {
                            return false;
                        }
                        if single_lines > 1 {
                            return false;
                        }
                    }
                )*
                single_lines == 1
            }

            fn is_empty(&self) -> bool {
                $(
                    if !self.$value.is_empty() {
                        return false;
                    }
                )*
                true
            }
        }
    };
}

impl FancyFormat for Item {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        match self {
            Item::Function(x) => x.fmt(buf, indent, interner),
            Item::Class(x) => x.fmt(buf, indent, interner),
            Item::Enum(x) => x.fmt(buf, indent, interner),
        }
    }
    fn is_single_line(&self) -> bool {
        match self {
            Item::Function(x) => x.is_single_line(),
            Item::Class(x) => x.is_single_line(),
            Item::Enum(x) => x.is_single_line(),
        }
    }
}

impl_fancy_format_struct! {
    Function: "Function" {
        "name" => name,
        "generics" => generics,
        "args" => args,
        "return_ty" => return_ty,
        "body" => body,
    }
}

impl_fancy_format_struct! {
    Class: "Class" {
        "name" => name,
        "generics" => generics,
        // "fields" => fields,
    }
}

impl_fancy_format_struct! {
    Enum: "Enum" {
        "name" => name,
        "generics" => generics,
        // "variants" => variants,
    }
}

impl FancyFormat for Name {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        match self {
            Name::Operator(x) => x.fmt(buf, indent, interner),
            Name::Ident(x) => x.fmt(buf, indent, interner),
            Name::Type(x) => x.fmt(buf, indent, interner),
        }
    }
    fn is_single_line(&self) -> bool { true }
}

impl FancyFormat for Operator {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        self.lookup(interner).unwrap().fmt(buf, indent, interner);
    }
    fn is_single_line(&self) -> bool { true }
}
impl FancyFormat for Ident {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        self.lookup(interner).unwrap().fmt(buf, indent, interner);
    }
    fn is_single_line(&self) -> bool { true }
}
impl FancyFormat for UpperIdent {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        self.lookup(interner).unwrap().fmt(buf, indent, interner);
    }
    fn is_single_line(&self) -> bool { true }
}

impl_fancy_format_struct! {
    GenericParam: "GenericParam" {
        "name" => name,
        "bounds" => bounds,
    }
}

impl FancyFormat for TypeBound {
    fn fmt_impl(&self, _buf: &mut String, _indent: usize, _interner: &StringInterner) {
        match *self {}
    }
}

impl_fancy_format_struct! {
    FunArgument: "FunArgument" {
        "name" => name,
        "ty" => ty,
        "default" => default,
    }
}

impl_fancy_format_struct! {
    NamedType: "NamedType" {
        "name" => name,
        "args" => args,
    }
}

impl FancyFormat for TypeArgument {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        match self {
            TypeArgument::Type(t) => t.fmt(buf, indent, interner),
            TypeArgument::Wildcard => "Wildcard".fmt(buf, indent, interner),
        }
    }
    fn is_single_line(&self) -> bool {
        match self {
            TypeArgument::Type(t) => t.is_single_line(),
            TypeArgument::Wildcard => true,
        }
    }
}

impl FancyFormat for Expr {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        self.inner.fmt(buf, indent, interner)
    }

    fn is_single_line(&self) -> bool { self.inner.is_single_line() }

    fn is_empty(&self) -> bool { self.inner.is_empty() }
}

impl FancyFormat for ExprData {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        match self {
            ExprData::Invokable(x) => x.fmt(buf, indent, interner),
            ExprData::Literal(x) => x.fmt(buf, indent, interner),
            ExprData::ParenCall(x) => x.fmt(buf, indent, interner),
            ExprData::MemberCall(x) => x.fmt(buf, indent, interner),
            ExprData::Operation(x) => x.fmt(buf, indent, interner),
            ExprData::ShortcircuitingOp(x) => x.fmt(buf, indent, interner),
            ExprData::Assignment(x) => x.fmt(buf, indent, interner),
            ExprData::TypeAscription(x) => x.fmt(buf, indent, interner),
            ExprData::Statement(x) => x.fmt(buf, indent, interner),
            ExprData::Lambda(x) => x.fmt(buf, indent, interner),
            ExprData::Block(x) => x.fmt(buf, indent, interner),
            ExprData::Tuple(x) => x.fmt(buf, indent, interner),
            ExprData::Empty(x) => x.fmt(buf, indent, interner),
            ExprData::Declaration(x) => x.fmt(buf, indent, interner),
            ExprData::Case(x) => x.fmt(buf, indent, interner),
        }
    }

    fn is_single_line(&self) -> bool {
        match self {
            ExprData::Invokable(x) => x.is_single_line(),
            ExprData::Literal(x) => x.is_single_line(),
            ExprData::ParenCall(x) => x.is_single_line(),
            ExprData::MemberCall(x) => x.is_single_line(),
            ExprData::Operation(x) => x.is_single_line(),
            ExprData::ShortcircuitingOp(x) => x.is_single_line(),
            ExprData::Assignment(x) => x.is_single_line(),
            ExprData::TypeAscription(x) => x.is_single_line(),
            ExprData::Statement(x) => x.is_single_line(),
            ExprData::Lambda(x) => x.is_single_line(),
            ExprData::Block(x) => x.is_single_line(),
            ExprData::Tuple(x) => x.is_single_line(),
            ExprData::Empty(x) => x.is_single_line(),
            ExprData::Declaration(x) => x.is_single_line(),
            ExprData::Case(x) => x.is_single_line(),
        }
    }
}

impl_fancy_format_struct! {
    Invokable: "Invokable" {
        "name" => name,
        "generics" => generics,
    }
}

impl FancyFormat for Literal {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        match self {
            Literal::NumberLit(x) => x.fmt(buf, indent, interner),
            Literal::StringLit(x) => x.fmt(buf, indent, interner),
        }
    }
    fn is_single_line(&self) -> bool { true }
}

impl FancyFormat for NumberLiteral {
    fn fmt_impl(&self, buf: &mut String, _indent: usize, _interner: &StringInterner) {
        match self {
            NumberLiteral::Int(x) => buf.push_str(&format!("Int: {}", x)),
            NumberLiteral::UInt(x) => buf.push_str(&format!("UInt: {}", x)),
            NumberLiteral::Float(x) => buf.push_str(&format!("Float: {}", x)),
        }
    }
    fn is_single_line(&self) -> bool { true }
}

impl FancyFormat for StringLiteral {
    fn fmt_impl(&self, buf: &mut String, _indent: usize, interner: &StringInterner) {
        buf.push_str("String: ");
        buf.push_str(self.lookup(interner).unwrap());
    }
    fn is_single_line(&self) -> bool { true }
}

impl_fancy_format_struct! {
    ParenCall: "ParenCall" {
        "receiver" => receiver,
        "args" => args,
    }
}

impl_fancy_format_struct! {
    FunCallArgument: "FunCallArgument" {
        "name" => name,
        "expr" => expr,
    }
}

impl_fancy_format_struct! {
    MemberCall: "MemberCall" {
        "receiver" => receiver,
        "member" => member,
    }
}

impl_fancy_format_struct! {
    Operation: "Operation" {
        "operator" => operator,
        "lhs" => lhs,
        "rhs" => rhs,
    }
}

impl_fancy_format_struct! {
    ScOperation: "ScOperation" {
        "operator" => operator,
        "lhs" => lhs,
        "rhs" => rhs,
    }
}

impl FancyFormat for ScOperator {
    fn fmt_impl(&self, buf: &mut String, _indent: usize, _interner: &StringInterner) {
        match self {
            ScOperator::And => buf.push_str("And"),
            ScOperator::Or => buf.push_str("Or"),
        }
    }
    fn is_single_line(&self) -> bool { true }
}

impl_fancy_format_struct! {
    Assignment: "Assignment" {
        "lhs" => lhs,
        "rhs" => rhs,
    }
}

impl_fancy_format_struct! {
    TypeAscription: "TypeAscription" {
        "ty" => ty,
        "expr" => expr,
    }
}

impl_fancy_format_struct! {
    Lambda: "Lambda" {
        "args" => args,
        "body" => body,
    }
}

impl_fancy_format_struct! {
    LambdaArgument: "LambdaArgument" {
        "name" => name,
        "ty" => ty,
    }
}

impl FancyFormat for Block {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        if FancyList(self.exprs.as_slice()).is_empty() {
            buf.push_str("Block");
        } else {
            FancyKV("Block", FancyList(self.exprs.as_slice())).fmt(buf, indent, interner)
        }
    }

    fn is_single_line(&self) -> bool {
        let list = FancyList(self.exprs.as_slice());
        list.is_empty() || list.is_single_line()
    }

    fn is_empty(&self) -> bool { false }
}

impl FancyFormat for Parens {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        if FancyList(self.exprs.as_slice()).is_empty() {
            buf.push_str("Parens");
        } else {
            FancyKV("Parens", FancyList(self.exprs.as_slice())).fmt(buf, indent, interner)
        }
    }

    fn is_single_line(&self) -> bool {
        let list = FancyList(self.exprs.as_slice());
        list.is_empty() || list.is_single_line()
    }

    fn is_empty(&self) -> bool { false }
}

impl FancyFormat for Empty {
    fn fmt_impl(&self, buf: &mut String, _indent: usize, _interner: &StringInterner) {
        buf.push_str("Empty");
    }
    fn is_single_line(&self) -> bool { true }
}

impl_fancy_format_struct! {
    Declaration: "Declaration" {
        "decl_kind" => decl_kind,
        "name" => name,
        "value" => value,
    }
}

impl FancyFormat for DeclKind {
    fn fmt_impl(&self, buf: &mut String, _indent: usize, _interner: &StringInterner) {
        match self {
            DeclKind::Let => buf.push_str("Let"),
            DeclKind::Var => buf.push_str("Var"),
        }
    }

    fn is_single_line(&self) -> bool { true }
}

impl_fancy_format_struct! {
    Case: "Case" {
        "expr" => expr,
        // "match_arms" => match_arms,
    }
}
