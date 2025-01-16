mod error;

use crate::error::InterpreterError;
use falsec_types::source::{Command, Lambda, LambdaCommand, Pos, Program, Span};
use falsec_types::{Config, TypeSafety};
use std::collections::HashMap;
use std::io::{Read, Write};
#[cfg(test)]
use std::{cell::RefCell, rc::Rc};

pub struct Interpreter<'source, Input: Read, Output: Write> {
    input: Input,
    output: Output,
    program: Program<'source>,
    config: Config,
    #[cfg(test)]
    stack: Rc<RefCell<Vec<StackValue>>>,
}

impl<'source, Input: Read, Output: Write> Interpreter<'source, Input, Output> {
    pub fn new(input: Input, output: Output, program: Program<'source>, config: Config) -> Self {
        Self {
            input,
            output,
            program,
            config,
            #[cfg(test)]
            stack: Default::default(),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
enum LoopState {
    #[default]
    None,
    ExecutingCondition(u64, u64),
    ExecutingBody(u64, u64),
}

struct StackFrame {
    lambda_id: u64,
    program_counter: usize,
    loop_state: LoopState,
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
            variables: HashMap<char, StackValue>,
            current_lambda_id: u64,
            program_counter: usize,
            current_pos: Pos,
            current_lambda: &'source Lambda<'source>,
            loop_state: LoopState,
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
            variables: HashMap::new(),
            current_lambda_id: self.program.main_id,
            program_counter: 0,
            current_pos: Pos::at_start(),
            current_lambda: get_lambda(&self.program, self.program.main_id, Pos::at_start())?,
            loop_state: LoopState::None,
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
                    loop_state: self.loop_state,
                });
                self.current_lambda_id = id;
                self.program_counter = 0;
                self.current_pos = Pos::at_start();
                self.current_lambda = get_lambda(self.program, id, self.current_pos)?;
                self.loop_state = LoopState::None;
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
                self.loop_state = frame.loop_state;
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

        trait TypeValidation {
            fn into_integer(self, config: &Config, state: &State) -> Result<i64, InterpreterError>;
            fn into_var(self, config: &Config, state: &State) -> Result<char, InterpreterError>;
            fn into_lambda(self, config: &Config, state: &State) -> Result<u64, InterpreterError>;
        }

        impl TypeValidation for StackValue {
            fn into_integer(self, config: &Config, state: &State) -> Result<i64, InterpreterError> {
                use StackValue::*;
                match (self, config.type_safety) {
                    (Integer(i), _) => Ok(i),
                    (Var(_), TypeSafety::Full) => Err(InterpreterError::type_cast_error(
                        "Integer",
                        "Var",
                        state.current_pos,
                        state.current_lambda_id,
                        state.program_counter,
                    )),
                    (Var(c), _) => Ok(c as i64),
                    (Lambda(_), TypeSafety::Full) => Err(InterpreterError::type_cast_error(
                        "Integer",
                        "Lambda",
                        state.current_pos,
                        state.current_lambda_id,
                        state.program_counter,
                    )),
                    (Lambda(id), _) => Ok(id as i64),
                }
            }

            fn into_var(self, config: &Config, state: &State) -> Result<char, InterpreterError> {
                use StackValue::*;
                match (self, config.type_safety) {
                    (Integer(_), TypeSafety::Full | TypeSafety::LambdaAndVar) => {
                        Err(InterpreterError::type_cast_error(
                            "Integer",
                            "Var",
                            state.current_pos,
                            state.current_lambda_id,
                            state.program_counter,
                        ))
                    }
                    (Integer(i), _) => Ok(i as u8 as char),
                    (Var(c), _) => Ok(c),
                    (Lambda(_), TypeSafety::Full | TypeSafety::LambdaAndVar) => {
                        Err(InterpreterError::type_cast_error(
                            "Lambda",
                            "Var",
                            state.current_pos,
                            state.current_lambda_id,
                            state.program_counter,
                        ))
                    }
                    (Lambda(id), _) => Ok(id as u8 as char),
                }
            }

            fn into_lambda(self, config: &Config, state: &State) -> Result<u64, InterpreterError> {
                use StackValue::*;
                match (self, config.type_safety) {
                    (
                        Integer(_),
                        TypeSafety::Full | TypeSafety::LambdaAndVar | TypeSafety::Lambda,
                    ) => Err(InterpreterError::type_cast_error(
                        "Integer",
                        "Lambda",
                        state.current_pos,
                        state.current_lambda_id,
                        state.program_counter,
                    )),
                    (Integer(i), _) => Ok(i as u64),
                    (Var(_), TypeSafety::Full | TypeSafety::LambdaAndVar | TypeSafety::Lambda) => {
                        Err(InterpreterError::type_cast_error(
                            "Var",
                            "Lambda",
                            state.current_pos,
                            state.current_lambda_id,
                            state.program_counter,
                        ))
                    }
                    (Var(c), _) => Ok(c as u64),
                    (Lambda(id), _) => Ok(id),
                }
            }
        }

        #[cfg(test)]
        {
            let stack = self.stack.borrow();
            for i in stack.iter() {
                state.push(*i);
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
                    let index = state.pop()?.into_integer(&self.config, &state)?;
                    if index < 0 || index as usize >= state.data_stack.len() {
                        return Err(InterpreterError::index_out_of_bounds(
                            index,
                            state.data_stack.len(),
                            state.current_pos,
                            state.current_lambda_id,
                            state.program_counter,
                        ));
                    }
                    state.push(state.data_stack[state.data_stack.len() - index as usize - 1]);
                }
                Command::Plus => {
                    let a = state.pop()?.into_integer(&self.config, &state)?;
                    let b = state.pop()?.into_integer(&self.config, &state)?;
                    state.pushi(a + b);
                }
                Command::Minus => {
                    let a = state.pop()?.into_integer(&self.config, &state)?;
                    let b = state.pop()?.into_integer(&self.config, &state)?;
                    state.pushi(b - a);
                }
                Command::Mul => {
                    let a = state.pop()?.into_integer(&self.config, &state)?;
                    let b = state.pop()?.into_integer(&self.config, &state)?;
                    state.pushi(a * b);
                }
                Command::Div => {
                    let a = state.pop()?.into_integer(&self.config, &state)?;
                    let b = state.pop()?.into_integer(&self.config, &state)?;
                    state.pushi(b / a);
                }
                Command::Neg => {
                    let a = state.pop()?.into_integer(&self.config, &state)?;
                    state.pushi(-a);
                }
                Command::BitAnd => {
                    let a = state.pop()?.into_integer(&self.config, &state)?;
                    let b = state.pop()?.into_integer(&self.config, &state)?;
                    state.pushi(a & b);
                }
                Command::BitOr => {
                    let a = state.pop()?.into_integer(&self.config, &state)?;
                    let b = state.pop()?.into_integer(&self.config, &state)?;
                    state.pushi(a | b);
                }
                Command::BitNot => {
                    let a = state.pop()?.into_integer(&self.config, &state)?;
                    state.pushi(!a);
                }
                Command::Gt => {
                    let a = state.pop()?.into_integer(&self.config, &state)?;
                    let b = state.pop()?.into_integer(&self.config, &state)?;
                    state.pushi(if b > a { -1 } else { 0 });
                }
                Command::Eq => {
                    let a = state.pop()?.into_integer(&self.config, &state)?;
                    let b = state.pop()?.into_integer(&self.config, &state)?;
                    state.pushi(if b == a { -1 } else { 0 });
                }
                Command::Lambda(LambdaCommand::LambdaReference(id)) => {
                    state.push(StackValue::Lambda(*id));
                }
                Command::Lambda(LambdaCommand::LambdaDefinition(..)) => {
                    return Err(InterpreterError::lambda_definition_not_allowed(
                        state.current_pos,
                    ));
                }
                Command::Exec => {
                    let id = state.pop()?.into_lambda(&self.config, &state)?;
                    state.call_lambda(id)?;
                }
                Command::Conditional => {
                    let condition = state.pop()?.into_integer(&self.config, &state)?;
                    let lambda_id = state.pop()?.into_lambda(&self.config, &state)?;
                    if condition != 0 {
                        state.call_lambda(lambda_id)?;
                    }
                }
                Command::While => match state.loop_state {
                    LoopState::None => {
                        let body = state.pop()?.into_lambda(&self.config, &state)?;
                        let condition = state.pop()?.into_lambda(&self.config, &state)?;
                        state.loop_state = LoopState::ExecutingCondition(condition, body);
                        state.program_counter -= 1;
                        state.call_lambda(condition)?;
                    }
                    LoopState::ExecutingBody(condition, body) => {
                        state.loop_state = LoopState::ExecutingCondition(condition, body);
                        state.program_counter -= 1;
                        state.call_lambda(condition)?;
                    }
                    LoopState::ExecutingCondition(condition, body) => {
                        let result = state.pop()?.into_integer(&self.config, &state)?;
                        if result == 0 {
                            state.loop_state = LoopState::None;
                        } else {
                            state.loop_state = LoopState::ExecutingBody(condition, body);
                            state.program_counter -= 1;
                            state.call_lambda(body)?;
                        }
                    }
                },
                Command::Var(c) => state.push(StackValue::Var(*c)),
                Command::Store => {
                    let var = state.pop()?.into_var(&self.config, &state)?;
                    let value = state.pop()?;
                    state.variables.insert(var, value);
                }
                Command::Load => {
                    let var = state.pop()?.into_var(&self.config, &state)?;
                    let value = state
                        .variables
                        .get(&var)
                        .copied()
                        .unwrap_or(StackValue::Integer(0));
                    state.push(value);
                }
                Command::ReadChar => {}
                Command::WriteChar => {}
                Command::StringLiteral(_) => {}
                Command::WriteInt => {}
                Command::Flush => {}
                Command::Comment(_) => {}
            }
        }

