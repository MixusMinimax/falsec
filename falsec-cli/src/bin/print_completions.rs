use clap::CommandFactory;
use clap::Parser;
use clap_complete::{generate, Shell};
use falsec_cli::Cli;
use std::fs::File;
use std::io::{stdout, Write};
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct PrintCompletionsCommand {
    #[arg(short, long)]
    generator: Shell,

    file: Option<PathBuf>,
}

fn main() {
    let args = PrintCompletionsCommand::parse();
    let mut command = Cli::command();

    let mut out: Box<dyn Write> = if let Some(path) = args.file {
        Box::new(File::create(path).unwrap())
    } else {
        Box::new(stdout())
    };

    let name = command.get_name().to_string();
    println!("Generating completions for {}", name);
    generate(args.generator, &mut command, name, &mut out);
}
