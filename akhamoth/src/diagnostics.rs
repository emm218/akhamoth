use std::{fmt::Display, path::Path};

use crate::source::{SourceMap, Span};

pub enum Level {
    Error,
    Warning,
}

pub struct Diagnostic<'a> {
    pub level: Level,
    pub msg: &'a dyn Display,
    pub ctx: Context<'a>,
}

pub enum Context<'a> {
    Span(Span),
    File(&'a Path),
}

impl<'a> Diagnostic<'a> {
    pub fn error(msg: &'a dyn Display, ctx: Context<'a>) -> Self {
        Self {
            level: Level::Error,
            msg,
            ctx,
        }
    }

    pub fn warn(msg: &'a dyn Display, ctx: Context<'a>) -> Self {
        Self {
            level: Level::Warning,
            msg,
            ctx,
        }
    }
}

/// trait for emitting diagnostic messages
///
/// we do this through a trait instead of a function pointer to allow monomorphization instead of
/// dynamic dispatch and to allow consumers to keep track of diagnostic state.
pub trait EmitDiagnostic {
    fn emit_diagnostic(&mut self, source_map: &SourceMap, d: Diagnostic);
}
