use clap::builder::Styles;
use clap::{Parser, Subcommand, ValueEnum, ValueHint};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "falsec", version, about, long_about = None, styles=styles())]
pub struct Cli {
    pub name: Option<String>,

    #[arg(short, long, value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub config: Option<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Run(Run),
    Compile(Compile),
}

#[derive(ValueEnum, Copy, Clone, PartialEq, Eq, Debug)]
pub enum TypeSafety {
    /// No type safety checks are performed.
    None,
    /// When trying to execute a lambda, make sure that the popped value is a lambda.
    Lambda,
    /// Include all checks from [TypeSafety::Lambda], and make sure that when storing or loading
    /// a variable, the popped value is a variable name.
    LambdaAndVar,
    /// Include all checks from [TypeSafety::LambdaAndVar], and ensure that only integers can be
    /// used for arithmetic operations.
    Full,
}

mod run {
    use crate::TypeSafety;
    use clap::{Args, ValueHint};
    use std::ffi::OsString;

    /// Execute a FALSE program
    #[derive(Debug, Args)]
    #[command(version, about, long_about = None)]
    pub struct Run {
        #[arg(long, require_equals = true, value_name = "TYPE", value_enum)]
        pub type_safety: Option<TypeSafety>,

        /// The path to the FALSE program to execute
        #[arg(value_name = "FILE", value_hint = ValueHint::FilePath)]
        pub program: OsString,
    }
}

pub use run::Run;

mod compile {
    use crate::TypeSafety;
    use clap::{Args, ValueHint};
    use std::ffi::OsString;

    /// Compile a FALSE program
    #[derive(Debug, Args)]
    #[command(version, about, long_about = None)]
    pub struct Compile {
        #[arg(long, require_equals = true, value_name = "TYPE", value_enum)]
        pub type_safety: Option<TypeSafety>,

        /// The path to the intermediary assembly
        #[arg(long, value_name = "FILE", value_hint = ValueHint::FilePath)]
        pub dump_asm: Option<OsString>,

        /// The path to the compiled FALSE program
        #[arg(short, long, value_name = "FILE", value_hint = ValueHint::FilePath)]
        pub out: Option<OsString>,

        /// The path to the FALSE program to execute
        #[arg(value_name = "FILE", value_hint = ValueHint::FilePath)]
        pub program: OsString,
    }
}

pub use compile::Compile;

fn styles() -> Styles {
    Styles::styled()
        .usage(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .header(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .literal(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .invalid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .error(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .valid(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .placeholder(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::White))),
        )
}
