use ast::expr::*;
use ast::item::{Class, Enum, Function, Impl, Item, ItemKind, Name, NamedType, Use};
use ast::token::Operator;
use ast::Spanned;

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Expected {0}, got {1}")]
    ExpectedGot(&'static str, &'static str),

    #[error("Invalid function call receiver: {0:?}")]
    InvalidCallReceiver(ExprKind),

    #[error("Invalid member receiver: {0:?}")]
    InvalidMemberReceiver(ExprKind),

    #[error("Invalid operand: {0:?}")]
    InvalidOperand(ExprKind),

    #[error("Named argument after unnamed argument")]
    NamedAfterUnnamed,

    #[error("An argument without a default can't appear after an argument with default")]
    NoDefaultAfterDefault,

    #[error("Named argument not allowed in tuple")]
    NamedArgInTuple,

    #[error("Evaluation order must be disambiguated with a block, e.g. `a + {{b * c}}`")]
    OperationsRequireBlock,

    #[error("This is not a place expression, so it can't be assigned to: {0:?}")]
    NoPlaceExpr(ExprKind),

    #[error("No generics were expected here")]
    UnexpectedGenerics,

    #[error("Function doesn't have a body")]
    ExpectedFunctionBody,

    #[error("Function doesn't have a return type")]
    ExpectedReturnType,

    #[error("Argument doesn't specify its type")]
    ExpectedArgType,

    #[error("impl blocks can't contain {}", match .0 {
        ItemKind::Class => "classes",
        ItemKind::Enum => "enums",
        ItemKind::Impl => "impl blocks",
        ItemKind::Function => "functions",
        ItemKind::Use => "use items",
    })]
    ForbiddenItemInImpl(ItemKind),
}

pub(super) trait Validate {
    type State;
    fn validate(&self, state: Self::State) -> Result<(), ValidationError>;
}

impl<T: Validate> Validate for [T]
where
    T::State: Copy,
{
    type State = T::State;

    fn validate(&self, state: Self::State) -> Result<(), ValidationError> {
        for item in self {
            item.validate(state)?;
        }
        Ok(())
    }
}

impl<T: Validate> Validate for Spanned<T> {
    type State = T::State;

    fn validate(&self, state: Self::State) -> Result<(), ValidationError> {
        self.inner.validate(state)
    }
}

impl Validate for NamedType {
    type State = ();

