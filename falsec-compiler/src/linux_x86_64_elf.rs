use crate::error::CompilerError;
use falsec_types::source::{Command, Program};
use falsec_types::{Config, TypeSafety};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::io::Write;

pub fn compile<Output: Write>(
    program: Program,
    output: Output,
    config: Config,
) -> Result<(), CompilerError> {
    fn lambda_label(id: u64) -> String {
        format!(".L{}", id)
    }
    let mut assembly = Assembly {
        config: config.clone(),
        ..Default::default()
    };
    assembly.add_instructions(
        SectionId::Data,
        [
            Instruction::Label(Cow::Borrowed("msg")),
            Instruction::DB(Cow::Borrowed(b"Hello, World!\n")),
            Instruction::Label(Cow::Borrowed("len")),
            Instruction::Equ(Cow::Borrowed("$-msg")),
        ],
    );
    assembly.add_instructions(
        SectionId::Text,
        [
            Instruction::Global(Cow::Borrowed("_start")),
            Instruction::Global(Cow::Borrowed("main")),
            Instruction::Label(Cow::Borrowed("_start")),
            Instruction::Label(Cow::Borrowed("main")),
            Instruction::Jmp(Operand::Label(lambda_label(program.main_id).into())),
        ],
    );
    for (id, lambda) in program.lambdas {
        assembly.ins(Instruction::Label(Cow::Owned(lambda_label(id))));
        for (command, span) in lambda {
            if config.write_command_comments {
                assembly.ins(Instruction::Comment(Cow::Owned(format!(
                    "-- {} --",
                    span.source,
                ))));
            }
            match command {
                Command::IntLiteral(i) => assembly
                    .mov(Register::RAX, Operand::Immediate(i))
                    .push(Register::RAX, ValueType::Number),
                Command::CharLiteral(c) => assembly
                    .mov(Register::RAX, Operand::Immediate(c as u64))
                    .push(Register::RAX, ValueType::Number),
                _ => todo!(),
            };
        }
    }
    write_assembly(assembly, output)?;
    Ok(())
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
enum ValueType {
    Number,
    Variable,
    Lambda,
    #[default]
    Any,
}

impl ValueType {
    /// Numerical representation is consistent with falsedotnet.
    fn into_id(self) -> u64 {
        match self {
            ValueType::Number => 0,
            ValueType::Variable => 2,
            ValueType::Lambda => 1,
            ValueType::Any => unreachable!(),
        }
    }
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

#[derive(Debug, Default)]
struct Assembly<'source> {
    sections: HashMap<SectionId, Section<'source>>,
    config: Config,
}

impl<'source> Assembly<'source> {
    fn add_instructions(
        &mut self,
        section_id: SectionId,
        instructions: impl AsRef<[Instruction<'source>]>,
    ) -> &mut Assembly<'source> {
        let section = self.sections.entry(section_id).or_insert_with(|| Section {
            section_id,
            instructions: Vec::new(),
        });
        section
            .instructions
            .extend_from_slice(instructions.as_ref());
        self
    }

    fn ins(&mut self, instruction: Instruction<'source>) -> &mut Assembly<'source> {
        self.add_instructions(SectionId::Text, [instruction])
    }

    fn mov(
        &mut self,
        dst: impl Into<Operand<'source>>,
        src: impl Into<Operand<'source>>,
    ) -> &mut Assembly<'source> {
        self.ins(Instruction::Mov(dst.into(), src.into()))
    }

    fn push(&mut self, register: Register, value_type: ValueType) -> &mut Assembly<'source> {
        assert_ne!(value_type, ValueType::Any);
        self.mov(
            Address {
                base: Register::STACK_BASE,
                index: Some(Register::STACK_COUNTER),
                stride: 8,
                ..Default::default()
            },
            register,
        );
        if self.config.type_safety != TypeSafety::None {
            self.mov(Register::CUR_TYPE, Operand::Immediate(value_type.into_id()))
                .mov(
                    Address {
                        base: Register::TYPE_STACK_BASE,
                        index: Some(Register::STACK_COUNTER),
                        stride: 1,
                        ..Default::default()
                    },
                    Register::CUR_TYPE,
                );
        }
        self.ins(Instruction::Inc(Register::STACK_COUNTER.into()))
    }
}

