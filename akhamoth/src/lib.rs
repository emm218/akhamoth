use std::path::PathBuf;

use source::LoadError;
use thiserror::Error;

mod lexer;
mod source;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error(transparent)]
    Load(#[from] LoadError),
}

#[derive(Default)]
pub struct CompileSession {
    source_map: source::SourceMap,
}

impl CompileSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compile<P: Into<PathBuf>>(&mut self, path: P) -> Result<(), CompileError> {
        let source = self.source_map.load_file(path)?;

        for (lexer::Token { span, .. }, _) in lexer::tokenize(&source) {
            println!("{}", self.source_map.span_to_string(span));
        }

        Ok(())
    }
}
