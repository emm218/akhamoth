use std::path::PathBuf;

use akhamoth::{compile, CompileError};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(version)]
struct Opts {
    /// Input file
    file: PathBuf,
}

fn main() -> Result<(), CompileError> {
    let Opts { file } = Opts::parse();

    compile(&file)
}
