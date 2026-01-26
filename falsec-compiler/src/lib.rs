use crate::error::CompilerError;
use falsec_types::Config;
use falsec_types::source::Program;
use std::any::Any;
use std::fmt::Debug;
use std::io;
use std::io::{Seek, SeekFrom, Write};
use tempfile::NamedTempFile;

mod error;
mod linux_x86_64_elf;
mod nasm;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub enum Target {
    #[default]
    Auto,
    LinuxX86_64Elf,
}

pub trait Dump: Debug + Write + Any {}
impl<T: Debug + Write + Any> Dump for T {}

#[derive(Debug, Default)]
pub struct CompileRequest<'source, Output: Write> {
    pub source: &'source str,
    pub program: Program<'source>,
    pub output: Output,
    pub target: Target,
    pub config: Config,
    pub dump_asm: Option<Box<dyn Dump>>,
}

pub fn compile<Output: Write>(
    CompileRequest {
        program,
        target,
        mut output,
        config,
        dump_asm,
        ..
    }: CompileRequest<Output>,
) -> Result<(), CompilerError> {
    let mut assembly_file = NamedTempFile::new()?;
    match target {
        Target::LinuxX86_64Elf => linux_x86_64_elf::compile(program, &mut assembly_file, config)?,
        t => panic!("Unsupported target: {:?}", t),
    }
    if let Some(mut dump_asm) = dump_asm {
        assembly_file.seek(SeekFrom::Start(0))?;
        std::io::copy(&mut assembly_file, &mut dump_asm)?;
    }
    let mut output_file = NamedTempFile::new()?;
    nasm::assemble(assembly_file.path(), output_file.path(), target);
    output_file.flush()?;
    io::copy(&mut output_file, &mut output)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{CompileRequest, compile};
    use falsec_types::source::Program;
    use std::collections::HashMap;

    #[test]
    fn test_compile() {
        let program = Program {
            main_id: 0,
            lambdas: HashMap::from([(0, Vec::new())]),
            ..Default::default()
        };
        let mut output = Vec::<u8>::new();
        compile(CompileRequest {
            source: "",
            program,
            output: &mut output,
            target: super::Target::LinuxX86_64Elf,
            config: Default::default(),
            dump_asm: None,
        })
        .unwrap();
        assert_ne!(output.len(), 0);
    }
}
