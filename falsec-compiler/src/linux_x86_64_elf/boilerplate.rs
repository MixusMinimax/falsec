use crate::linux_x86_64_elf::asm::{
    Address, Instruction, Label, Register, RegisterSize, SectionId,
};

use crate::linux_x86_64_elf::{label_expected_type, label_expected_type_len, Assembly, ValueType};
use falsec_types::{Config, TypeSafety};
use std::borrow::Cow;
use std::collections::HashMap;

pub(super) trait Boilerplate<'source> {
    fn write_bss(&mut self, config: &Config) -> &mut Self;

    fn write_error_messages(&mut self, strings: HashMap<u64, Cow<'source, str>>) -> &mut Self;

    fn write_setup(&mut self, config: &Config) -> &mut Self;

    fn write_print_string(&mut self, config: &Config) -> &mut Self;

    fn write_print_char(&mut self, config: &Config) -> &mut Self;

    fn write_flush_stdout(&mut self) -> &mut Self;

    fn write_print_decimal(&mut self, config: &Config) -> &mut Self;
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
                        format!("Expected to pop {} from stack!\n", t).into_bytes(),
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

    fn write_setup(&mut self, config: &Config) -> &mut Self {
        if config.stack_size.0 % 8 != 0 {
            panic!("Stack size must be a multiple of 8");
        }

        fn mmap<'a, 'source>(
            assm: &'a mut Assembly<'source>,
            size: u64,
        ) -> &'a mut Assembly<'source> {
            let success_label = assm.new_label();
            assm.mov(Register::RAX, 9)
                .xor(Register::RDI, Register::RDI)
                .mov(Register::RSI, size)
                .mov(Register::RDX, 0b0001 | 0b0010)
                .mov(Register::R10, 0b00000010 | 0b00100000)
                .mov(Register::R8, -1)
                .xor(Register::R9, Register::R9)
                .ins(Instruction::Syscall)
                .test(Register::RAX, Register::RAX)
                .jns(success_label)
                .com("mmap failed")
                .mov(Register::RDI, 2)
                .lea(Register::RSI, Address::b(Label::Named("mmap_error")))
                .mov(Register::RDX, Address::b(Label::Named("mmap_error_len")))
                .call(Label::PrintString)
                .mov(Register::RAX, 60)
                .mov(Register::RDI, 1)
                .ins(Instruction::Syscall)
                .label(success_label)
                .ins(Instruction::Nop)
        }

        self.com("===[SETUP START]===").com("Allocate FALSE stack:");
        mmap(self, config.stack_size.0);
        self.mov(Register::STACK_BASE, Register::RAX);

        if config.type_safety != TypeSafety::None {
            self.com("Allocate type stack:");
            mmap(self, config.stack_size.0 / 8);
            self.mov(Register::TYPE_STACK_BASE, Register::RAX);
        }

        self.com("===[SETUP END]===")
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
            .cpush(Register::RDX)
            .call(Label::FlushStdout)
            .cpop(Register::RDX)
            .mov(Register::RAX, 1) // sys_write
            .ins(Instruction::Syscall)
            .jmp(return_label)
            .label(handle_stdout)
            .mov(Register::RAX, config.stdout_buffer_size.0)
            .sub(Register::RAX, Register::RDX)
            .cmp(Register::RAX, Address::b(Label::StdoutLen)) // remaining space
            .jns(skip_flush)
            .call(Label::FlushStdout)
            .label(skip_flush)
            .test(Register::RAX, Register::RAX)
            .js(direct_print_stdout)
            // there is enough space in the stdout_buffer. copy the contents of rsi there.
            // vvv
            .mov(Register::RAX, Address::b(Label::StdoutLen))
            .add(Address::b(Label::StdoutLen), Register::RDX)
            .lea(
                Register::RDI,
                Address::bi(Label::StdoutBuffer, Register::RAX),
            )
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

    fn write_print_char(&mut self, config: &Config) -> &mut Self {
        // rdi: c

        let skip_flush = self.new_label();
        self.label(Label::PrintChar)
            .ins(Instruction::Comment(Cow::Borrowed(
                "void print_char(char c)",
            )))
            .mov(Register::RAX, config.stdout_buffer_size.0)
            .cmp(Register::RAX, Address::b(Label::StdoutLen)) // remaining space
            .jg(skip_flush)
            .call(Label::FlushStdout)
            .label(skip_flush)
            .lea(Register::RSI, Address::b(Label::StdoutBuffer))
            .mov(Register::RCX, Address::b(Label::StdoutLen))
            .mov(
                Address::bi(Register::RSI, Register::RCX).with_size(RegisterSize::L),
                Register::DIL,
            )
            .inc(Address::b(Label::StdoutLen).with_size(RegisterSize::R))
            .ins(Instruction::Ret)
    }

    fn write_flush_stdout(&mut self) -> &mut Self {
        let return_label = self.new_label();
        self.label(Label::FlushStdout)
            .ins(Instruction::Comment(Cow::Borrowed("void flush_stdout()")))
            .mov(Register::RAX, Address::b(Label::StdoutLen))
            .jz(return_label)
            .cpush(Register::RDI)
            .cpush(Register::RSI)
            .mov(Register::RAX, 1) // sys_write
            .mov(Register::RDI, 1) // stdout
            .lea(Register::RSI, Address::b(Label::StdoutBuffer))
            .mov(Register::RDX, Address::b(Label::StdoutLen))
            .ins(Instruction::Syscall)
            .mov(Address::b(Label::StdoutLen).with_size(RegisterSize::R), 0)
            .cpop(Register::RSI)
            .cpop(Register::RDI)
            .label(return_label)
            .ins(Instruction::Ret)
    }

    fn write_print_decimal(&mut self, config: &Config) -> &mut Self {
        // rdi: num
        // this function only writes to stdout.

        let skip_flush = self.new_label();
        let return_label = self.new_label();

        self.label(Label::PrintDecimal)
            .ins(Instruction::Comment(Cow::Borrowed(
                "void print_decimal(int64_t num)",
            )))
            // we write to the stdout_buffer directly, and we assume to need at most 20 bytes.
            .mov(Register::RAX, config.stdout_buffer_size.0)
            .sub(Register::RAX, Address::b(Label::StdoutLen))
            .cmp(Register::RAX, 20)
            .jns(skip_flush)
            .call(Label::FlushStdout)
            .label(skip_flush)
            // TODO
            .label(return_label)
            .ins(Instruction::Ret)
    }
}
