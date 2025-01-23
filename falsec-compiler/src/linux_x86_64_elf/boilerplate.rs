use crate::linux_x86_64_elf::{
    label_expected_type, label_expected_type_len, Assembly, Instruction, Label, Register,
    RegisterSize, SectionId, ValueType,
};
use falsec_types::Config;
use std::borrow::Cow;
use std::collections::HashMap;

pub(super) trait Boilerplate<'source> {
    fn write_bss(&mut self, config: &Config) -> &mut Self;

    fn write_error_messages(&mut self, strings: HashMap<u64, Cow<'source, str>>) -> &mut Self;

    fn write_print_string(&mut self, config: &Config) -> &mut Self;

    fn write_flush_stdout(&mut self) -> &mut Self;

    fn write_print_decimal(&mut self) -> &mut Self;
}

impl<'source> Boilerplate<'source> for Assembly<'source> {
    fn write_bss(&mut self, config: &Config) -> &mut Self {
        self.add_instructions(
            SectionId::Bss,
            [
                Instruction::Label(Label::Variables),
                Instruction::Reserve(RegisterSize::R, 32),
                Instruction::Label(Label::VariableTypes),
                Instruction::Reserve(RegisterSize::L, 32),
                Instruction::Label(Label::DecimalBuffer),
                Instruction::Reserve(RegisterSize::L, 32),
                Instruction::Label(Label::StdoutBuffer),
                Instruction::Reserve(RegisterSize::L, config.stdout_buffer_size.0),
            ],
        )
    }

    fn write_error_messages(&mut self, strings: HashMap<u64, Cow<'source, str>>) -> &mut Self {
        self.add_instructions(
            SectionId::RoData,
            [Instruction::Comment(Cow::Borrowed("Error messages"))],
        );
        self.add_instructions(
            SectionId::RoData,
            [
                Instruction::Label(Label::Named("mmap_error")),
                Instruction::DB(Cow::Borrowed(b"MMAP Failed! Exiting.\n")),
                Instruction::Label(Label::Named("mmap_error_len")),
                Instruction::Equ(Cow::Owned(format!("$ - {}", Label::Named("mmap_error")))),
            ],
        );
        for t in [ValueType::Number, ValueType::Variable, ValueType::Lambda] {
            self.add_instructions(
                SectionId::RoData,
                [
                    Instruction::Label(label_expected_type(t)),
                    Instruction::DB(Cow::Owned(
                        format!("Tried to pop {} from stack!\n", t).into_bytes(),
                    )),
                    Instruction::Label(label_expected_type_len(t)),
                    Instruction::Equ(Cow::Owned(format!("$ - {}", label_expected_type(t)))),
                ],
            );
        }

        self.add_instructions(
            SectionId::RoData,
            [Instruction::Comment(Cow::Borrowed("String literals"))],
        );
        for (id, string) in strings {
            self.add_instructions(
                SectionId::RoData,
                [
                    Instruction::Label(Label::StringLiteral(id)),
                    Instruction::DB(match string {
                        Cow::Owned(s) => Cow::Owned(s.into_bytes()),
                        Cow::Borrowed(s) => Cow::Borrowed(s.as_bytes()),
                    }),
                    Instruction::Label(Label::StringLiteralLen(id)),
                    Instruction::Equ(Cow::Owned(format!("$ - {}", Label::StringLiteral(id)))),
                ],
            );
        }
        self
    }

    fn write_print_string(&mut self, config: &Config) -> &mut Self {
        // rdi: fd
        // rsi: ptr
        // rdx: len

        let handle_stdout = self.new_label();
        let skip_flush = self.new_label();
        let direct_print_stdout = self.new_label();
        let skip_qwords = self.new_label();
        let return_label = self.new_label();

        self.label(Label::PrintString)
            .ins(Instruction::Comment(Cow::Borrowed(
                "void print_string(int fd, void *buf, size_t len)",
            )))
            // if len is 0, immediately return
            .test(Register::RDX, Register::RDX)
            .jz(return_label)
            // if fd (in rdi) is 1, go to stdout buffering. otherwise, direct syscall.
            .cmp(Register::RDI, 1) // stdout
            .je(handle_stdout)
            .mov(Register::RAX, 1) // sys_write
            .ins(Instruction::Syscall)
            .jmp(return_label)
            .label(handle_stdout)
            .mov(Register::RAX, config.stdout_buffer_size.0)
            .sub(Register::RAX, Register::RDX)
            .cmp(Register::RAX, Label::StdoutLen) // remaining space
            .jns(skip_flush)
            .call(Label::FlushStdout)
            .label(skip_flush)
            .test(Register::RAX, Register::RAX)
            .js(direct_print_stdout)
            // there is enough space in the stdout_buffer. copy the contents of rsi there.
            // vvv
            .add(Label::StdoutLen, Register::RDX)
            .lea(Register::RDI, Label::StdoutBuffer)
            .ins(Instruction::Cld)
            .mov(Register::RCX, Register::RDX)
            .shr(Register::RCX, 3)
            .jz(skip_qwords)
            .ins(Instruction::RepMovsq)
            .label(skip_qwords)
            .mov(Register::RCX, Register::RDX)
            .and(Register::RCX, 7)
            .ins(Instruction::RepMovsb)
            .jmp(return_label)
            // ^^^
            // direct print to stdout
            .label(direct_print_stdout)
            .mov(Register::RAX, 1) // sys_write
            .ins(Instruction::Syscall)
            .label(return_label)
            .ins(Instruction::Ret)
    }

    fn write_flush_stdout(&mut self) -> &mut Self {
        self.label(Label::FlushStdout)
            .ins(Instruction::Comment(Cow::Borrowed("void flush_stdout()")))
            .mov(Register::RAX, 1) // sys_write
            .mov(Register::RDI, 1) // stdout
            .lea(Register::RSI, Label::StdoutBuffer)
            .mov(Register::RDX, Label::StdoutLen)
            .ins(Instruction::Syscall)
            .mov(Label::StdoutLen, 0)
            .ins(Instruction::Ret)
    }

    fn write_print_decimal(&mut self) -> &mut Self {
        // rdi: fd
        // rsi: num

        self.label(Label::PrintDecimal)
            .ins(Instruction::Comment(Cow::Borrowed(
                "void print_decimal(int fd, int64_t num)",
            )))
            .ins(Instruction::Ret)
    }
}
