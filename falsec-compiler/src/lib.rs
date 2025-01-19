use falsec_types::source::Program;
use std::io::Write;

mod error;
mod linux_x86_64_elf;

#[derive(Clone, Debug, Default)]
pub struct CompileRequest<'source, Output: Write> {
    pub source: &'source str,
    pub program: Program<'source>,
    pub output: Output,
}

#[cfg(test)]
mod tests {}
