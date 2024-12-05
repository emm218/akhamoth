use std::{
    fmt::Arguments as FmtArgs,
    io::{stderr, IsTerminal},
    path::PathBuf,
    process::exit,
};

use akhamoth::{
    diagnostics::{Context, EmitDiagnostic},
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

macro_rules! error {
    ($c:expr, $($arg:tt)*) => {
        let level = if $c {
            "\x1b[31;1merror\x1b[0m"
        } else {
            "error"
        };

        std::eprintln!("{level}: {}", std::format_args!($($arg)*));
    }
}

macro_rules! warn {
    ($c:expr, $($arg:tt)*) => {
        let level = if $c {
            "\x1b[33;1mwarning\x1b[0m"
        } else {
            "warning"
        };

        std::eprintln!("{level}: {}", std::format_args!($($arg)*));
    }
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
        error!(color, "no input files provided");
        exit(1);
    }

    let mut session = CompileSession::new(diagnostics);

    for file in files {
        let _ = session.compile(file);
    }

    let Diagnostics {
        errors, warnings, ..
    } = session.diagnostics;

    if errors > 0 {
        error!(color, "could not compile project due to {errors} previous errors; {warnings} warnings emitted");
        exit(1)
    } else if warnings > 0 {
        warn!(color, "{warnings} warnings emitted");
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
    fn error(&mut self, args: FmtArgs, ctx: Context) {
        self.errors += 1;

        error!(self.color, "{ctx}: {args}");
    }

    fn warn(&mut self, args: FmtArgs, ctx: Context) {
        self.warnings += 1;

        warn!(self.color, "{ctx}: {args}");
    }
}
