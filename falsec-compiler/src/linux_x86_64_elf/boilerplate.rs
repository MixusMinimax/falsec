use crate::linux_x86_64_elf::{
    label_expected_type, label_expected_type_len, Assembly, Instruction, Label, RegisterSize,
    SectionId, ValueType,
};
use falsec_types::Config;
use std::borrow::Cow;
use std::collections::HashMap;

pub fn write_bss<'source, 'asm>(
    asm: &'asm mut Assembly<'source>,
    config: &Config,
) -> &'asm mut Assembly<'source> {
    asm.add_instructions(
        SectionId::Bss,
        [
            Instruction::Label(Label::Variables),
            Instruction::Reserve(RegisterSize::R, 32),
            Instruction::Label(Label::VariableTypes),
            Instruction::Reserve(RegisterSize::L, 32),
            Instruction::Label(Label::Named("decimal_buffer")),
            Instruction::Reserve(RegisterSize::L, 32),
            Instruction::Label(Label::Named("stdout_buffer")),
            Instruction::Reserve(RegisterSize::L, config.stdout_buffer_size.0),
        ],
    )
}

pub fn write_error_messages<'source, 'asm>(
    asm: &'asm mut Assembly<'source>,
    strings: HashMap<u64, Cow<'source, str>>,
    config: &Config,
) -> &'asm mut Assembly<'source> {
    asm.add_instructions(
        SectionId::RoData,
        [Instruction::Comment(Cow::Borrowed("Error messages"))],
    );
    asm.add_instructions(
        SectionId::RoData,
        [
            Instruction::Label(Label::Named("mmap_error")),
            Instruction::DB(Cow::Borrowed(b"MMAP Failed! Exiting.\n")),
            Instruction::Label(Label::Named("mmap_error_len")),
            Instruction::Equ(Cow::Owned(format!("$ - {}", Label::Named("mmap_error")))),
        ],
    );
    for t in [ValueType::Number, ValueType::Variable, ValueType::Lambda] {
        asm.add_instructions(
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

    asm.add_instructions(
        SectionId::RoData,
        [Instruction::Comment(Cow::Borrowed("String literals"))],
    );
    for (id, string) in strings {
        asm.add_instructions(
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
    asm
}
