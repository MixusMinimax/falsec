use falsec_types::source::Lambda;
use falsec_types::Config;
use std::io::{Read, Write};

pub struct InterpreterError {}

pub struct Interpreter<'source, Input: Read, Output: Write> {
    input: Input,
    output: Output,
    program: Lambda<'source>,
    config: Config,
}

impl<'source, Input: Read, Output: Write> Interpreter<'source, Input, Output> {
    pub fn new(input: Input, output: Output, program: Lambda<'source>, config: Config) -> Self {
        Self {
            input,
            output,
            program,
            config,
        }
    }
}

enum StackValue {
    Integer(i64),
    Var(char),
    Lambda(u64),
}

impl<'source, Input: Read, Output: Write> Interpreter<'source, Input, Output> {
    pub fn run(self) -> Result<(), InterpreterError> {
        // let mut stack = Vec::new();
        // let mut lambdas = HashMap::new();
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
