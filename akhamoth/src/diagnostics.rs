use std::{fmt, path::Path};

use crate::source::{SourceMap, Span};

pub enum Context<'a> {
    Source { span: Span, src: &'a SourceMap },
    File(&'a Path),
}

/// trait for emitting diagnostic messages
///
/// we do this through a trait instead of a function pointer to allow monomorphization instead of
/// dynamic dispatch and to allow consumers to keep track of diagnostic state.
pub trait EmitDiagnostic {
    fn error(&mut self, fmt: fmt::Arguments, ctx: Context);

    fn warn(&mut self, fmt: fmt::Arguments, ctx: Context);
}

macro_rules! error {
    ($diag:expr, $ctx:expr, $($arg:tt)*) => {
        $diag.error(std::format_args!($($arg)*), $ctx)
    }
}

macro_rules! warning {
    ($diag:expr, $ctx:expr, $($arg:tt)*) => {
        $diag.warn(std::format_args!($($arg)*), $ctx)
    }
}

pub(crate) use error;
pub(crate) use warning;
