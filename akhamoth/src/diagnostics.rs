use std::{fmt::Display, path::Path};

use crate::source::{SourceMap, Span};

pub enum Context<'a> {
    Span(Span),
    File(&'a Path),
}

/// trait for emitting diagnostic messages
///
/// we do this through a trait instead of a function pointer to allow monomorphization instead of
/// dynamic dispatch and to allow consumers to keep track of diagnostic state.
pub trait EmitDiagnostic {
    fn error(&mut self, source_map: &SourceMap, msg: &dyn Display, ctx: Context);

    fn warn(&mut self, source_map: &SourceMap, msg: &dyn Display, ctx: Context);
}