#[derive(Clone, Debug)]
enum Instruction<'source> {
    Comment(Cow<'source, str>),
    CommentEndOfLine(Cow<'source, str>),
    DB(Cow<'source, [u8]>),
    Equ(Cow<'source, str>),
    Global(Cow<'source, str>),
    Inc(Operand<'source>),
    Jmp(Operand<'source>),
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
    Immediate(u64),
    Label(Cow<'source, str>),
    Address(Address),
}

#[derive(Clone, Debug, Default)]
struct Address {
    base: Register,
    index: Option<Register>,
    index_offset: u64,
    /// 0 is treated as 1, to make the Default derive work. 0 is not a valid value.
    stride: u64,
    address_offset: u64,
}

impl From<Register> for Operand<'_> {
    fn from(register: Register) -> Self {
        Operand::Register(register)
    }
}

impl From<Address> for Operand<'_> {
    fn from(address: Address) -> Self {
        Operand::Address(address)
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
struct Register(RegisterSize, RegisterName);

impl Register {
    const RAX: Self = Self(RegisterSize::R, RegisterName::AX);
    const RBX: Self = Self(RegisterSize::R, RegisterName::BX);
    const RCX: Self = Self(RegisterSize::R, RegisterName::CX);
    const RSP: Self = Self(RegisterSize::R, RegisterName::SP);
    const RBP: Self = Self(RegisterSize::R, RegisterName::BP);
    const RDI: Self = Self(RegisterSize::R, RegisterName::DI);
    const RSI: Self = Self(RegisterSize::R, RegisterName::SI);
    const RDX: Self = Self(RegisterSize::R, RegisterName::DX);

    /// Stack counter used for the data stack. The data stack is separate from the call stack.
    const STACK_COUNTER: Self = Self(RegisterSize::R, RegisterName::R12);

    /// Stack base used for the data stack. The data stack is separate from the call stack.
    const STACK_BASE: Self = Self(RegisterSize::R, RegisterName::R13);

    /// Stack base used for the type stack. The type stack stores [ValueType] instances
    /// for each value on the data stack. Type validation is optional.
    const TYPE_STACK_BASE: Self = Self(RegisterSize::R, RegisterName::R14);

    /// The current type. Used for validation, if enabled.
    const CUR_TYPE: Self = Self(RegisterSize::R, RegisterName::R15);
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
enum RegisterSize {
    /// Low (8-bit)
    L,
    /// High (8-bit)
    H,
    /// Word (16-bit)
    W,
    /// Extended (32-bit)
    E,
    /// Register (64-bit)
    #[default]
    R,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
enum RegisterName {
    /// Accumulator
    #[default]
    AX,
    /// Base
    BX,
    /// Counter
    CX,
    /// Stack Pointer
    SP,
    /// Stack Base Pointer
    BP,
    /// Destination
    DI,
    /// Source
    SI,
    /// Data
    DX,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
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

impl fmt::Display for Operand<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Register(r) => write!(f, "{}", r),
            Operand::Immediate(i) => write!(f, "{}", i),
            Operand::Label(label) => write!(f, "{}", label),
            Operand::Address(address) => write!(f, "{}", address),
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use RegisterName::*;
        use RegisterSize::*;
        match self.1 {
            AX | BX | CX | SP | BP | DI | SI | DX => match self.0 {
                L => write!(f, "{}l", self.1),
                H => write!(f, "{}h", self.1),
                W => write!(f, "{}", self.1),
                E => write!(f, "e{}", self.1),
                R => write!(f, "r{}", self.1),
            },
            R8 | R9 | R10 | R11 | R12 | R13 | R14 | R15 => match self.0 {
                L => write!(f, "{}b", self.1),
                H => unreachable!(),
                W => write!(f, "{}w", self.1),
                E => write!(f, "{}d", self.1),
                R => write!(f, "{}", self.1),
            },
        }
    }
}

impl fmt::Display for RegisterName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RegisterName::AX => write!(f, "ax"),
            RegisterName::BX => write!(f, "bx"),
            RegisterName::CX => write!(f, "cx"),
            RegisterName::SP => write!(f, "sp"),
            RegisterName::BP => write!(f, "bp"),
            RegisterName::DI => write!(f, "di"),
            RegisterName::SI => write!(f, "si"),
            RegisterName::DX => write!(f, "dx"),
            RegisterName::R8 => write!(f, "r8"),
            RegisterName::R9 => write!(f, "r9"),
            RegisterName::R10 => write!(f, "r10"),
            RegisterName::R11 => write!(f, "r11"),
            RegisterName::R12 => write!(f, "r12"),
            RegisterName::R13 => write!(f, "r13"),
            RegisterName::R14 => write!(f, "r14"),
            RegisterName::R15 => write!(f, "r15"),
        }
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{}", self.base)?;
        if let Some(index) = self.index {
            write!(f, "+")?;
            if self.index_offset != 0 {
                write!(f, "({}+{})", index, self.index_offset)?;
            } else {
                write!(f, "{}", index)?;
            }
        }
        if self.stride > 1 {
            write!(f, "*{}", self.stride)?;
        }
        if self.address_offset != 0 {
            write!(f, "+{}", self.address_offset)?;
        }
        write!(f, "]")
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
                Instruction::CommentEndOfLine(_) => todo!(),
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
                Instruction::Equ(expr) => write!(current_line, "\tequ {}", expr)?,
                Instruction::Global(symbol) => write!(current_line, "\tglobal {}", symbol)?,
                Instruction::Inc(operand) => write!(current_line, "\tinc {}", operand)?,
                Instruction::Jmp(_) => todo!(),
                Instruction::Label(label) => write!(current_line, "{}:", label)?,
                Instruction::Mov(dst, src) => write!(current_line, "\tmov {}, {}", dst, src)?,
                Instruction::Syscall => write!(current_line, "\tsyscall")?,
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

    #[test]
    fn simple_compile() {
        let mut output = Vec::new();
        compile(Default::default(), &mut output, Default::default()).unwrap();
        let asm = String::from_utf8(output).unwrap();
        assert_ne!(asm.len(), 0);
    }
}
