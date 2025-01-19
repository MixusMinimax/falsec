use crate::error::CompilerError;
use crate::CompileRequest;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::io::Write;

pub fn compile<'source, Output: Write>(
    CompileRequest { .. }: CompileRequest<'source, Output>,
) -> Result<(), CompilerError> {
    let mut assembly = Assembly::<'source>::default();
    assembly.add_instructions(
        SectionId::Data,
        &[
            Instruction::Label(Cow::Borrowed("msg")),
            Instruction::DB(Cow::Borrowed(b"Hello, World!\n")),
            Instruction::Label(Cow::Borrowed("len")),
            Instruction::Equ(Cow::Borrowed("$-msg")),
        ],
    );
    assembly.add_instructions(
        SectionId::Text,
        &[
            Instruction::Global(Cow::Borrowed("_start")),
            Instruction::Global(Cow::Borrowed("main")),
            Instruction::Label(Cow::Borrowed("_start")),
            Instruction::Label(Cow::Borrowed("main")),
            Instruction::Mov(Register::Rax.into(), Operand::Immediate(1)), // sys_write
            Instruction::Mov(Register::Rdi.into(), Operand::Immediate(1)), // stdout
            Instruction::Mov(Register::Rsi.into(), Operand::Label(Cow::Borrowed("msg"))),
            Instruction::Mov(Register::Rdx.into(), Operand::Label(Cow::Borrowed("len"))),
            Instruction::Syscall,
            Instruction::Mov(Register::Rax.into(), Operand::Immediate(60)), // sys_exit
            Instruction::Mov(Register::Rdi.into(), Operand::Immediate(0)),  // exit code
            Instruction::Syscall,
        ],
    );
    Ok(())
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
enum SectionId {
    Bss,
    Comment,
    Data,
    Data1,
    Debug,
    Init,
    Line,
    Note,
    RoData,
    RoData1,
    Text,
}

#[derive(Clone, Debug)]
struct Section<'source> {
    section_id: SectionId,
    instructions: Vec<Instruction<'source>>,
}

#[derive(Clone, Debug, Default)]
struct Assembly<'source> {
    sections: HashMap<SectionId, Section<'source>>,
}

impl<'source> Assembly<'source> {
    fn add_instructions(&mut self, section_id: SectionId, instructions: &[Instruction<'source>]) {
        let section = self.sections.entry(section_id).or_insert_with(|| Section {
            section_id,
            instructions: Vec::new(),
        });
        section.instructions.extend_from_slice(instructions);
    }
}

#[derive(Clone, Debug)]
enum Instruction<'source> {
    Comment(Cow<'source, str>),
    CommentEndOfLine(Cow<'source, str>),
    DB(Cow<'source, [u8]>),
    Equ(Cow<'source, str>),
    Global(Cow<'source, str>),
    Label(Cow<'source, str>),
    Mov(
        /// Destination
        Operand<'source>,
        /// Source
        Operand<'source>,
    ),
    Syscall,
}

#[derive(Clone, Debug)]
enum Operand<'source> {
    Register(Register),
    Immediate(i64),
    Label(Cow<'source, str>),
}

impl<'source> From<Register> for Operand<'source> {
    fn from(register: Register) -> Self {
        Operand::Register(register)
    }
}

#[derive(Clone, Debug)]
enum Register {
    /// Accumulator
    Rax,
    /// Base
    Rbx,
    /// Counter
    Rcx,
    /// Stack Pointer
    Rsp,
    /// Stack Base Pointer
    Rbp,
    /// Destination
    Rdi,
    /// Source
    Rsi,
    /// Data
    Rdx,
}

impl fmt::Display for SectionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SectionId::Bss => write!(f, ".bss"),
            SectionId::Comment => write!(f, ".comment"),
            SectionId::Data => write!(f, ".data"),
            SectionId::Data1 => write!(f, ".data1"),
            SectionId::Debug => write!(f, ".debug"),
            SectionId::Init => write!(f, ".init"),
            SectionId::Line => write!(f, ".line"),
            SectionId::Note => write!(f, ".note"),
            SectionId::RoData => write!(f, ".rodata"),
            SectionId::RoData1 => write!(f, ".rodata1"),
            SectionId::Text => write!(f, ".text"),
        }
    }
}

fn write_assembly(assembly: Assembly, mut output: impl Write) -> Result<(), CompilerError> {
    for Section { section_id, .. } in assembly.sections.into_values() {
        write!(output, "\tSECTION {}", section_id)?;
    }
    Ok(())
}
