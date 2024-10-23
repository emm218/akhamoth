use std::{fs::read_to_string, path::Path};

use thiserror::Error;

mod lexer;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn compile(path: &Path) -> Result<(), CompileError> {
    let source = read_to_string(path)?;

    for token in lexer::tokenize(&source) {
        println!("{token:?}");
    }

    Ok(())
}
