use crate::uoret;
use crate::{
    lexer::{Ident, Keyword, Operator, Punctuation, TokenData, UpperIdent},
    text_range::TextRange,
};

use super::{
    expr::{Block, Expr, ExprData},
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
        let mut span = uoret!(lexer.eat(Keyword::Fun));

        let name = Name::parse_expect(lexer, "name")?;

        let generics = enclose_multiple(
            GenericParam::parse,
            Punctuation::OpenBracket,
            Punctuation::Comma,
            Punctuation::CloseBracket,
            true,
        )(lexer)?
        .unwrap_or_default();

        let args = enclose_multiple_expect(
            FunArgument::parse,
            Punctuation::OpenParen,
            Punctuation::Comma,
            Punctuation::CloseParen,
            true,
        )(lexer)?;
        span = span.merge(args.1);

        let return_ty = NamedType::parse(lexer)?;
        span = span.merge_if(&return_ty);

        let body = match lexer.eat(Punctuation::Semicolon) {
            Some(_) => None,
            None => {
                let (body, body_span) = Block::parse_expect(lexer, "function body")?;
                span = span.merge(body_span);
                Some(Expr::new(ExprData::Block(body), body_span))
            }
        };
        span = span.merge_if_expr(&body);

        Ok(Some((Function { name, generics, args, return_ty, body }, span)))
    }
}

impl Parse for GenericParam {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let (name, span) = uoret!(UpperIdent::parse(lexer)?);
        let bounds = Vec::new();
        Ok(Some((GenericParam { name, bounds }, span)))
    }
}

impl Parse for NamedType {
    fn parse(rest: LexerMut) -> ParseResult<Self> {
        let name = uoret!(UpperIdent::parse(rest)?);
        let (args, args_span) = parse_type_arguments(rest)?.unwrap_or_default();
        let span = name.1.merge(args_span);
        Ok(Some((NamedType { name, args }, span)))
    }
}

impl Parse for FunArgument {
    fn parse(rest: LexerMut) -> ParseResult<Self> {
        let (name, mut span) = uoret!(Ident::parse(rest)?);
        let ty = NamedType::parse(rest)?;
        span = span.merge_if(&ty);

        let mut fun_arg = FunArgument { name, ty, default: None };
        if rest.eat(Punctuation::Equals).is_some() {
            let (expr, def_span) = ExprData::parse_expect(rest, "default expression")?;
            span = span.merge(def_span);
            fun_arg.default = Some(Expr::new(expr, def_span));
        }
        Ok(Some((fun_arg, span)))
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
        let span = lexer.next().span();
        Ok(Some((fun_name, span)))
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
            let span = uoret!(lexer.eat(Punctuation::Underscore));
            Ok(Some((TypeArgument::Wildcard, span)))
        })(lexer)
    }
}

#[derive(Debug, Clone)]
pub struct NamedType {
    pub name: (UpperIdent, TextRange),
    pub args: Vec<(TypeArgument, TextRange)>,
}

#[derive(Debug, Clone)]
pub enum TypeArgument {
    Type(NamedType),
    Wildcard,
    // TODO: Tuple type
}

pub struct Function {
    pub name: (Name, TextRange),
    pub generics: (Vec<(GenericParam, TextRange)>, TextRange),
    pub args: (Vec<(FunArgument, TextRange)>, TextRange),
    pub return_ty: Option<(NamedType, TextRange)>,
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
    pub ty: Option<(NamedType, TextRange)>,
    pub default: Option<Expr>,
}

pub struct Class {
    pub name: UpperIdent,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<ClassField>,
}

pub struct GenericParam {
    pub name: UpperIdent,
    pub bounds: Vec<(TypeBound, TextRange)>,
}

pub enum TypeBound {
    // TODO: Interface/trait/contract/superclass
}

pub struct ClassField {
    pub name: Ident,
    pub ty: Option<NamedType>,
    pub default: Option<ExprData>,
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