        #[cfg(test)]
        {
            let mut stack = self.stack.borrow_mut();
            stack.clear();
            stack.extend(state.data_stack.iter().copied());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Interpreter, StackValue};
    use falsec_types::source::{Command, Pos, Program, Span};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::ops::Deref;
    use std::rc::Rc;

    fn run_simple(program: Program, stack: Rc<RefCell<Vec<StackValue>>>) {
        let interpreter = Interpreter::<&[_], &mut [_]> {
            input: &[],
            output: &mut [],
            program,
            config: Default::default(),
            stack,
        };
        interpreter.run().unwrap();
    }

    #[test]
    fn empty_program() {
        let program = Program {
            main_id: 0,
            lambdas: HashMap::from([(0, Vec::new())]),
        };
        let stack = Rc::<RefCell<_>>::default();
        run_simple(program, stack.clone());
        assert_eq!(stack.borrow().len(), 0);
    }

    #[test]
    fn push_integer() {
        let program = Program {
            main_id: 0,
            lambdas: HashMap::from([(
                0,
                vec![(
                    Command::IntLiteral(123),
                    Span {
                        start: Pos::at_start(),
                        end: Pos::new(3, 1, 4),
                        source: "123",
                    },
                )],
            )]),
        };
        let stack = Rc::<RefCell<_>>::default();
        run_simple(program, stack.clone());
        assert_eq!(stack.borrow().deref(), &[StackValue::Integer(123)]);
    }
}
