use clap::Parser;
use falsec_types::Config;
use schemars::schema_for;
use std::fs::File;
use std::io::{BufWriter, Seek, Write};
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    output: PathBuf,
}

fn main() {
    let args = Args::parse();
    let schema = schema_for!(Config);
    if args.verbose {
        println!(
            "Writing schema to {}",
            args.output.as_os_str().to_str().unwrap()
        );
    }
    let file = File::create(args.output).unwrap();
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &schema).unwrap();
    writer.flush().unwrap();
    if args.verbose {
        println!("Done writing {} bytes", writer.stream_position().unwrap());
    }
}
