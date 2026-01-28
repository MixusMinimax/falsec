use clap::Parser;
use falsec_cli::{Cli, Commands, Compile, Run, TypeSafety};
use falsec_compiler::{CompileRequest, Target, compile};
use falsec_types::Config;
use falsec_types::source::Program;
use std::borrow::Cow;
use std::cell::OnceCell;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write, stdin, stdout};
use std::path::PathBuf;

trait FromArg<T> {
    fn from_arg(t: T) -> Self;
}

impl FromArg<TypeSafety> for falsec_types::TypeSafety {
    fn from_arg(t: TypeSafety) -> Self {
        match t {
            TypeSafety::None => Self::None,
            TypeSafety::Lambda => Self::Lambda,
            TypeSafety::LambdaAndVar => Self::LambdaAndVar,
            TypeSafety::Full => Self::Full,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let mut config = if let Some(config) = cli.config {
        match config
            .extension()
            .map(|e| e.to_str().unwrap().to_ascii_lowercase())
            .as_deref()
        {
            Some("toml") => {
                let config = std::fs::read_to_string(config).unwrap();
                toml::from_str(&config).unwrap()
            }
            Some("json") => serde_json::from_reader(File::open(config).unwrap()).unwrap(),
            Some("json5") => {
                let config = std::fs::read_to_string(config).unwrap();
                json5::from_str(&config).unwrap()
            }
            Some("yml" | "yaml") => serde_yaml::from_reader(File::open(config).unwrap()).unwrap(),
            _ => panic!("Unsupported config file format"),
        }
    } else {
        Config::default()
    };

    match cli.command {
        Commands::Run(Run {
            program,
            type_safety,
        }) => {
            if let Some(type_safety) = type_safety {
                config.type_safety = FromArg::from_arg(type_safety);
            }
            let source_code = read_program(program);
            let program = parse_program(&source_code, &config);
            let interpreter =
                falsec_interpreter::Interpreter::new(stdin(), stdout(), program, config);
            interpreter.run().unwrap();
        }
        Commands::Compile(Compile {
            program,
            out,
            type_safety,
            dump_asm,
        }) => {
            if let Some(type_safety) = type_safety {
                config.type_safety = FromArg::from_arg(type_safety);
            }
            let out_path =
                out.unwrap_or_else(|| PathBuf::from(&program).with_extension("").into_os_string());
            let source_code = read_program(program);
            let program = parse_program(&source_code, &config);
            #[derive(Debug, Default)]
            struct LazyFile<'a>(Cow<'a, OsStr>, OnceCell<File>);
            impl<'a> LazyFile<'a> {
                fn new(p: impl Into<Cow<'a, OsStr>>) -> Self {
                    Self(p.into(), Default::default())
                }

                fn get(&self) -> &File {
                    self.1.get_or_init(|| File::create(&self.0).unwrap())
                }
            }
            impl Write for LazyFile<'_> {
                fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                    self.get().write(buf)
                }
                fn flush(&mut self) -> std::io::Result<()> {
                    self.get().flush()
                }
            }
            compile(CompileRequest {
                source: &source_code,
                program,
                output: LazyFile::new(&out_path),
                target: Target::LinuxX86_64Elf,
                config,
                dump_asm: dump_asm
                    .map(|p| PathBuf::from(p).with_extension("asm").into_os_string())
                    .map(|p| Box::new(LazyFile::new(p)) as _),
            })
            .unwrap();
        }
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
