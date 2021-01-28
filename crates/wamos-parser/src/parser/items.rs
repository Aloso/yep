use crate::lexer::{Ident, Keyword, Operator, Punctuation, TokenData, UpperIdent};
use crate::uoret;

use super::{
    expr::{Block, Expr},
    helpers::*,
    LexerMut, Parse, ParseResult,
};

pub enum Item {
    Function(Function),
    Class(Class),
    Enum(Enum),
}

impl Parse for Item {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        or3(
            map(Function::parse, Item::Function),
            map(Class::parse, Item::Class),
            map(Enum::parse, Item::Enum),
        )(lexer)
    }
}

impl Parse for Function {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        uoret!(lexer.eat(Keyword::Fun));

        let name = Name::parse_expect(lexer, "name")?;
        let generics = parse_generics(lexer)?.unwrap_or_default();

        lexer.expect(Punctuation::OpenParen)?;
        let args =
            vec_separated(lexer, FunArgument::parse, Punctuation::Comma)?.unwrap_or_default();
        lexer.expect(Punctuation::CloseParen)?;

        let return_ty = NamedType::parse(lexer)?;

        let body = match lexer.eat(Punctuation::Semicolon) {
            Some(_) => None,
            None => {
                let body = Block::parse_expect(lexer, "function body")?;
                Some(Expr::Block(body))
            }
        };

        Ok(Some(Function {
            name,
            generics,
            args,
            return_ty,
            body,
        }))
    }
}

impl Parse for GenericParam {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let name = uoret!(UpperIdent::parse(lexer)?);
        let bounds = Vec::new();
        Ok(Some(GenericParam { name, bounds }))
    }
}

impl Parse for NamedType {
    fn parse(rest: LexerMut) -> ParseResult<Self> {
        let name = uoret!(UpperIdent::parse(rest)?);
        let args = parse_type_arguments(rest)?.unwrap_or_default();
        Ok(Some(NamedType { name, args }))
    }
}

impl Parse for FunArgument {
    fn parse(rest: LexerMut) -> ParseResult<Self> {
        let name = uoret!(Ident::parse(rest)?);
        let ty = NamedType::parse(rest)?;

        let mut fun_arg = FunArgument {
            name,
            ty,
            default: None,
        };
        if rest.eat(Punctuation::Equals).is_some() {
            fun_arg.default = Some(Expr::parse_expect(rest, "default expression")?);
        }
        Ok(Some(fun_arg))
    }
}

impl Parse for Name {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let fun_name = match lexer.peek().data() {
            TokenData::Ident(name) => Name::Ident(name),
            TokenData::UpperIdent(name) => Name::Type(name),
            TokenData::Operator(name) => Name::Operator(name),
            _ => return Ok(None),
        };
        lexer.next();
        Ok(Some(fun_name))
    }
}

impl Parse for Class {
    fn parse(_rest: LexerMut) -> ParseResult<Self> {
        // TODO
        Ok(None)
    }
}

impl Parse for Enum {
    fn parse(_rest: LexerMut) -> ParseResult<Self> {
        // TODO
        Ok(None)
    }
}

impl Parse for TypeArgument {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        or2(map(NamedType::parse, TypeArgument::Type), |lexer| {
            uoret!(lexer.eat(Punctuation::Underscore));
            Ok(Some(TypeArgument::Wildcard))
        })(lexer)
    }
}

#[derive(Debug)]
pub struct NamedType {
    pub name: UpperIdent,
    pub args: Vec<TypeArgument>,
}

#[derive(Debug)]
pub enum TypeArgument {
    Type(NamedType),
    Wildcard,
    // TODO: Tuple type
}

pub struct Function {
    pub name: Name,
    pub generics: Vec<GenericParam>,
    pub args: Vec<FunArgument>,
    pub return_ty: Option<NamedType>,
    pub body: Option<Expr>,
}

#[derive(Debug, Clone, Copy)]
pub enum Name {
    Operator(Operator),
    Ident(Ident),
    Type(UpperIdent),
}

pub struct FunArgument {
    pub name: Ident,
    pub ty: Option<NamedType>,
    pub default: Option<Expr>,
}

pub struct Class {
    pub name: UpperIdent,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<ClassField>,
}

pub struct GenericParam {
    pub name: UpperIdent,
    pub bounds: Vec<TypeBound>,
}

pub enum TypeBound {
    // TODO: Interface/trait/contract/superclass
}

pub struct ClassField {
    pub name: Ident,
    pub ty: Option<NamedType>,
    pub default: Option<Expr>,
}

pub struct Enum {
    pub name: UpperIdent,
    pub generics: Vec<GenericParam>,
    pub variants: Vec<EnumVariant>,
}

pub struct EnumVariant {
    pub name: Ident,
    pub argument: Option<NamedType>,
}
