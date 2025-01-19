use crate::error::CompilerError;
use crate::CompileRequest;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::io::Write;

pub fn compile<'source, Output: Write>(
    CompileRequest { output, .. }: CompileRequest<'source, Output>,
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
    write_assembly(assembly, output)?;
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
    for Section {
        section_id,
        instructions,
    } in assembly.sections.into_values()
    {
        writeln!(output, "\tSECTION {}", section_id)?;
        let mut current_line = Vec::<u8>::new();
        let mut previous_instruction_was_label = false;
        for instruction in instructions {
            if !current_line.is_empty()
                && match instruction {
                    Instruction::CommentEndOfLine(..) => false,
                    Instruction::DB(..) | Instruction::Equ(..)
                        if previous_instruction_was_label && current_line.len() < 8 =>
                    {
                        false
                    }
                    _ => true,
                }
            {
                output.write_all(&current_line)?;
                writeln!(output)?;
                current_line.clear();
            }
            previous_instruction_was_label = matches!(instruction, Instruction::Label(..));
            match instruction {
                Instruction::Comment(comment) => writeln!(output, "; {}", comment)?,
                Instruction::DB(bytes) => {
                    write!(current_line, "\tDB ")?;
                    let mut in_string = false;
                    let mut first = true;
                    for byte in bytes.iter() {
                        if byte.is_ascii_alphanumeric() || b" ,.!?".contains(byte) {
                            if !in_string {
                                if !first {
                                    write!(current_line, ", ")?;
                                }
                                write!(current_line, "\"")?;
                                in_string = true;
                            }
                            write!(current_line, "{}", *byte as char)?;
                        } else {
                            if in_string {
                                write!(current_line, "\"")?;
                                in_string = false;
                            }
                            if !first {
                                write!(current_line, ", ")?;
                            }
                            write!(current_line, "{:#04x}", byte)?;
                        }
                        first = false;
                    }
                    if in_string {
                        write!(current_line, "\"")?;
                    }
                }
                Instruction::Equ(expr) => write!(current_line, "\tEQU {}", expr)?,
                Instruction::Global(symbol) => write!(current_line, "\tglobal {}", symbol)?,
                Instruction::Label(label) => write!(current_line, "{}:", label)?,
                _ => (),
            }
        }
        if !current_line.is_empty() {
            output.write_all(&current_line)?;
            writeln!(output)?;
            current_line.clear();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::linux_x86_64_elf::compile;
    use crate::CompileRequest;

    #[test]
    fn simple_compile() {
        let mut output = Vec::new();
        compile(CompileRequest {
            output: &mut output,
            source: "\"Hello, World!\"10,\n",
            program: Default::default(),
        })
        .unwrap();
        let asm = String::from_utf8(output).unwrap();
        assert_ne!(asm.len(), 0);
    }
}
