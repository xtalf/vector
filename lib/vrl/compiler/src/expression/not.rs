use std::fmt;

use diagnostic::{DiagnosticMessage, Label, Note, Urls};

use crate::{
    expression::{Expr, Resolved},
    parser::Node,
    state::{ExternalEnv, LocalEnv},
    value::{Kind, VrlValueConvert},
    Context, Expression, Span, TypeDef,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Not {
    inner: Box<Expr>,
}

impl Not {
    pub fn new(
        node: Node<Expr>,
        not_span: Span,
        state: (&LocalEnv, &ExternalEnv),
    ) -> Result<Not, Error> {
        let (expr_span, expr) = node.take();
        let type_def = expr.type_def(state);

        if !type_def.is_boolean() {
            return Err(Error {
                variant: ErrorVariant::NonBoolean(type_def.into()),
                not_span,
                expr_span,
            });
        }

        Ok(Self {
            inner: Box::new(expr),
        })
    }
}

impl Expression for Not {
    fn resolve(&self, ctx: &mut Context) -> Resolved {
        Ok((!self.inner.resolve(ctx)?.try_boolean()?).into())
    }

    fn type_def(&self, state: (&LocalEnv, &ExternalEnv)) -> TypeDef {
        let type_def = self.inner.type_def(state);
        let fallible = type_def.is_fallible();
        let abortable = type_def.is_abortable();

        TypeDef::boolean()
            .with_fallibility(fallible)
            .with_abortability(abortable)
    }

    #[cfg(feature = "llvm")]
    fn emit_llvm<'ctx>(
        &self,
        _: (&mut LocalEnv, &mut ExternalEnv),
        _: &mut crate::llvm::Context<'ctx>,
    ) -> Result<(), String> {
        todo!()
    }
}

impl fmt::Display for Not {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#"!{}"#, self.inner)
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct Error {
    pub(crate) variant: ErrorVariant,

    not_span: Span,
    expr_span: Span,
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum ErrorVariant {
    #[error("non-boolean negation")]
    NonBoolean(Kind),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#}", self.variant)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.variant)
    }
}

impl DiagnosticMessage for Error {
    fn code(&self) -> usize {
        use ErrorVariant::NonBoolean;

        match &self.variant {
            NonBoolean(..) => 660,
        }
    }

    fn labels(&self) -> Vec<Label> {
        use ErrorVariant::NonBoolean;

        match &self.variant {
            NonBoolean(kind) => vec![
                Label::primary("negation only works on boolean values", self.not_span),
                Label::context(
                    format!("this expression resolves to {}", kind),
                    self.expr_span,
                ),
            ],
        }
    }

    fn notes(&self) -> Vec<Note> {
        use ErrorVariant::NonBoolean;

        match &self.variant {
            NonBoolean(..) => {
                vec![
                    Note::CoerceValue,
                    Note::SeeDocs(
                        "type coercion".to_owned(),
                        Urls::func_docs("#coerce-functions"),
                    ),
                ]
            }
        }
    }
}
