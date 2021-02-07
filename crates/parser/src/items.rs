use ast::expr::{Block, Expr};
use ast::item::*;
use ast::token::{Ident, Keyword, Punctuation, TokenData, UpperIdent};

use crate::uoret;

use super::helpers::*;
use super::{LexerMut, Parse, ParseResult};

impl Parse for Item {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        or5(
            map(Function::parse, Item::Function),
            map(Class::parse, Item::Class),
            map(Enum::parse, Item::Enum),
            map(Impl::parse, Item::Impl),
            map(Use::parse, Item::Use),
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

impl Parse for Impl {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let span1 = uoret!(lexer.eat(Keyword::Impl));
        let generics = parse_generics(lexer)?.unwrap_or_default();
        let r#trait = NamedType::parse_expect(lexer, "type or trait")?;
        let (r#trait, r#type) = if lexer.eat(Keyword::For).is_some() {
            let r#type = NamedType::parse_expect(lexer, "type")?;
            (Some(r#trait), r#type)
        } else {
            (None, r#trait)
        };

        let mut items = Vec::new();
        let items_span1 = lexer.expect(Punctuation::OpenBrace)?;
        while let Some(item) = Item::parse(lexer)? {
            items.push(item);
        }
        let items_span2 = lexer.expect(Punctuation::CloseBrace)?;
        let items = items_span1.merge(items_span2).embed(items.into_boxed_slice());

        Ok(Some(span1.merge(items.span).embed(Impl {
            generics,
            r#trait,
            ty: r#type,
            items,
        })))
    }
}

impl Parse for Use {
    fn parse(lexer: LexerMut) -> ParseResult<Self> {
        let span1 = uoret!(lexer.eat(Keyword::Use));

        let mut names = vec![Name::parse_expect(lexer, "path")?];
        let mut wildcard = None;

        while lexer.eat(Punctuation::Dot).is_some() {
            if let Some(span) = lexer.eat(Punctuation::Underscore) {
                wildcard = Some(span.embed(()));
                break;
            } else {
                names.push(Name::parse_expect(lexer, "path segment")?);
            }
        }
        let fst_segment = names[0].span;
        let lst_segment = names[names.len() - 1].span;
        let path = fst_segment.merge(lst_segment).embed(names.into_boxed_slice());

        let span2 = lexer.expect(Punctuation::Semicolon)?;

        Ok(Some(span1.merge(span2).embed(Use { path, wildcard })))
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
