use std::path::PathBuf;

use thiserror::Error;

pub mod diagnostics;
mod lexer;
mod parser;
pub mod source;

use diagnostics::{error, Context, EmitDiagnostic};
use parser::Parser;
use source::{LoadError, SourceFile, SourceMap};

#[derive(Debug, Error)]
pub enum CompileError {
    #[error(transparent)]
    Load(#[from] LoadError),
}

#[derive(Default)]
pub struct CompileSession<D: EmitDiagnostic> {
    source_map: SourceMap,
    pub diagnostics: D,
}

impl<E: EmitDiagnostic> CompileSession<E> {
    pub fn new(diagnostics: E) -> Self {
        Self {
            diagnostics,
            source_map: SourceMap::default(),
        }
    }

    pub fn compile(&mut self, path: PathBuf) -> Result<(), CompileError> {
        let d = &mut self.diagnostics;
        let source = self.source_map.load_file(&path);

        match source {
            Ok(source) => self.compile_sf(&source),
            Err(e) => {
                error!(d, Context::File(&path), "{e}");
                Err(e.into())
            }
        }
    }

    pub fn compile_sf(&mut self, sf: &SourceFile) -> Result<(), CompileError> {
        let mut parser = Parser::new(self);

        let tokens = lexer::tokenize(sf);
        let ast = parser.parse(tokens);
        Ok(())
    }
}
