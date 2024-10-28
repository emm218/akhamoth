use std::{
    io::{stderr, IsTerminal},
    path::PathBuf,
    process::exit,
};

use akhamoth::{
    diagnostics::{Context, Diagnostic, EmitDiagnostic, Level},
    source::SourceMap,
    CompileSession,
};
use clap::{Parser, ValueEnum};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ColorSetting {
    /// colorize if output goes to a tty
    Auto,
    /// never colorize output
    Never,
    /// alwways colorize output
    Always,
}

#[derive(Parser)]
#[command(version)]
struct Opts {
    /// Input files
    files: Vec<PathBuf>,

    /// whether to colorize output
    #[arg(long, default_value = "auto")]
    color: ColorSetting,
}

fn main() {
    let Opts { files, color } = Opts::parse();

    let color = match color {
        ColorSetting::Always => true,
        ColorSetting::Never => false,
        ColorSetting::Auto => stderr().is_terminal(),
    };

    let ed = Diagnostics::new(color);

    let mut session = CompileSession::new(ed);

    for file in files {
        let _ = session.compile(file);
    }

    let ed = session.ed;

    if ed.errors > 0 {
        let level = if color {
            "\x1b[31;1merror\x1b[0m"
        } else {
            "error"
        };
        eprintln!(
            "{level}: could not compile project due to {} previous errors; {} warnings emitted",
            ed.errors, ed.warnings
        );
        exit(1)
    } else if ed.warnings > 0 {
        let level = if color {
            "\x1b[33;1mwarning\x1b[0m"
        } else {
            "warning"
        };
        eprintln!("{level}: {} warnings emitted", ed.warnings);
    }
}

struct Diagnostics {
    pub errors: usize,
    pub warnings: usize,
    color: bool,
}

impl Diagnostics {
    pub fn new(color: bool) -> Self {
        Self {
            errors: 0,
            warnings: 0,
            color,
        }
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
                if self.color {
                    "\x1b[31;1merror\x1b[0m"
                } else {
                    "error"
                }
            }
            Level::Warning => {
                self.warnings += 1;
                if self.color {
                    "\x1b[33;1mwarning\x1b[0m"
                } else {
                    "warning"
                }
            }
        };

        match ctx {
            Context::Span(ctx) => eprintln!("{level}: {}: {msg}", source_map.span_to_location(ctx)),
            Context::File(path) => {
                eprintln!("{level}\x1b[0m: {}: {msg}", path.display())
            }
        }
    }
}
