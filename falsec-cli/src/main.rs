use clap::Parser;
use falsec_cli::{Cli, Commands, Run};
use falsec_types::source::Program;
use falsec_types::Config;
use std::ffi::OsString;
use std::io::{stdin, stdout, Read};

fn main() {
    let cli = Cli::parse();

    eprintln!("args: {:?}", cli);

    let config = if let Some(config) = cli.config {
        let config = std::fs::read_to_string(config).unwrap();
        toml::from_str(&config).unwrap()
    } else {
        Config::default()
    };

    match cli.command {
        Commands::Run(Run { program, .. }) => {
            let source_code = read_program(program);
            let program = parse_program(&source_code, &config);
            let interpreter =
                falsec_interpreter::Interpreter::new(stdin(), stdout(), program, config);
            interpreter.run().unwrap();
        }
        Commands::Compile(..) => todo!(),
    }
}

fn read_program(program: OsString) -> String {
    if program == "-" {
        let mut buffer = String::new();
        stdin().read_to_string(&mut buffer).unwrap();
        buffer
    } else {
        std::fs::read_to_string(program).unwrap()
    }
}

fn parse_program<'source>(program: &'source str, config: &Config) -> Program<'source> {
    let parser = falsec_parser::Parser::new(program, config.clone());
    let commands: Result<Vec<_>, _> = parser.collect();
    falsec_analyzer::Analyzer::new(commands.unwrap(), config.clone())
        .analyze()
        .unwrap()
}
