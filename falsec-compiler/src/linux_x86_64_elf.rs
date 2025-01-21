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
    let mut asm = Assembly {
        config: config.clone(),
        ..Default::default()
    };
    asm.add_instructions(
        SectionId::Text,
        [
            Instruction::Global(Label::Named(Cow::Borrowed("_start"))),
            Instruction::Global(Label::Named(Cow::Borrowed("main"))),
            Instruction::Label(Label::Named(Cow::Borrowed("_start"))),
            Instruction::Label(Label::Named(Cow::Borrowed("main"))),
        ],
    );
    asm.call(Label::Lambda(program.main_id)).exit(0);

    for (id, lambda) in program.lambdas {
        asm.ins(Instruction::Label(Label::Lambda(id)));
        for (command, span) in lambda {
            if config.write_command_comments {
                asm.ins(Instruction::Comment(Cow::Owned(format!(
                    "-- {} --",
                    span.source,
                ))));
            }
            match command {
                Command::IntLiteral(i) => asm
                    .mov(Register::RAX, Operand::Immediate(i))
                    .push(Register::RAX, ValueType::Number),
                Command::CharLiteral(c) => asm
                    .mov(Register::RAX, Operand::Immediate(c as u64))
                    .push(Register::RAX, ValueType::Number),
                Command::Dup => asm
                    .peek_any(Register::RAX)
                    .push(Register::RAX, ValueTypeSelector::Current),
                Command::Drop => asm.dec(Register::STACK_COUNTER),
                Command::Swap => {
                    asm.mov(
                        Register::RAX,
                        Address::biis(Register::STACK_BASE, Register::STACK_COUNTER, -1, 8),
                    )
                    .mov(
                        Register::RDX,
                        Address::biis(Register::STACK_BASE, Register::STACK_COUNTER, -2, 8),
                    )
                    .mov(
                        Address::biis(Register::STACK_BASE, Register::STACK_COUNTER, -1, 8),
                        Register::RDX,
                    )
                    .mov(
                        Address::biis(Register::STACK_BASE, Register::STACK_COUNTER, -2, 8),
                        Register::RAX,
                    );
                    if config.type_safety != TypeSafety::None {
                        asm.mov(
                            Register::AL,
                            Address::bii(Register::TYPE_STACK_BASE, Register::STACK_COUNTER, -1),
                        )
                        .mov(
                            Register::DL,
                            Address::bii(Register::TYPE_STACK_BASE, Register::STACK_COUNTER, -2),
                        )
                        .mov(
                            Address::bii(Register::TYPE_STACK_BASE, Register::STACK_COUNTER, -1),
                            Register::DL,
                        )
                        .mov(
                            Address::bii(Register::TYPE_STACK_BASE, Register::STACK_COUNTER, -2),
                            Register::AL,
                        );
                    }
                    &mut asm
                }
                Command::Rot => {
                    asm.mov(
                        Register::RAX,
                        Address::biis(Register::STACK_BASE, Register::STACK_COUNTER, -1, 8),
                    )
                    .mov(
                        Register::RDX,
                        Address::biis(Register::STACK_BASE, Register::STACK_COUNTER, -2, 8),
                    )
                    .mov(
                        Register::RSI,
                        Address::biis(Register::STACK_BASE, Register::STACK_COUNTER, -3, 8),
                    )
                    .mov(
                        Address::biis(Register::STACK_BASE, Register::STACK_COUNTER, -1, 8),
                        Register::RSI,
                    )
                    .mov(
                        Address::biis(Register::STACK_BASE, Register::STACK_COUNTER, -2, 8),
                        Register::RAX,
                    )
                    .mov(
                        Address::biis(Register::STACK_BASE, Register::STACK_COUNTER, -3, 8),
                        Register::RDX,
                    );
                    if config.type_safety != TypeSafety::None {
                        asm.mov(
                            Register::AL,
                            Address::bii(Register::TYPE_STACK_BASE, Register::STACK_COUNTER, -1),
                        )
                        .mov(
                            Register::DL,
                            Address::bii(Register::TYPE_STACK_BASE, Register::STACK_COUNTER, -2),
                        )
                        .mov(
                            Register::SIL,
                            Address::bii(Register::TYPE_STACK_BASE, Register::STACK_COUNTER, -3),
                        )
                        .mov(
                            Address::bii(Register::TYPE_STACK_BASE, Register::STACK_COUNTER, -1),
                            Register::SIL,
                        )
                        .mov(
                            Address::bii(Register::TYPE_STACK_BASE, Register::STACK_COUNTER, -2),
                            Register::AL,
                        )
                        .mov(
                            Address::bii(Register::TYPE_STACK_BASE, Register::STACK_COUNTER, -3),
                            Register::DL,
                        );
                    }
                    &mut asm
                }
                Command::Pick => {
                    asm.peek(Register::RAX, ValueType::Number)
                        .mov(Register::RSI, Register::STACK_COUNTER)
                        .sub(Register::RSI, Register::RAX)
                        .mov(
                            Register::RAX,
                            Address::biis(Register::STACK_BASE, Register::RSI, -2, 8),
                        );
                    if config.type_safety != TypeSafety::None {
                        asm.mov(
                            Register::CUR_TYPE,
                            Address::bii(Register::TYPE_STACK_BASE, Register::RSI, -2),
                        );
                    }
                    asm.replace(Register::RAX, ValueTypeSelector::Current)
                }
                Command::Add => asm
                    .pop(Register::RDX, ValueType::Number)
                    .peek(Register::RAX, ValueType::Number)
                    .add(Register::RAX, Register::RDX)
                    .replace(Register::RAX, ValueType::Number),
                Command::Sub => asm
                    .pop(Register::RDX, ValueType::Number)
                    .peek(Register::RAX, ValueType::Number)
                    .sub(Register::RAX, Register::RDX)
                    .replace(Register::RAX, ValueType::Number),
                Command::Mul => asm
                    .pop(Register::RDX, ValueType::Number)
                    .peek(Register::RAX, ValueType::Number)
                    .mul(Register::RAX, Register::RDX)
                    .replace(Register::RAX, ValueType::Number),
                Command::Div => asm
                    .pop(Register::RDI, ValueType::Number)
                    .peek(Register::RAX, ValueType::Number)
                    .ins(Instruction::Cqo)
                    .idiv(Register::RDI)
                    .replace(Register::RAX, ValueType::Number),
                Command::Neg => asm
                    .peek(Register::RAX, ValueType::Number)
                    .neg(Register::RAX)
                    .replace(Register::RAX, ValueType::Number),
                Command::BitAnd => asm
                    .pop(Register::RDX, ValueType::Number)
                    .peek(Register::RAX, ValueType::Number)
                    .and(Register::RAX, Register::RDX)
                    .replace(Register::RAX, ValueType::Number),
                Command::BitOr => asm
                    .pop(Register::RDX, ValueType::Number)
                    .peek(Register::RAX, ValueType::Number)
                    .or(Register::RAX, Register::RDX)
                    .replace(Register::RAX, ValueType::Number),
                Command::BitNot => asm
                    .peek(Register::RAX, ValueType::Number)
                    .not(Register::RAX)
                    .replace(Register::RAX, ValueType::Number),
                Command::Gt => asm
                    .pop(Register::RDX, ValueType::Number)
                    .peek(Register::RAX, ValueType::Number)
                    .cmp(Register::RAX, Register::RDX)
                    .setg(Register::AL)
                    .movzx(Register::RAX, Register::AL)
                    .neg(Register::RAX)
                    .replace(Register::RAX, ValueType::Number),
                Command::Eq => asm
                    .pop(Register::RDX, ValueType::Number)
                    .peek(Register::RAX, ValueType::Number)
                    .cmp(Register::RAX, Register::RDX)
                    .sete(Register::AL)
                    .movzx(Register::RAX, Register::AL)
                    .neg(Register::RAX)
                    .replace(Register::RAX, ValueType::Number),
                _ => todo!(),
            };
        }
    }
    write_assembly(asm, output)?;
    Ok(())
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
enum ValueType {
    Number,
    Variable,
    Lambda,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
enum ValueTypeSelector {
    #[default]
    Current,
    Any,
    ValueType(ValueType),
}

impl From<ValueType> for ValueTypeSelector {
    fn from(value: ValueType) -> Self {
        Self::ValueType(value)
    }
}

impl ValueType {
    /// Numerical representation is consistent with falsedotnet.
    fn into_id(self) -> u64 {
        match self {
            ValueType::Number => 0,
            ValueType::Variable => 2,
            ValueType::Lambda => 1,
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

#[derive(Clone, Debug, Default)]
struct Assembly<'source> {
    sections: HashMap<SectionId, Section<'source>>,
    config: Config,
    label_generator: LabelGenerator,
}

#[derive(Copy, Clone, Debug, Default)]
struct LabelGenerator {
    next_id: u64,
}

impl Iterator for LabelGenerator {
    type Item = Label<'static>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.next_id;
        self.next_id += 1;
        Some(Label::Local(id))
    }
}

/// ```
/// use falsec_compiler::binop_fun;
///
/// binop_fun! {
/// }
/// ```
#[macro_export]
macro_rules! binop_fun {
    ($(fn $fun:ident -> $op:ident$(;)*)*) => {
        $(
            fn $fun(
                &mut self,
                dst: impl Into<Operand<'source>>,
                src: impl Into<Operand<'source>>,
            ) -> &mut Self {
                self.ins(Instruction::$op(dst.into(), src.into()))
            }
        )*
    };
}

/// ```
/// use falsec_compiler::unop_fun;
///
/// unop_fun! {
/// }
/// ```
#[macro_export]
macro_rules! unop_fun {
    ($(fn $fun:ident -> $op:ident$(;)*)*) => {
        $(
            fn $fun(
                &mut self,
                operand: impl Into<Operand<'source>>,
            ) -> &mut Self {
                self.ins(Instruction::$op(operand.into()))
            }
        )*
    };
}

impl<'source> Assembly<'source> {
    fn add_instructions(
        &mut self,
        section_id: SectionId,
        instructions: impl AsRef<[Instruction<'source>]>,
    ) -> &mut Self {
        let section = self.sections.entry(section_id).or_insert_with(|| Section {
            section_id,
            instructions: Vec::new(),
        });
        section
            .instructions
            .extend_from_slice(instructions.as_ref());
        self
    }

    fn ins(&mut self, instruction: Instruction<'source>) -> &mut Self {
        self.add_instructions(SectionId::Text, [instruction])
    }

    fn call(&mut self, label: impl Into<Label<'source>>) -> &mut Self {
        self.ins(Instruction::Call(label.into()))
    }

    fn je(&mut self, label: impl Into<Label<'source>>) -> &mut Self {
        self.ins(Instruction::Je(label.into()))
    }

    fn label(&mut self, label: impl Into<Label<'source>>) -> &mut Self {
        self.ins(Instruction::Label(label.into()))
    }

    fn lea(&mut self, dst: Register, src: impl Into<Operand<'source>>) -> &mut Self {
        self.ins(Instruction::Lea(dst, src.into()))
    }

    binop_fun! {
        fn add -> Add;
        fn and -> And;
        fn cmp -> Cmp;
        fn mov -> Mov;
        fn movzx -> MovZX;
        fn mul -> Mul;
        fn or -> Or;
        fn sub -> Sub;
        fn xor -> Xor;
    }

    unop_fun! {
        fn dec -> Dec;
        fn idiv -> IDiv;
        fn inc -> Inc;
        fn neg -> Neg;
        fn not -> Not;
        fn sete -> SetE;
        fn setg -> SetG;
    }
}

fn label_expected_type(value_type: ValueType) -> Label<'static> {
    Label::Named(Cow::Borrowed(match value_type {
        ValueType::Number => "err_msg_expected_number",
        ValueType::Variable => "err_msg_expected_variable",
        ValueType::Lambda => "err_msg_expected_lambda",
    }))
}

fn label_expected_type_len(value_type: ValueType) -> Label<'static> {
    Label::Named(Cow::Borrowed(match value_type {
        ValueType::Number => "err_msg_expected_number_len",
        ValueType::Variable => "err_msg_expected_variable_len",
        ValueType::Lambda => "err_msg_expected_lambda_len",
    }))
}

impl<'source> Assembly<'source> {
    fn push(&mut self, register: Register, value_type: impl Into<ValueTypeSelector>) -> &mut Self {
        let value_type = value_type.into();
        assert_ne!(value_type, ValueTypeSelector::Any);
        self.mov(
            Address::bis(Register::STACK_BASE, Register::STACK_COUNTER, 8),
            register,
        );
        if self.config.type_safety != TypeSafety::None {
            if let ValueTypeSelector::ValueType(value_type) = value_type {
                self.mov(Register::CUR_TYPE, Operand::Immediate(value_type.into_id()));
            }
            self.mov(
                Address::bi(Register::TYPE_STACK_BASE, Register::STACK_COUNTER),
                Register::CUR_TYPE,
            );
        }
        self.inc(Register::STACK_COUNTER)
    }

