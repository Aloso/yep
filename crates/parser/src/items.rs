use ast::expr::{Block, Expr};
use ast::item::*;
use ast::token::{Ident, Keyword, Punctuation, TokenData, UpperIdent};

use crate::uoret;

use super::helpers::*;
use super::{LexerMut, Parse, ParseResult};

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
        span = span.merge(args.span);

        let return_ty = NamedType::parse(lexer)?;
        span = span.merge_if(&return_ty);

        let body = match lexer.eat(Punctuation::Semicolon) {
            Some(s) => {
                span = span.merge(s);
                None
            }
            None => Some(Block::parse_expect(lexer, "function body")?),
        };
        span = span.merge_if(&body);

        Ok(Some(span.embed(Function { name, generics, args, return_ty, body })))
    }
}

impl Parse for GenericParam {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let name = uoret!(UpperIdent::parse(lexer)?);
        let bounds = Box::<[_]>::from([]);
        Ok(Some(name.span.embed(GenericParam { name, bounds })))
    }
}

impl Parse for NamedType {
    fn parse(rest: LexerMut) -> ParseResult<Self> {
        let name = uoret!(UpperIdent::parse(rest)?);
        let args = parse_type_arguments(rest)?;
        let span = name.span.merge_if(&args);
        let args = args.unwrap_or_default();
        Ok(Some(span.embed(NamedType { name, args })))
    }
}

impl Parse for FunArgument {
    fn parse(rest: LexerMut) -> ParseResult<Self> {
        let (name, mut span) = uoret!(Ident::parse(rest)?).into_inner();
        let ty = NamedType::parse(rest)?;
        span = span.merge_if(&ty);

        let mut fun_arg = FunArgument { name, ty, default: None };
        if rest.eat(Punctuation::Equals).is_some() {
            let expr = Expr::parse_expect(rest, "default value")?;
            span = span.merge(expr.span);
            fun_arg.default = Some(expr);
        }
        Ok(Some(span.embed(fun_arg)))
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
        Ok(Some(lexer.next().span.embed(fun_name)))
    }
}

impl Parse for Class {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let span1 = uoret!(lexer.eat(Keyword::Class));
        let name = UpperIdent::parse_expect(lexer, "class name")?;
        let generics = parse_generics(lexer)?.unwrap_or_default();
        let fields = enclose_multiple_expect(
            ClassField::parse,
            Punctuation::OpenParen,
            Punctuation::Comma,
            Punctuation::CloseParen,
            true,
        )(lexer)?;
        let span2 = lexer.expect(Punctuation::Semicolon)?;

        Ok(Some(span1.merge(span2).embed(Class { name, generics, fields })))
    }
}

impl Parse for ClassField {
    fn parse(rest: LexerMut) -> ParseResult<Self> {
        let name = uoret!(Ident::parse(rest)?);
        let ty = NamedType::parse(rest)?;
        let mut span = name.span.merge_if(&ty);

        let mut class_field = ClassField { name, ty, default: None };
        if rest.eat(Punctuation::Equals).is_some() {
            let expr = Expr::parse_expect(rest, "default value")?;
            span = span.merge(expr.span);
            class_field.default = Some(expr);
        }
        Ok(Some(span.embed(class_field)))
    }
}

impl Parse for Enum {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let span = uoret!(lexer.eat(Keyword::Enum));
        let name = UpperIdent::parse_expect(lexer, "enum name")?;
        let generics = parse_generics(lexer)?.unwrap_or_default();
        let variants = enclose_multiple_expect(
            EnumVariant::parse,
            Punctuation::OpenBrace,
            Punctuation::Comma,
            Punctuation::CloseBrace,
            true,
        )(lexer)?;

        Ok(Some(span.merge(variants.span).embed(Enum { name, generics, variants })))
    }
}

impl Parse for EnumVariant {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let name = uoret!(Ident::parse(lexer)?);
        let arguments = enclose_multiple(
            ClassField::parse,
            Punctuation::OpenParen,
            Punctuation::Comma,
            Punctuation::CloseParen,
            true,
        )(lexer)?;
        Ok(Some(name.span.merge_if(&arguments).embed(EnumVariant { name, arguments })))
    }
}

impl Parse for TypeArgument {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        or2(map(NamedType::parse, TypeArgument::Type), |lexer| {
            let span = uoret!(lexer.eat(Punctuation::Underscore));
            Ok(Some(span.embed(TypeArgument::Wildcard)))
        })(lexer)
    }
}
