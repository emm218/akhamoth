use std::{
    fmt::Display,
    io::{stderr, IsTerminal},
    path::PathBuf,
    process::exit,
};

use akhamoth::{
    diagnostics::{Context, EmitDiagnostic},
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

    let diagnostics = Diagnostics::new(color);
    if files.is_empty() {
        diagnostics.print_error("no input files provided");
        exit(1);
    }

    let mut session = CompileSession::new(diagnostics);

    for file in files {
        let _ = session.compile(file);
    }

    let diagnostics = session.diagnostics;

    if diagnostics.errors > 0 {
        diagnostics.print_error(&format!(
            "could not compile project due to {} previous errors; {} warnings emitted",
            diagnostics.errors, diagnostics.warnings
        ));
        exit(1)
    } else if diagnostics.warnings > 0 {
        diagnostics.print_warning(&format!("{} warnings emitted", diagnostics.warnings));
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

    pub fn print_error(&self, msg: &str) {
        let level = if self.color {
            "\x1b[31;1merror\x1b[0m"
        } else {
            "error"
        };

        eprintln!("{level}: {msg}")
    }

    pub fn print_warning(&self, msg: &str) {
        let level = if self.color {
            "\x1b[33;1mwarning\x1b[0m"
        } else {
            "warning"
        };

        eprintln!("{level}: {msg}")
    }
}

impl EmitDiagnostic for Diagnostics {
    fn error(&mut self, source_map: &SourceMap, msg: &dyn Display, ctx: Context) {
        self.errors += 1;

        self.print_error(&match ctx {
            Context::Span(ctx) => format!("{}: {msg}", source_map.span_to_location(ctx)),
            Context::File(path) => format!("{}: {msg}", path.display()),
        });
    }

    fn warn(&mut self, source_map: &SourceMap, msg: &dyn Display, ctx: Context) {
        self.warnings += 1;

        self.print_warning(&match ctx {
            Context::Span(ctx) => format!("{}: {msg}", source_map.span_to_location(ctx)),
            Context::File(path) => format!("{}: {msg}", path.display()),
        });
    }
}
