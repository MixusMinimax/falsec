use std::borrow::Cow;
use std::fmt;
use std::fmt::Formatter;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Instruction<'source> {
    Align(u64),
    Add(
        /// Destination
        Operand<'source>,
        /// Source
        Operand<'source>,
    ),
    And(Operand<'source>, Operand<'source>),
    Call(Operand<'source>),
    Cld,
    CMovE(
        /// Destination
        Operand<'source>,
        /// Source
        Operand<'source>,
    ),
    CMovL(
        /// Destination
        Operand<'source>,
        /// Source
        Operand<'source>,
    ),
    Cmp(Operand<'source>, Operand<'source>),
    Comment(Cow<'source, str>),
    CommentEndOfLine(Cow<'source, str>),
    Cqo,
    DB(Cow<'source, [u8]>),
    DW(i64),
    DD(i64),
    DQ(i64),
    Dec(Operand<'source>),
    Equ(Cow<'source, str>),
    Global(Label<'source>),
    IDiv(Operand<'source>),
    Inc(Operand<'source>),

    Jmp(Operand<'source>),
    /// Jump if greater
    Jg(Operand<'source>),
    /// Jump if not greater
    Jng(Operand<'source>),
    /// Jump if less
    Jl(Operand<'source>),
    /// Jump if not less
    Jnl(Operand<'source>),
    /// Jump if equal
    Je(Operand<'source>),
    /// Jump if not equal
    Jne(Operand<'source>),
    /// Jump if zero
    Jz(Operand<'source>),
    /// Jump if not zero
    Jnz(Operand<'source>),
    /// Jump if sign
    Js(Operand<'source>),
    /// Jump if not sign
    Jns(Operand<'source>),

    Label(Label<'source>),
    Lea(
        /// Destination
        Register,
        /// Source
        Operand<'source>,
    ),
    Mov(
        /// Destination
        Operand<'source>,
        /// Source
        Operand<'source>,
    ),
    MovZX(
        /// Destination
        Operand<'source>,
        /// Source
        Operand<'source>,
    ),
    Mul(
        /// Destination
        Operand<'source>,
        /// Source
        Operand<'source>,
    ),
    Neg(Operand<'source>),
    Nop,
    Not(Operand<'source>),
    Or(Operand<'source>, Operand<'source>),
    /// pop from call stack. not to be confused with the data stack.
    Pop(Operand<'source>),
    /// push to call stack. not to be confused with the data stack.
    Push(Operand<'source>),
    Ret,
    SetE(Operand<'source>),
    SetG(Operand<'source>),
    Shl(Operand<'source>, Operand<'source>),
    Shr(Operand<'source>, Operand<'source>),
    Sub(
        /// Destination
        Operand<'source>,
        /// Source
        Operand<'source>,
    ),
    Syscall,
    Test(Operand<'source>, Operand<'source>),
    Xor(
        /// Destination
        Operand<'source>,
        /// Source
        Operand<'source>,
    ),
    RepMovsb,
    RepMovsq,
    Reserve(RegisterSize, u64),
}

#[derive(Clone, Debug)]
pub enum Operand<'source> {
    Register(Register),
    Immediate(i64),
    Label(Label<'source>),
    Address(Address<'source>),
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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Label<'source> {
    Lambda(u64),
    Local(u64),
    PrintDecimal,
    /// `void print_string(int fd, const void *buf, size_t count);`
    /// if fd is 1, writes to stout_buffer, if the string is shorter than the buffer size.
    /// if it does, and the remaining space in the buffer is less than the string length,
    /// the buffer is flushed to stdout.
    /// if fd is 2, writes to stderr directly. The stdout buffer is not affected.
    PrintString,
    PrintChar,
    FlushStdout,
    StdoutBuffer,
    StdoutLen,
    DecimalBuffer,
    StringLiteral(u64),
    StringLiteralLen(u64),
    Variables,
    VariableTypes,
    Named(&'source str),
}

impl fmt::Display for Label<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Label::Lambda(id) => write!(f, "_lambda_{:03}", id),
            Label::Local(id) => write!(f, "_local_{:03}", id),
            Label::PrintDecimal => write!(f, "print_decimal"),
            Label::PrintString => write!(f, "print_string"),
            Label::PrintChar => write!(f, "print_char"),
            Label::FlushStdout => write!(f, "flush_stdout"),
            Label::StdoutBuffer => write!(f, "stdout_buffer"),
            Label::StdoutLen => write!(f, "stdout_len"),
            Label::DecimalBuffer => write!(f, "decimal_buffer"),
            Label::StringLiteral(id) => write!(f, "_string_{:03}", id),
            Label::StringLiteralLen(id) => write!(f, "_string_{:03}_len", id),
            Label::Variables => write!(f, "variables"),
            Label::VariableTypes => write!(f, "variable_types"),
            Label::Named(name) => write!(f, "{}", name),
        }
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RegisterOrLabel<'source> {
    Register(Register),
    Label(Label<'source>),
    LabelRel(Label<'source>),
}

impl Default for RegisterOrLabel<'_> {
    fn default() -> Self {
        Self::Register(Default::default())
    }
}

impl<R: Into<Register>> From<R> for RegisterOrLabel<'_> {
    fn from(register: R) -> Self {
        Self::Register(register.into())
    }
}

impl<'source> From<Label<'source>> for RegisterOrLabel<'source> {
    fn from(label: Label<'source>) -> Self {
        Self::Label(label)
    }
}

impl fmt::Display for RegisterOrLabel<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RegisterOrLabel::Register(r) => write!(f, "{}", r),
            RegisterOrLabel::Label(label) => write!(f, "{}", label),
            RegisterOrLabel::LabelRel(label) => write!(f, "rel {}", label),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct Address<'source> {
    pub base: RegisterOrLabel<'source>,
    pub index: Option<Register>,
    pub index_offset: i64,
    /// 0 is treated as 1, to make the Default derive work. 0 is not a valid value.
    pub stride: u64,
    pub address_offset: i64,
    pub size: Option<RegisterSize>,
}

impl fmt::Display for Address<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.size {
            Some(RegisterSize::L) => write!(f, "byte ")?,
            Some(RegisterSize::H) => unreachable!("High byte is not supported"),
            Some(RegisterSize::W) => write!(f, "word ")?,
            Some(RegisterSize::E) => write!(f, "dword ")?,
            Some(RegisterSize::R) => write!(f, "qword ")?,
            None => (),
        }
        write!(f, "[{}", self.base)?;
        if let Some(index) = self.index {
            write!(f, "+")?;
            if self.index_offset != 0 {
                write!(f, "({}+{})", index, self.index_offset)?;
            } else {
                write!(f, "{}", index)?;
            }
            if self.stride > 1 {
                write!(f, "*{}", self.stride)?;
            }
        }
        if self.address_offset != 0 {
            write!(f, "+{}", self.address_offset)?;
        }
        write!(f, "]")
    }
}

impl<'source> Address<'source> {
    /// `[base]`
    pub fn b(base: impl Into<RegisterOrLabel<'source>>) -> Self {
        Self {
            base: base.into(),
            ..Default::default()
        }
    }

    /// `[base+address_offset]`
    pub fn ba(base: impl Into<RegisterOrLabel<'source>>, address_offset: i64) -> Self {
        Self {
            base: base.into(),
            address_offset,
            ..Default::default()
        }
    }

    /// `[base+index]`
    pub fn bi(base: impl Into<RegisterOrLabel<'source>>, index: Register) -> Self {
        Self {
            base: base.into(),
            index: Some(index),
            ..Default::default()
        }
    }

    /// `[base+(index+index_offset)]`
    pub fn bii(
        base: impl Into<RegisterOrLabel<'source>>,
        index: Register,
        index_offset: i64,
    ) -> Self {
        Self {
            base: base.into(),
            index: Some(index),
            index_offset,
            ..Default::default()
        }
    }

    /// `[base+index*stride]`
    pub fn bis(base: impl Into<RegisterOrLabel<'source>>, index: Register, stride: u64) -> Self {
        Self {
            base: base.into(),
            index: Some(index),
            stride,
            ..Default::default()
        }
    }

    /// `[base+(index+index_offset)*stride]`
    pub fn biis(
        base: impl Into<RegisterOrLabel<'source>>,
        index: Register,
        index_offset: i64,
        stride: u64,
    ) -> Self {
        Self {
            base: base.into(),
            index: Some(index),
            index_offset,
            stride,
            ..Default::default()
        }
    }

    pub fn with_size(self, size: impl Into<Option<RegisterSize>>) -> Self {
        Self {
            size: size.into(),
            ..self
        }
    }
}

impl From<Register> for Operand<'_> {
    fn from(register: Register) -> Self {
        Operand::Register(register)
    }
}

impl From<i32> for Operand<'_> {
    fn from(value: i32) -> Self {
        Operand::Immediate(value as i64)
    }
}

impl From<i64> for Operand<'_> {
    fn from(value: i64) -> Self {
        Operand::Immediate(value)
    }
}

impl From<u64> for Operand<'_> {
    fn from(value: u64) -> Self {
        Operand::Immediate(value as i64)
    }
}

impl<'source> From<Address<'source>> for Operand<'source> {
    fn from(address: Address<'source>) -> Self {
        Operand::Address(address)
    }
}

impl<'source> From<Label<'source>> for Operand<'source> {
    fn from(value: Label<'source>) -> Self {
        Operand::Label(value)
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
pub struct Register(pub RegisterSize, pub RegisterName);

impl fmt::Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use RegisterName::*;
        use RegisterSize::*;
        match self.1 {
            AX | BX | CX | DX => match self.0 {
                L => write!(f, "{}l", self.1),
                H => write!(f, "{}h", self.1),
                W => write!(f, "{}x", self.1),
                E => write!(f, "e{}x", self.1),
                R => write!(f, "r{}x", self.1),
            },
            SP | BP | DI | SI => match self.0 {
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

#[allow(dead_code)]
impl Register {
    pub const RAX: Self = Self(RegisterSize::R, RegisterName::AX);
    pub const RBX: Self = Self(RegisterSize::R, RegisterName::BX);
    pub const RCX: Self = Self(RegisterSize::R, RegisterName::CX);
    pub const RSP: Self = Self(RegisterSize::R, RegisterName::SP);
    pub const RBP: Self = Self(RegisterSize::R, RegisterName::BP);
    pub const RDI: Self = Self(RegisterSize::R, RegisterName::DI);
    pub const RSI: Self = Self(RegisterSize::R, RegisterName::SI);
    pub const RDX: Self = Self(RegisterSize::R, RegisterName::DX);

    pub const R8: Self = Self(RegisterSize::R, RegisterName::R8);
    pub const R9: Self = Self(RegisterSize::R, RegisterName::R9);
    pub const R10: Self = Self(RegisterSize::R, RegisterName::R10);

    pub const AL: Self = Self(RegisterSize::L, RegisterName::AX);
    pub const DL: Self = Self(RegisterSize::L, RegisterName::DX);
    pub const DIL: Self = Self(RegisterSize::L, RegisterName::DI);
    pub const SIL: Self = Self(RegisterSize::L, RegisterName::SI);

    /// Stack counter used for the data stack. The data stack is separate from the call stack.
    pub const STACK_COUNTER: Self = Self(RegisterSize::R, RegisterName::R12);

    /// Stack base used for the data stack. The data stack is separate from the call stack.
    pub const STACK_BASE: Self = Self(RegisterSize::R, RegisterName::R13);

    /// Stack base used for the type stack. The type stack stores [ValueType] instances
    /// for each value on the data stack. Type validation is optional.
    pub const TYPE_STACK_BASE: Self = Self(RegisterSize::R, RegisterName::R14);

    /// The current type. Used for validation, if enabled.
    pub const CUR_TYPE: Self = Self(RegisterSize::R, RegisterName::R15);
}

#[allow(dead_code)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
pub enum RegisterSize {
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

#[allow(dead_code)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
pub enum RegisterName {
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

impl fmt::Display for RegisterName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RegisterName::AX => write!(f, "a"),
            RegisterName::BX => write!(f, "b"),
            RegisterName::CX => write!(f, "c"),
            RegisterName::SP => write!(f, "sp"),
            RegisterName::BP => write!(f, "bp"),
            RegisterName::DI => write!(f, "di"),
            RegisterName::SI => write!(f, "si"),
            RegisterName::DX => write!(f, "d"),
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

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[allow(dead_code)]
pub enum SectionId {
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