    fn peek_any(&mut self, register: Register) -> &mut Self {
        self.mov(
            register,
            Address::biis(Register::STACK_BASE, Register::STACK_COUNTER, -1, 8),
        );
        if self.config.type_safety != TypeSafety::None {
            self.mov(
                Register::CUR_TYPE,
                Address::bii(Register::TYPE_STACK_BASE, Register::STACK_COUNTER, -1),
            );
        }
        self
    }

    fn peek(&mut self, register: Register, value_type: impl Into<ValueTypeSelector>) -> &mut Self {
        let value_type = value_type.into();
        self.peek_any(register);
        if let ValueTypeSelector::ValueType(value_type) = value_type {
            self.verify_current(value_type);
        }
        self
    }

    fn pop_any(&mut self, register: Register) -> &mut Self {
        self.dec(Register::STACK_COUNTER).mov(
            register,
            Address::bis(Register::STACK_BASE, Register::STACK_COUNTER, 8),
        );
        if self.config.type_safety != TypeSafety::None {
            self.mov(
                Register::CUR_TYPE,
                Address::bi(Register::TYPE_STACK_BASE, Register::STACK_COUNTER),
            );
        }
        self
    }

    fn pop(&mut self, register: Register, value_type: impl Into<ValueTypeSelector>) -> &mut Self {
        let value_type = value_type.into();
        self.pop_any(register);
        if let ValueTypeSelector::ValueType(value_type) = value_type {
            self.verify_current(value_type);
        }
        self
    }

