mod asm;
mod boilerplate;

use crate::error::CompilerError;
use crate::linux_x86_64_elf::asm::{
    Address, Instruction, Label, Operand, Register, RegisterSize, SectionId,
};
use falsec_types::source::{Command, LambdaCommand, Program};
use falsec_types::{Config, TypeSafety};
use falsec_util::string_id;
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

    {
        use crate::linux_x86_64_elf::boilerplate::Boilerplate;
        asm.write_bss(&config)
            .write_error_messages(program.strings)
            .write_print_string(&config)
            .write_flush_stdout()
            .write_print_decimal(&config);
    }

    asm.add_instructions(
        SectionId::Data,
        [Instruction::Label(Label::StdoutLen), Instruction::DW(0)],
    );

    asm.add_instructions(
        SectionId::Text,
        [
            Instruction::Global(Label::Named("_start")),
            Instruction::Global(Label::Named("main")),
            Instruction::Label(Label::Named("_start")),
            Instruction::Label(Label::Named("main")),
        ],
    );
    asm.call(Label::Lambda(program.main_id))
        .call(Label::FlushStdout)
        .exit(0);

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
                    .mov(Register::RAX, i)
                    .push(Register::RAX, ValueType::Number),
                Command::CharLiteral(c) => asm
                    .mov(Register::RAX, c as u64)
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
                Command::Lambda(LambdaCommand::LambdaDefinition(..)) => {
                    return Err(CompilerError::lambda_definition_not_allowed(span.start))
                }
                Command::Lambda(LambdaCommand::LambdaReference(id)) => asm
                    .lea(Register::RAX, Address::b(Label::Lambda(id)))
                    .push(Register::RAX, ValueType::Lambda),
                Command::Exec => asm
                    .pop(Register::RAX, ValueType::Lambda)
                    .call(Register::RAX),
                Command::Conditional => {
                    let label = asm.label_generator.next().unwrap();
                    asm.pop(Register::RAX, ValueType::Lambda) // body
                        .pop(Register::RDX, ValueType::Number) // condition
                        .test(Register::RDX, Register::RDX)
                        .jz(label)
                        .call(Register::RAX)
                        .label(label)
                        .ins(Instruction::Nop)
                }
                Command::While => {
                    let start = asm.label_generator.next().unwrap();
                    let end = asm.label_generator.next().unwrap();
                    asm.pop(Register::RAX, ValueType::Lambda) // body
                        .pop(Register::RDX, ValueType::Lambda) // condition
                        .cpush(Register::RAX)
                        .cpush(Register::RDX)
                        .label(start)
                        .call(Register::RDX)
                        .pop(Register::RAX, ValueType::Number) // condition result
                        .test(Register::RAX, Register::RAX)
                        .jz(end)
                        .mov(Register::RAX, Address::ba(Register::RSP, 8))
                        .call(Register::RAX) // call body
                        .mov(Register::RDX, Address::b(Register::RSP))
                        .jmp(start)
                        .label(end)
                        .add(Register::RSP, 16)
                }
                Command::Var(c) => {
                    if !c.is_ascii_lowercase() {
                        return Err(CompilerError::invalid_variable_name(span.start, c));
                    }
                    asm.push((c as u8 - b'a') as u64, ValueType::Variable)
                }
                Command::Store => {
                    asm.pop(Register::RAX, ValueType::Variable)
                        .pop_any(Register::RDX)
                        .and(Register::RAX, 0b11111)
                        .lea(Register::RBX, Address::b(Label::Variables))
                        .mov(Address::bis(Register::RBX, Register::RAX, 8), Register::RDX);
                    if config.type_safety != TypeSafety::None {
                        asm.lea(Register::RBX, Address::b(Label::VariableTypes))
                            .mov(
                                Address::bi(Register::RBX, Register::RAX),
                                Register::CUR_TYPE,
                            );
                    }
                    &mut asm
                }
                Command::Load => {
                    asm.peek(Register::RAX, ValueType::Variable)
                        .and(Register::RAX, 0b11111)
                        .lea(Register::RBX, Address::b(Label::Variables))
                        .mov(Register::RDX, Address::bis(Register::RBX, Register::RAX, 8));
                    if config.type_safety != TypeSafety::None {
                        asm.lea(Register::RBX, Address::b(Label::VariableTypes))
                            .mov(
                                Register::CUR_TYPE,
                                Address::bi(Register::RBX, Register::RAX),
                            );
                    }
                    asm.replace(Register::RDX, ValueTypeSelector::Current)
                }
                Command::ReadChar => {
                    asm.xor(Register::RAX, Register::RAX) // sys_read
                        .xor(Register::RDI, Register::RDI) // stdin
                        .lea(
                            Register::RSI,
                            Address::bis(Register::STACK_BASE, Register::STACK_COUNTER, 8),
                        ) // buffer
                        .mov(Register::RDX, 1) // size
                        .ins(Instruction::Syscall);
                    if config.type_safety != TypeSafety::None {
                        asm.mov(Register::CUR_TYPE, ValueType::Number.into_id());
                        asm.mov(
                            Address::bi(Register::TYPE_STACK_BASE, Register::STACK_COUNTER),
                            Register::CUR_TYPE,
                        );
                    }
                    let label = asm.label_generator.next().unwrap();
                    asm.test(Register::RAX, Register::RAX)
                        .jnz(label)
                        .mov(
                            Address::bis(Register::STACK_BASE, Register::STACK_COUNTER, 8),
                            -1,
                        )
                        .label(label)
                        .inc(Register::STACK_COUNTER)
                }
                Command::WriteChar => asm
                    .pop(Register::RDI, ValueType::Number)
                    .call(Label::PrintChar),
                Command::StringLiteral(string) => {
                    let id = string_id(&string);
                    asm.mov(Register::RDI, 1) // stdout
                        .lea(Register::RSI, Address::b(Label::StringLiteral(id)))
                        .lea(Register::RDX, Label::StringLiteralLen(id))
                        .call(Label::PrintString)
                }
                Command::WriteInt => asm
                    .pop(Register::RDI, ValueType::Number)
                    .call(Label::PrintDecimal),
                Command::Flush => asm.call(Label::FlushStdout),
                Command::Comment(_) => &mut asm,
            };
        }
        asm.ins(Instruction::Ret);
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

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::Number => write!(f, "number"),
            ValueType::Variable => write!(f, "variable"),
            ValueType::Lambda => write!(f, "lambda"),
        }
    }
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
    ($($(#[$attr:meta])* fn $fun:ident -> $op:ident$(;)*)*) => {
        $(
            $(#[$attr])*
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
    ($($(#[$attr:meta])* fn $fun:ident -> $op:ident$(;)*)*) => {
        $(
            $(#[$attr])*
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
    fn new_label(&mut self) -> Label<'source> {
        self.label_generator.next().unwrap()
    }

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

    fn label(&mut self, label: impl Into<Label<'source>>) -> &mut Self {
        self.ins(Instruction::Label(label.into()))
    }

    fn lea(&mut self, dst: Register, src: impl Into<Operand<'source>>) -> &mut Self {
        self.ins(Instruction::Lea(dst, src.into()))
    }
}

#[allow(dead_code)]
impl<'source> Assembly<'source> {
    binop_fun! {
        fn add -> Add;
        fn and -> And;
        fn cmp -> Cmp;
        fn mov -> Mov;
        fn movzx -> MovZX;
        fn mul -> Mul;
        fn or -> Or;
        fn shl -> Shl;
        fn shr -> Shr;
        fn sub -> Sub;
        fn test -> Test;
        fn xor -> Xor;
    }

    unop_fun! {
        fn call -> Call;
        fn dec -> Dec;
        fn idiv -> IDiv;
        fn inc -> Inc;

        fn jmp -> Jmp;
        /// Jump if equal
        fn je -> Je;
        /// Jump if not equal
        fn jne -> Jne;
        /// Jump if zero
        fn jz -> Jz;
        /// Jump if not zero
        fn jnz -> Jnz;
        /// Jump if sign
        fn js -> Js;
        /// Jump if not sign
        fn jns -> Jns;

        fn neg -> Neg;
        fn not -> Not;
        fn sete -> SetE;
        fn setg -> SetG;

        /// pop from call stack. not to be confused with the data stack.
        #[allow(dead_code)]
        fn cpop -> Pop;
        /// push to call stack. not to be confused with the data stack.
        fn cpush -> Push;
    }
}

fn label_expected_type(value_type: ValueType) -> Label<'static> {
    Label::Named(match value_type {
        ValueType::Number => "err_msg_expected_number",
        ValueType::Variable => "err_msg_expected_variable",
        ValueType::Lambda => "err_msg_expected_lambda",
    })
}

fn label_expected_type_len(value_type: ValueType) -> Label<'static> {
    Label::Named(match value_type {
        ValueType::Number => "err_msg_expected_number_len",
        ValueType::Variable => "err_msg_expected_variable_len",
        ValueType::Lambda => "err_msg_expected_lambda_len",
    })
}

impl<'source> Assembly<'source> {
    fn push(
        &mut self,
        value: impl Into<Operand<'source>>,
        value_type: impl Into<ValueTypeSelector>,
    ) -> &mut Self {
        let value_type = value_type.into();
        assert_ne!(value_type, ValueTypeSelector::Any);
        self.mov(
            Address::bis(Register::STACK_BASE, Register::STACK_COUNTER, 8),
            value.into(),
        );
        if self.config.type_safety != TypeSafety::None {
            if let ValueTypeSelector::ValueType(value_type) = value_type {
                self.mov(Register::CUR_TYPE, value_type.into_id());
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
            .je(label)
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
                    Instruction::DB(_)
                    | Instruction::DW(_)
                    | Instruction::Equ(_)
                    | Instruction::Reserve(..)
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
                Instruction::Cld => write!(current_line, "\tcld")?,
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
                Instruction::DW(i) => write!(current_line, "\tDW {}", i)?,
                Instruction::Dec(operand) => write!(current_line, "\tdec {}", operand)?,
                Instruction::Equ(expr) => write!(current_line, "\tequ {}", expr)?,
                Instruction::Global(symbol) => write!(current_line, "\tglobal {}", symbol)?,
                Instruction::IDiv(operand) => write!(current_line, "\tidiv {}", operand)?,
                Instruction::Inc(operand) => write!(current_line, "\tinc {}", operand)?,

                Instruction::Jmp(operand) => write!(current_line, "\tjmp {}", operand)?,
                Instruction::Je(operand) => write!(current_line, "\tje {}", operand)?,
                Instruction::Jne(operand) => write!(current_line, "\tjne {}", operand)?,
                Instruction::Jz(operand) => write!(current_line, "\tjz {}", operand)?,
                Instruction::Jnz(operand) => write!(current_line, "\tjnz {}", operand)?,
                Instruction::Js(operand) => write!(current_line, "\tjs {}", operand)?,
                Instruction::Jns(operand) => write!(current_line, "\tjns {}", operand)?,

                Instruction::Label(label) => write!(current_line, "{}:", label)?,
                Instruction::Lea(dst, src) => write!(current_line, "\tlea {}, {}", dst, src)?,
                Instruction::Mov(dst, src) => write!(current_line, "\tmov {}, {}", dst, src)?,
                Instruction::MovZX(dst, src) => write!(current_line, "\tmovzx {}, {}", dst, src)?,
                Instruction::Mul(dst, src) => write!(current_line, "\tmul {}, {}", dst, src)?,
                Instruction::Neg(operand) => write!(current_line, "\tneg {}", operand)?,
                Instruction::Nop => write!(current_line, "\tnop")?,
                Instruction::Not(operand) => write!(current_line, "\tnot {}", operand)?,
                Instruction::Pop(operand) => write!(current_line, "\tpop {}", operand)?,
                Instruction::Push(operand) => write!(current_line, "\tpush {}", operand)?,
                Instruction::Or(a, b) => write!(current_line, "\tor {}, {}", a, b)?,
                Instruction::Ret => write!(current_line, "\tret")?,
                Instruction::SetE(operand) => write!(current_line, "\tsete {}", operand)?,
                Instruction::SetG(operand) => write!(current_line, "\tsetg {}", operand)?,
                Instruction::Shl(dst, src) => write!(current_line, "\tshl {}, {}", dst, src)?,
                Instruction::Shr(dst, src) => write!(current_line, "\tshr {}, {}", dst, src)?,
                Instruction::Sub(dst, src) => write!(current_line, "\tsub {}, {}", dst, src)?,
                Instruction::Syscall => write!(current_line, "\tsyscall")?,
                Instruction::Test(a, b) => write!(current_line, "\ttest {}, {}", a, b)?,
                Instruction::Xor(dst, src) => write!(current_line, "\txor {}, {}", dst, src)?,
                Instruction::RepMovsb => write!(current_line, "\trep movsb")?,
                Instruction::RepMovsq => write!(current_line, "\trep movsq")?,
                Instruction::Reserve(size, count) => match size {
                    RegisterSize::L => write!(current_line, "\tresb {}", count)?,
                    RegisterSize::H => panic!("Cannot reserve high byte"),
                    RegisterSize::W => write!(current_line, "\tresw {}", count)?,
                    RegisterSize::E => write!(current_line, "\tresd {}", count)?,
                    RegisterSize::R => write!(current_line, "\tresq {}", count)?,
                },
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
