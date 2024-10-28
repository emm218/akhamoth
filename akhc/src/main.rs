use std::path::PathBuf;

use akhamoth::{CompileError, CompileSession};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(version)]
struct Opts {
    /// Input file
    file: PathBuf,
}

fn main() -> Result<(), CompileError> {
    let Opts { file } = Opts::parse();

    let mut session = CompileSession::new();

    session.compile(&file)?;
    session.compile(&file)
}