    fn replace(
        &mut self,
        register: Register,
        value_type: impl Into<ValueTypeSelector>,
    ) -> &mut Self {
        let value_type = value_type.into();
        assert_ne!(value_type, ValueTypeSelector::Any);
        self.mov(
            Address::biis(Register::STACK_BASE, Register::STACK_COUNTER, -1, 8),
            register,
        );
        if self.config.type_safety != TypeSafety::None {
            if let ValueTypeSelector::ValueType(value_type) = value_type {
                self.mov(Register::CUR_TYPE, value_type.into_id());
            }
            self.mov(
                Address::bii(Register::TYPE_STACK_BASE, Register::STACK_COUNTER, -1),
                Register::CUR_TYPE,
            );
        }
        self
    }

    fn verify_current(&mut self, value_type: ValueType) -> &mut Self {
        match (self.config.type_safety, value_type) {
            (TypeSafety::Lambda, ValueType::Lambda) => (),
            (TypeSafety::LambdaAndVar, ValueType::Lambda | ValueType::Variable) => (),
            (TypeSafety::Full, _) => (),
            _ => return self,
        };
        let label = self.label_generator.next().unwrap();
        self.cmp(Register::CUR_TYPE, value_type.into_id())
            .je(label.clone())
            .mov(Register::RDI, 2) // stderr
            .lea(Register::RSI, label_expected_type(value_type))
            .mov(Register::RDX, label_expected_type_len(value_type))
            .call(Label::PrintString)
            .exit(1)
            .label(label)
    }

