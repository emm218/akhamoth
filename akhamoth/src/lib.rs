use std::{fmt::Display, path::PathBuf};

use diagnostics::{Context, Diagnostic, EmitDiagnostic};
use source::{LoadError, SourceFile, SourceMap};
use thiserror::Error;

pub mod diagnostics;
mod lexer;
mod parser;
pub mod source;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error(transparent)]
    Load(#[from] LoadError),
}

#[derive(Default)]
pub struct CompileSession<E: EmitDiagnostic> {
    source_map: SourceMap,
    pub ed: E,
}

impl<E: EmitDiagnostic> CompileSession<E> {
    pub fn new(ed: E) -> Self {
        Self {
            ed,
            source_map: SourceMap::default(),
        }
    }

    pub fn compile(&mut self, path: PathBuf) -> Result<(), CompileError> {
        let source = self.source_map.load_file(&path);

        match source {
            Ok(source) => self.compile_sf(&source),
            Err(e) => {
                self.error(&e, Context::File(&path));
                Err(e.into())
            }
        }
    }

    pub fn compile_sf(&mut self, sf: &SourceFile) -> Result<(), CompileError> {
        let mut parser = parser::Parser::new(self);

        let tokens = lexer::tokenize(sf);
        let ast = parser.parse(tokens);
        Ok(())
    }

    pub fn error(&mut self, msg: &dyn Display, ctx: Context) {
        self.ed
            .emit_diagnostic(&self.source_map, Diagnostic::error(msg, ctx))
    }

    pub fn warn(&mut self, msg: &dyn Display, ctx: Context) {
        self.ed
            .emit_diagnostic(&self.source_map, Diagnostic::warn(msg, ctx))
    }
}