    fn validate(&self, _: ()) -> Result<(), ValidationError> { Ok(()) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExprPlaceType {
    Place,
    Other,
}

impl Validate for Expr {
    type State = ExprPlaceType;

    fn validate(&self, state: ExprPlaceType) -> Result<(), ValidationError> {
        fn check_place_name(name: &Name) -> Result<(), ValidationError> {
            match name {
                Name::Operator(_) => {
                    Err(ValidationError::ExpectedGot("identifier", "operator"))
                }
                Name::Type(_) => Err(ValidationError::ExpectedGot("identifier", "type")),
                Name::Ident(_) => Ok(()),
            }
        }

        if state == ExprPlaceType::Place {
            match self {
                Expr::Invokable(i) | Expr::MemberCall(MemberCall { member: i, .. }) => {
                    if !i.generics.is_empty() {
                        return Err(ValidationError::UnexpectedGenerics);
                    }
                    check_place_name(&i.name.inner)?;
                }
                _ => return Err(ValidationError::NoPlaceExpr(self.kind())),
            }
        }

        match self {
            Expr::Invokable(i) => i.validate(())?,
            Expr::Literal(_) => {}
            Expr::ParenCall(p) => p.validate(())?,
            Expr::MemberCall(m) => m.validate(())?,
            Expr::Operation(o) => o.validate(())?,
            Expr::ShortcircuitingOp(o) => o.validate(())?,
            Expr::Assignment(a) => a.validate(())?,
            Expr::TypeAscription(t) => t.validate(())?,
            Expr::Statement(s) => s.validate(ExprPlaceType::Other)?,
            Expr::Lambda(l) => l.validate(())?,
            Expr::Block(b) => b.validate(())?,
            Expr::Tuple(t) => t.validate(())?,
            Expr::Empty(_) => {}
            Expr::Declaration(d) => d.validate(())?,
            Expr::Match(c) => c.validate(())?,
        }
        Ok(())
    }
}

impl Validate for ParenCall {
    type State = ();

    fn validate(&self, _: ()) -> Result<(), ValidationError> {
        self.receiver.validate(ExprPlaceType::Other)?;

        let kind = self.receiver.kind();
        match kind {
            | ExprKind::Operation
            | ExprKind::Assignment
            | ExprKind::TypeAscription
            | ExprKind::Statement
            | ExprKind::Empty
            | ExprKind::ShortcircuitingOp
            | ExprKind::Declaration => {
                return Err(ValidationError::InvalidCallReceiver(kind))
            }
            _ => {}
        }

        if let Some(args) = &self.args {
            let mut unnamed_found = false;
            for arg in &**args {
                if arg.name.is_some() {
                    if unnamed_found {
                        return Err(ValidationError::NamedAfterUnnamed);
                    }
                } else {
                    unnamed_found = true;
                }
                arg.validate(())?;
            }
        }
        Ok(())
    }
}

impl Validate for FunCallArgument {
    type State = ();

    fn validate(&self, _: ()) -> Result<(), ValidationError> {
        self.expr.validate(ExprPlaceType::Other)
    }
}

impl Validate for MemberCall {
    type State = ();

    fn validate(&self, _: ()) -> Result<(), ValidationError> {
        self.receiver.validate(ExprPlaceType::Other)?;
        let kind = self.receiver.kind();
        match kind {
            | ExprKind::Operation
            | ExprKind::Assignment
            | ExprKind::TypeAscription
            | ExprKind::Statement
            | ExprKind::Empty
            | ExprKind::ShortcircuitingOp
            | ExprKind::Declaration => {
                return Err(ValidationError::InvalidMemberReceiver(kind))
            }
            _ => {}
        }
        self.member.validate(())?;
        Ok(())
    }
}

impl Validate for Invokable {
    type State = ();

    fn validate(&self, _: ()) -> Result<(), ValidationError> { Ok(()) }
}

impl Validate for Operation {
    type State = ();

    fn validate(&self, _: ()) -> Result<(), ValidationError> {
        ensure_no_operation_except(&self.lhs.inner, &self.operator)?;
        ensure_no_operation_except(&self.rhs.inner, &self.operator)?;
        self.lhs.validate(ExprPlaceType::Other)?;
        self.rhs.validate(ExprPlaceType::Other)?;
        Ok(())
    }
}

fn ensure_no_operation_except(
    expr: &Expr,
    except: &Operator,
) -> Result<(), ValidationError> {
    match expr {
        Expr::Operation(o) if &o.operator != except => {
            return Err(ValidationError::OperationsRequireBlock);
        }
        _ => {}
    }
    let kind = expr.kind();
    match kind {
        | ExprKind::Statement
        | ExprKind::ShortcircuitingOp
        | ExprKind::Assignment
        | ExprKind::Empty
        | ExprKind::Declaration => return Err(ValidationError::InvalidOperand(kind)),
        _ => {}
    }
    Ok(())
}

impl Validate for ScOperation {
    type State = ();

    fn validate(&self, _: ()) -> Result<(), ValidationError> {
        ensure_no_sc_operation_except(&self.lhs.inner, self.operator)?;
        ensure_no_sc_operation_except(&self.rhs.inner, self.operator)?;
        self.lhs.validate(ExprPlaceType::Other)?;
        self.rhs.validate(ExprPlaceType::Other)?;
        Ok(())
    }
}

fn ensure_no_sc_operation_except(
    expr: &Expr,
    except: ScOperator,
) -> Result<(), ValidationError> {
    match expr {
        Expr::ShortcircuitingOp(o) if o.operator != except => {
            return Err(ValidationError::OperationsRequireBlock);
        }
        _ => {}
    }
    let kind = expr.kind();
    match kind {
        | ExprKind::Statement
        | ExprKind::Assignment
        | ExprKind::Empty
        | ExprKind::Declaration => return Err(ValidationError::InvalidOperand(kind)),
        _ => {}
    }
    Ok(())
}

impl Validate for Assignment {
    type State = ();

    fn validate(&self, _: ()) -> Result<(), ValidationError> {
        self.lhs.validate(ExprPlaceType::Place)?;
        self.rhs.validate(ExprPlaceType::Other)?;
        Ok(())
    }
}

impl Validate for TypeAscription {
    type State = ();

    fn validate(&self, _: ()) -> Result<(), ValidationError> {
        self.expr.validate(ExprPlaceType::Other)?;
        self.ty.validate(())?;
        Ok(())
    }
}

impl Validate for Lambda {
    type State = ();

    fn validate(&self, _: ()) -> Result<(), ValidationError> {
        self.args.validate(())?;
        self.body.validate(ExprPlaceType::Other)?;
        Ok(())
    }
}

impl Validate for LambdaArgument {
    type State = ();

    fn validate(&self, _: ()) -> Result<(), ValidationError> { Ok(()) }
}

impl Validate for Block {
    type State = ();

    fn validate(&self, _: ()) -> Result<(), ValidationError> {
        self.exprs.validate(ExprPlaceType::Other)
    }
}

impl Validate for Parens {
    type State = ();

    fn validate(&self, _: ()) -> Result<(), ValidationError> {
        for arg in &*self.exprs {
            if arg.name.is_some() {
                return Err(ValidationError::NamedArgInTuple);
            }
            arg.validate(())?;
        }
        Ok(())
    }
}

impl Validate for Declaration {
    type State = ();

    fn validate(&self, _: ()) -> Result<(), ValidationError> {
        self.value.validate(ExprPlaceType::Other)
    }
}

impl Validate for Match {
    type State = ();

    fn validate(&self, _: ()) -> Result<(), ValidationError> { todo!() }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionType {
    Complete,
    NoBody,
}

impl Validate for Function {
    type State = FunctionType;

    fn validate(&self, state: Self::State) -> Result<(), ValidationError> {
        let mut default_found = false;
        for arg in &**self.args {
            match &arg.ty {
                Some(ty) => ty.validate(())?,
                None => return Err(ValidationError::ExpectedArgType),
            }
            match &arg.default {
                Some(default) => {
                    default_found = true;
                    default.validate(ExprPlaceType::Other)?;
                }
                None if default_found => {
                    return Err(ValidationError::NoDefaultAfterDefault);
                }
                _ => {}
            }
        }

        match &self.return_ty {
            Some(ty) => ty.validate(())?,
            None => return Err(ValidationError::ExpectedReturnType),
        }

        match &self.body {
            Some(b) => b.validate(())?,
            None if state == FunctionType::Complete => {
                return Err(ValidationError::ExpectedFunctionBody);
            }
            _ => {}
        }
        Ok(())
    }
}

impl Validate for Class {
    type State = ();

    fn validate(&self, _: ()) -> Result<(), ValidationError> { Ok(()) }
}

impl Validate for Enum {
    type State = ();

    fn validate(&self, _: ()) -> Result<(), ValidationError> { Ok(()) }
}

impl Validate for Impl {
    type State = ();

    fn validate(&self, _: ()) -> Result<(), ValidationError> {
        for item in self.items.iter() {
            match item.inner.kind() {
                ItemKind::Function => {}
                k => return Err(ValidationError::ForbiddenItemInImpl(k)),
            }
        }
        self.items.validate(())
    }
}

impl Validate for Use {
    type State = ();

    fn validate(&self, _: ()) -> Result<(), ValidationError> { Ok(()) }
}


impl Validate for Item {
    type State = ();

    fn validate(&self, _: ()) -> Result<(), ValidationError> {
        match self {
            Item::Function(f) => f.validate(FunctionType::Complete)?,
            Item::Class(c) => c.validate(())?,
            Item::Enum(e) => e.validate(())?,
            Item::Impl(i) => i.validate(())?,
            Item::Use(i) => i.validate(())?,
        }
        Ok(())
    }
}
