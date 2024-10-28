use std::borrow::Cow;

use crate::{
    diagnostics::{Context, EmitDiagnostic},
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

    pub fn parse<'src, I: Iterator<Item = (Token<'src>, bool)>>(
        &mut self,
        input: I,
    ) -> impl Iterator<Item = Expr<'src>> {
        for (Token { inner, span }, _) in input {
            match inner {
                TokenInner::StringLiteral { unclosed: true, .. } => self
                    .session
                    .error(&"unclosed string literal", Context::Span(span)),
                TokenInner::IntLiteral(Err(e)) => self.session.error(&e, Context::Span(span)),
                TokenInner::Unrecognized => self
                    .session
                    .error(&"unrecognized token", Context::Span(span)),
                _ => (),
            }
        }

        std::iter::from_fn(|| None)
    }
}
