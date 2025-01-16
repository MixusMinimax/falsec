mod error;

use crate::error::InterpreterError;
use falsec_types::source::{Command, Lambda, LambdaCommand, Pos, Program, Span};
use falsec_types::Config;
use std::io::{Read, Write};

pub struct Interpreter<'source, Input: Read, Output: Write> {
    input: Input,
    output: Output,
    program: Program<'source>,
    config: Config,
}

impl<'source, Input: Read, Output: Write> Interpreter<'source, Input, Output> {
    pub fn new(input: Input, output: Output, program: Program<'source>, config: Config) -> Self {
        Self {
            input,
            output,
            program,
            config,
        }
    }
}

struct StackFrame {
    lambda_id: u64,
    program_counter: usize,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum StackValue {
    Integer(i64),
    Var(char),
    Lambda(u64),
}

impl<Input: Read, Output: Write> Interpreter<'_, Input, Output> {
    pub fn run(self) -> Result<(), InterpreterError> {
        struct State<'source> {
            program: &'source Program<'source>,
            call_stack: Vec<StackFrame>,
            data_stack: Vec<StackValue>,
            current_lambda_id: u64,
            program_counter: usize,
            current_pos: Pos,
            current_lambda: &'source Lambda<'source>,
        }

        fn get_lambda<'source>(
            program: &'source Program<'source>,
            current_lambda_id: u64,
            current_pos: Pos,
        ) -> Result<&'source Lambda<'source>, InterpreterError> {
            program.lambdas.get(&current_lambda_id).ok_or_else(|| {
                InterpreterError::invalid_lambda_reference(current_pos, current_lambda_id, 0)
            })
        }

        let mut state = State {
            program: &self.program,
            call_stack: Vec::new(),
            data_stack: Vec::new(),
            current_lambda_id: self.program.main_id,
            program_counter: 0,
            current_pos: Pos::at_start(),
            current_lambda: get_lambda(&self.program, self.program.main_id, Pos::at_start())?,
        };

        impl<'source> State<'source> {
            fn get_current_instruction(
                &self,
            ) -> Result<&'source (Command<'source>, Span<'source>), InterpreterError> {
                self.current_lambda
                    .get(self.program_counter)
                    .ok_or_else(|| {
                        InterpreterError::invalid_program_counter(
                            self.current_pos,
                            self.current_lambda_id,
                            self.program_counter,
                        )
                    })
            }

            fn call_lambda(&mut self, id: u64) -> Result<(), InterpreterError> {
                self.call_stack.push(StackFrame {
                    lambda_id: self.current_lambda_id,
                    program_counter: self.program_counter,
                });
                self.current_lambda_id = id;
                self.program_counter = 0;
                self.current_pos = Pos::at_start();
                self.current_lambda = get_lambda(self.program, id, self.current_pos)?;
                Ok(())
            }

            fn ret_lambda(&mut self) -> Result<(), InterpreterError> {
                let frame = self.call_stack.pop().ok_or_else(|| {
                    InterpreterError::tried_to_pop_from_empty_call_stack(
                        self.current_pos,
                        self.current_lambda_id,
                        self.program_counter,
                    )
                })?;
                self.current_lambda_id = frame.lambda_id;
                self.program_counter = frame.program_counter;
                self.current_pos = Pos::at_start();
                self.current_lambda = get_lambda(self.program, frame.lambda_id, self.current_pos)?;
                Ok(())
            }

            fn push(&mut self, value: StackValue) {
                self.data_stack.push(value);
            }

            fn peek(&self) -> Result<StackValue, InterpreterError> {
                self.data_stack.last().copied().ok_or_else(|| {
                    InterpreterError::tried_to_pop_from_empty_data_stack(
                        self.current_pos,
                        self.current_lambda_id,
                        self.program_counter,
                    )
                })
            }

            fn pop(&mut self) -> Result<StackValue, InterpreterError> {
                self.data_stack.pop().ok_or_else(|| {
                    InterpreterError::tried_to_pop_from_empty_data_stack(
                        self.current_pos,
                        self.current_lambda_id,
                        self.program_counter,
                    )
                })
            }

            fn pushi(&mut self, i: impl Into<i64>) {
                self.data_stack.push(StackValue::Integer(i.into()));
            }
        }

        loop {
            if state.program_counter >= state.current_lambda.len() {
                if state.call_stack.is_empty() {
                    break;
                }
                state.ret_lambda()?;
                continue;
            }
            let (instruction, _) = state.get_current_instruction()?;
            state.program_counter += 1;
            match instruction {
                Command::IntLiteral(i) => state.pushi(*i as i64),
                Command::CharLiteral(c) => state.pushi(*c as i64),
                Command::Dup => state.push(state.peek()?),
                Command::Drop => _ = state.pop()?,
                Command::Swap => {
                    let a = state.pop()?;
                    let b = state.pop()?;
                    state.push(a);
                    state.push(b);
                }
                Command::Rot => {
                    let a = state.pop()?;
                    let b = state.pop()?;
                    let c = state.pop()?;
                    state.push(b);
                    state.push(a);
                    state.push(c);
                }
                Command::Pick => {
                    let index = state.pop()?;
                }
                Command::Plus => {}
                Command::Minus => {}
                Command::Mul => {}
                Command::Div => {}
                Command::Neg => {}
                Command::BitAnd => {}
                Command::BitOr => {}
                Command::BitNot => {}
                Command::Gt => {}
                Command::Eq => {}
                Command::Lambda(LambdaCommand::LambdaReference(id)) => {
                    state.push(StackValue::Lambda(*id));
                }
                Command::Lambda(LambdaCommand::LambdaDefinition(..)) => {
                    return Err(InterpreterError::lambda_definition_not_allowed(
                        state.current_pos,
                    ));
                }
                Command::Exec => {
                    if let StackValue::Lambda(id) = state.pop()? {
                        state.call_lambda(id)?;
                    } else {
                        return Err(InterpreterError::invalid_lambda_reference(
                            state.current_pos,
                            state.current_lambda_id,
                            0,
                        ));
                    }
                }
                Command::Conditional => {}
                Command::While => {}
                Command::Var(_) => {}
                Command::Store => {}
                Command::Load => {}
                Command::ReadChar => {}
                Command::WriteChar => {}
                Command::StringLiteral(_) => {}
                Command::WriteInt => {}
                Command::Flush => {}
                Command::Comment(_) => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
