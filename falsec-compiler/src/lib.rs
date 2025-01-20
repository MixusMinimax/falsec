use crate::error::CompilerError;
use falsec_types::source::Program;
use falsec_types::Config;
use std::io;
use std::io::Write;
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

#[derive(Clone, Debug, Default)]
pub struct CompileRequest<'source, Output: Write> {
    pub source: &'source str,
    pub program: Program<'source>,
    pub output: Output,
    pub target: Target,
    pub config: Config,
}

pub fn compile<Output: Write>(
    CompileRequest {
        program,
        target,
        mut output,
        config,
        ..
    }: CompileRequest<Output>,
) -> Result<(), CompilerError> {
    let mut assembly_file = NamedTempFile::new()?;
    match target {
        Target::LinuxX86_64Elf => linux_x86_64_elf::compile(program, &mut assembly_file, config)?,
        t => panic!("Unsupported target: {:?}", t),
    }
    let mut output_file = NamedTempFile::new()?;
    nasm::assemble(assembly_file.path(), output_file.path(), target);
    output_file.flush()?;
    io::copy(&mut output_file, &mut output)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{compile, CompileRequest};
    use falsec_types::source::Program;

    #[test]
    fn test_compile() {
        let program = Program::default();
        let mut output = Vec::<u8>::new();
        compile(CompileRequest {
            source: "",
            program,
            output: &mut output,
            target: super::Target::LinuxX86_64Elf,
            config: Default::default(),
        })
        .unwrap();
        assert_ne!(output.len(), 0);
        std::fs::write("/home/maxib/sample_program", &output).unwrap();
    }
}
