use std::borrow::Cow;

use crate::{
    diagnostics::{error, Context, EmitDiagnostic},
    lexer::{Token, TokenInner},
    source::Span,
    CompileSession,
};

#[derive(Debug)]
pub struct Expr<'src> {
    span: Span,
    inner: ExprInner<'src>,
}

#[derive(Debug)]
pub enum ExprInner<'src> {
    StringLiteral(Cow<'src, str>),
    IntLiteral(i64),
    Identifier(&'src str),
}

pub struct Parser<'sess, E: EmitDiagnostic> {
    session: &'sess mut CompileSession<E>,
}

impl<'sess, E: EmitDiagnostic> Parser<'sess, E> {
    pub fn new(session: &'sess mut CompileSession<E>) -> Self {
        Self { session }
    }

    pub fn parse<'src>(
        &mut self,
        input: impl Iterator<Item = (Token<'src>, bool)>,
    ) -> impl Iterator<Item = Expr<'src>> {
        for (Token { inner, span }, _) in input {
            let ctx = Context::Source {
                span,
                src: &self.session.source_map,
            };
            let d = &mut self.session.diagnostics;
            match inner {
                TokenInner::StringLiteral { unclosed: true, .. } => {
                    error!(d, ctx, "unclosed string literal")
                }
                TokenInner::IntLiteral(Err(e)) => error!(d, ctx, "{e}"),
                TokenInner::Unrecognized => error!(d, ctx, "unrecognized token"),
                _ => (),
            }
        }

        std::iter::from_fn(|| None)
    }
}