    fn exit(&mut self, code: u64) -> &mut Self {
        self.mov(Register::RAX, 60)
            .mov(Register::RDI, code)
            .ins(Instruction::Syscall)
    }
}

#[derive(Clone, Debug)]
enum Instruction<'source> {
    Add(
        /// Destination
        Operand<'source>,
        /// Source
        Operand<'source>,
    ),
    And(Operand<'source>, Operand<'source>),
    Call(Label<'source>),
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
    Dec(Operand<'source>),
    Equ(Cow<'source, str>),
    Global(Label<'source>),
    IDiv(Operand<'source>),
    Inc(Operand<'source>),
    Je(Label<'source>),
    Jmp(Label<'source>),
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
    Not(Operand<'source>),
    Or(Operand<'source>, Operand<'source>),
    SetE(Operand<'source>),
    SetG(Operand<'source>),
    Sub(
        /// Destination
        Operand<'source>,
        /// Source
        Operand<'source>,
    ),
    Syscall,
    Xor(
        /// Destination
        Operand<'source>,
        /// Source
        Operand<'source>,
    ),
}

#[derive(Clone, Debug)]
enum Operand<'source> {
    Register(Register),
    Immediate(u64),
    Label(Label<'source>),
    Address(Address),
}

#[derive(Clone, Debug)]
enum Label<'source> {
    Lambda(u64),
    Local(u64),
    PrintDecimal,
    PrintString,
    PrintChar,
    FlushStdout,
    Named(Cow<'source, str>),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
struct Address {
    base: Register,
    index: Option<Register>,
    index_offset: i64,
    /// 0 is treated as 1, to make the Default derive work. 0 is not a valid value.
    stride: u64,
    address_offset: i64,
}

impl Address {
    /// `[base+index]`
    fn bi(base: Register, index: Register) -> Self {
        Self {
            base,
            index: Some(index),
            ..Default::default()
        }
    }

    /// `[base+(index+index_offset)]`
    fn bii(base: Register, index: Register, index_offset: i64) -> Self {
        Self {
            base,
            index: Some(index),
            index_offset,
            ..Default::default()
        }
    }

    /// `[base+index*stride]`
    fn bis(base: Register, index: Register, stride: u64) -> Self {
        Self {
            base,
            index: Some(index),
            stride,
            ..Default::default()
        }
    }

    /// `[base+(index+index_offset)*stride]`
    fn biis(base: Register, index: Register, index_offset: i64, stride: u64) -> Self {
        Self {
            base,
            index: Some(index),
            index_offset,
            stride,
            ..Default::default()
        }
    }
}

impl From<Register> for Operand<'_> {
    fn from(register: Register) -> Self {
        Operand::Register(register)
    }
}

impl From<u64> for Operand<'_> {
    fn from(value: u64) -> Self {
        Operand::Immediate(value)
    }
}

impl From<Address> for Operand<'_> {
    fn from(address: Address) -> Self {
        Operand::Address(address)
    }
}

impl<'source> From<Label<'source>> for Operand<'source> {
    fn from(value: Label<'source>) -> Self {
        Operand::Label(value)
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

    const AL: Self = Self(RegisterSize::L, RegisterName::AX);
    const DL: Self = Self(RegisterSize::L, RegisterName::DX);
    const SIL: Self = Self(RegisterSize::L, RegisterName::SI);

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
            Operand::Label(label) => write!(f, "[rel {}]", label),
            Operand::Address(address) => write!(f, "{}", address),
        }
    }
}

impl fmt::Display for Label<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Label::Lambda(id) => write!(f, "_lambda_{}", id),
            Label::Local(id) => write!(f, "_local_{:03}", id),
            Label::PrintDecimal => write!(f, "print_decimal"),
            Label::PrintString => write!(f, "print_string"),
            Label::PrintChar => write!(f, "print_char"),
            Label::FlushStdout => write!(f, "flush_stdout"),
            Label::Named(name) => write!(f, "{}", name),
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
                Instruction::Add(dst, src) => write!(current_line, "\tadd {}, {}", dst, src)?,
                Instruction::And(dst, src) => write!(current_line, "\tand {}, {}", dst, src)?,
                Instruction::Call(label) => write!(current_line, "\tcall [rel {}]", label)?,
                Instruction::CMovE(dst, src) => write!(current_line, "\tcmove {}, {}", dst, src)?,
                Instruction::CMovL(dst, src) => write!(current_line, "\tcmovl {}, {}", dst, src)?,
                Instruction::Cmp(a, b) => write!(current_line, "\tcmp {}, {}", a, b)?,
                Instruction::Comment(comment) => writeln!(output, "; {}", comment)?,
                Instruction::CommentEndOfLine(_) => todo!(),
                Instruction::Cqo => write!(current_line, "\tcqo")?,
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
                Instruction::Dec(operand) => write!(current_line, "\tdec {}", operand)?,
                Instruction::Equ(expr) => write!(current_line, "\tequ {}", expr)?,
                Instruction::Global(symbol) => write!(current_line, "\tglobal {}", symbol)?,
                Instruction::IDiv(operand) => write!(current_line, "\tidiv {}", operand)?,
                Instruction::Inc(operand) => write!(current_line, "\tinc {}", operand)?,
                Instruction::Je(label) => write!(current_line, "\tje [rel {}]", label)?,
                Instruction::Jmp(label) => write!(current_line, "\tjmp [rel {}]", label)?,
                Instruction::Label(label) => write!(current_line, "{}:", label)?,
                Instruction::Lea(dst, src) => write!(current_line, "\tlea {}, {}", dst, src)?,
                Instruction::Mov(dst, src) => write!(current_line, "\tmov {}, {}", dst, src)?,
                Instruction::MovZX(dst, src) => write!(current_line, "\tmovzx {}, {}", dst, src)?,
                Instruction::Mul(dst, src) => write!(current_line, "\tmul {}, {}", dst, src)?,
                Instruction::Neg(operand) => write!(current_line, "\tneg {}", operand)?,
                Instruction::Not(operand) => write!(current_line, "\tnot {}", operand)?,
                Instruction::Or(a, b) => write!(current_line, "\tor {}, {}", a, b)?,
                Instruction::SetE(operand) => write!(current_line, "\tsete {}", operand)?,
                Instruction::SetG(operand) => write!(current_line, "\tsetg {}", operand)?,
                Instruction::Sub(dst, src) => write!(current_line, "\tsub {}, {}", dst, src)?,
                Instruction::Syscall => write!(current_line, "\tsyscall")?,
                Instruction::Xor(dst, src) => write!(current_line, "\txor {}, {}", dst, src)?,
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
