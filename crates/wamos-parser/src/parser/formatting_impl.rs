use string_interner::StringInterner;

use crate::lexer::{Ident, Operator, StringLiteral, UpperIdent};
use crate::{key_values, lexer::NumberLiteral};

use super::expr::*;
use super::formatting::{FancyFormat, FancyKV, FancyList};
use super::items::*;

impl FancyFormat for Item {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        match self {
            Item::Function(x) => x.fmt(buf, indent, interner),
            Item::Class(x) => x.fmt(buf, indent, interner),
            Item::Enum(x) => x.fmt(buf, indent, interner),
        }
    }
}

impl FancyFormat for Function {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        key_values!("Function" {
            FancyKV("Name", &self.name),
            FancyKV("Generics", &self.generics),
            FancyKV("Arguments", &self.args),
            FancyKV("ReturnType", &self.return_ty),
            FancyKV("Body", &self.body),
        })
        .fmt(buf, indent, interner)
    }
}

impl FancyFormat for Class {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        key_values!("Class" {
            FancyKV("Name", &self.name),
            FancyKV("Generics", &self.generics),
        //     FancyKV("Fields", &self.fields),
        })
        .fmt(buf, indent, interner)
    }
}

impl FancyFormat for Enum {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        key_values!("Enum" {
            FancyKV("Name", &self.name),
            FancyKV("Generics", &self.generics),
        //     FancyKV("Variants", &self.variants),
        })
        .fmt(buf, indent, interner)
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

impl FancyFormat for GenericParam {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        if self.bounds.is_empty() {
            FancyKV("GenericParam", self.name).fmt(buf, indent, interner)
        } else {
            key_values!("GenericParam" {
                FancyKV("Name", &self.name),
                FancyKV("Bounds", &self.bounds),
            })
            .fmt(buf, indent, interner)
        }
    }
    fn is_single_line(&self) -> bool { self.bounds.is_empty() }
}

impl FancyFormat for TypeBound {
    fn fmt_impl(&self, _buf: &mut String, _indent: usize, _interner: &StringInterner) {
        match *self {}
    }
}

impl FancyFormat for FunArgument {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        key_values!("FunArgument" {
            FancyKV("Name", &self.name),
            FancyKV("Type", &self.ty),
            FancyKV("Default", &self.default),
        })
        .fmt(buf, indent, interner)
    }
}

impl FancyFormat for NamedType {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        if self.args.is_empty() {
            FancyKV("NamedType", self.name).fmt(buf, indent, interner)
        } else {
            key_values!("NamedType" {
                FancyKV("Name", &self.name),
                FancyKV("Arguments", &self.args),
            })
            .fmt(buf, indent, interner)
        }
    }
    fn is_single_line(&self) -> bool { self.args.is_empty() }
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
            TypeArgument::Type(_) => false,
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

impl FancyFormat for Invokable {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        if self.generics.is_empty() {
            FancyKV("Invokable", FancyKV("Name", &self.name)).fmt(buf, indent, interner)
        } else {
            key_values!("Invokable" {
                FancyKV("Name", &self.name),
                FancyKV("Generics", &self.generics),
            })
            .fmt(buf, indent, interner)
        }
    }
    fn is_single_line(&self) -> bool { self.generics.is_empty() }
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

impl FancyFormat for ParenCall {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        key_values!("ParenCall" {
            FancyKV("Receiver", &self.receiver),
            FancyKV("Arguments", &self.args),
        })
        .fmt(buf, indent, interner)
    }
}

impl FancyFormat for FunCallArgument {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        key_values!("FunCallArgument" {
            FancyKV("Name", &self.name),
            FancyKV("Expr", &self.expr),
        })
        .fmt(buf, indent, interner)
    }
}

impl FancyFormat for MemberCall {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        key_values!("MemberCall" {
            FancyKV("Receiver", &self.receiver),
            FancyKV("Member", &self.member),
        })
        .fmt(buf, indent, interner)
    }
}

impl FancyFormat for Operation {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        key_values!("Operation" {
            FancyKV("Op", &self.operator),
            FancyKV("LHS", &self.lhs),
            FancyKV("RHS", &self.rhs),
        })
        .fmt(buf, indent, interner)
    }
}

impl FancyFormat for ScOperation {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        key_values!("SC-operation" {
            FancyKV("Op", &self.operator),
            FancyKV("LHS", &self.lhs),
            FancyKV("RHS", &self.rhs),
        })
        .fmt(buf, indent, interner)
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

impl FancyFormat for Assignment {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        key_values!("Assignment" {
            FancyKV("LHS", &self.lhs),
            FancyKV("RHS", &self.rhs),
        })
        .fmt(buf, indent, interner)
    }
}

impl FancyFormat for TypeAscription {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        key_values!("TypeAscription" {
            FancyKV("Type", &self.ty),
            FancyKV("Expr", &self.expr),
        })
        .fmt(buf, indent, interner)
    }
}

impl FancyFormat for Lambda {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        key_values!("Lambda" {
            FancyKV("Arguments", &self.args),
            FancyKV("Body", &self.body),
        })
        .fmt(buf, indent, interner)
    }
}

impl FancyFormat for LambdaArgument {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        key_values!("LambdaArgument" {
            FancyKV("Name", &self.name),
            FancyKV("Type", &self.ty),
        })
        .fmt(buf, indent, interner)
    }
}

impl FancyFormat for Block {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        FancyKV("Block", FancyList(self.exprs.as_slice())).fmt(buf, indent, interner)
    }
}

impl FancyFormat for Parens {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        FancyKV("Parens", FancyList(self.exprs.as_slice())).fmt(buf, indent, interner)
    }
}

impl FancyFormat for Empty {
    fn fmt_impl(&self, buf: &mut String, _indent: usize, _interner: &StringInterner) {
        buf.push_str("Empty");
    }
    fn is_single_line(&self) -> bool { true }
}

impl FancyFormat for Declaration {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        key_values!("Declaration" {
            FancyKV("Kind", &self.decl_kind),
            FancyKV("Name", &self.name),
            FancyKV("Value", &self.value),
        })
        .fmt(buf, indent, interner)
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

impl FancyFormat for Case {
    fn fmt_impl(&self, buf: &mut String, indent: usize, interner: &StringInterner) {
        key_values!("Case" {
            FancyKV("Expr", &self.expr),
        //     FancyKV("MatchArms", &self.match_arms),
        })
        .fmt(buf, indent, interner)
    }
}
