use std::path::PathBuf;

use akhamoth::{
    diagnostics::{Diagnostic, EmitDiagnostic, Level},
    source::SourceMap,
    CompileSession,
};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(version)]
struct Opts {
    /// Input files
    files: Vec<PathBuf>,
}

fn main() {
    let Opts { files } = Opts::parse();

    let ed = Diagnostics::new();

    let mut session = CompileSession::new(ed);

    for file in files {
        let _ = session.compile(file);
    }

    let ed = session.ed;

    if ed.errors > 0 {
        eprintln!("\x1b[31;1merror\x1b[0m: could not compile project due to {} previous errors; {} warnings emitted", ed.errors, ed.warnings);
    } else if ed.warnings > 0 {
        eprintln!("\x1b[33;1warning\x1b[0m: {} warnings emitted", ed.warnings);
    }
}

#[derive(Default)]
struct Diagnostics {
    pub errors: usize,
    pub warnings: usize,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EmitDiagnostic for Diagnostics {
    fn emit_diagnostic(
        &mut self,
        source_map: &SourceMap,
        Diagnostic { level, msg, ctx }: Diagnostic,
    ) {
        let level = match level {
            Level::Error => {
                self.errors += 1;
                "\x1b[31;1merror"
            }
            Level::Warning => {
                self.warnings += 1;
                "\x1b[33;1merror"
            }
        };

        match ctx {
            akhamoth::diagnostics::Context::Span(ctx) => eprintln!(
                "{level}\x1b[0m: {}: {msg}",
                source_map.span_to_location(ctx)
            ),
            akhamoth::diagnostics::Context::File(path) => {
                eprintln!("{level}\x1b[0m: {}: {msg}", path.display())
            }
        }
    }
}
