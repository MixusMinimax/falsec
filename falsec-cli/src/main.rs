use clap::Parser;
use falsec_cli::{Cli, Commands, Run};

fn main() {
    let cli = Cli::parse();

    println!("args: {:?}", cli);

    match cli.command {
        Commands::Run(Run { .. }) => {
            println!("Running program");
        }
        _ => (),
    }
}
