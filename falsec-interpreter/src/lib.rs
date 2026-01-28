mod error;

use crate::error::InterpreterError;
use falsec_types::source::{Command, Lambda, LambdaCommand, Pos, Program, Span};
use falsec_types::{Config, TypeSafety};
use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
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
    pub fn run(mut self) -> Result<(), InterpreterError> {
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
                    (Var(c), _) => Ok((c as u8 - b'a') as i64),
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
                    (Integer(i), _) => Ok(((i as u8 & 31) + b'a') as char),
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
                    (Lambda(id), _) => Ok(((id as u8 & 31) + b'a') as char),
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
            let (instruction, pos) = state.get_current_instruction()?;
            state.current_pos = pos.start;
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
                Command::Add => {
                    let a = state.pop()?.into_integer(&self.config, &state)?;
                    let b = state.pop()?.into_integer(&self.config, &state)?;
                    state.pushi(a + b);
                }
                Command::Sub => {
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
                    let lambda_id = state.pop()?.into_lambda(&self.config, &state)?;
                    let condition = state.pop()?.into_integer(&self.config, &state)?;
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
                Command::ReadChar => {
                    let mut buf = [0];
                    match self.input.read_exact(&mut buf) {
                        Ok(()) => {
                            state.pushi(buf[0] as i64);
                        }
                        Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                            state.pushi(-1);
                        }
                        Err(e) => {
                            return Err(InterpreterError::io_error(
                                e,
                                state.current_pos,
                                state.current_lambda_id,
                                state.program_counter,
                            ));
                        }
                    };
                }
                Command::WriteChar => {
                    let c = state.pop()?.into_integer(&self.config, &state)?;
                    self.output.write_all(&[c as u8]).map_err(|e| {
                        InterpreterError::io_error(
                            e,
                            state.current_pos,
                            state.current_lambda_id,
                            state.program_counter,
                        )
                    })?;
                }
                Command::StringLiteral(s) => {
                    self.output.write_all(s.as_bytes()).map_err(|e| {
                        InterpreterError::io_error(
                            e,
                            state.current_pos,
                            state.current_lambda_id,
                            state.program_counter,
                        )
                    })?;
                }
                Command::WriteInt => {
                    let i = state.pop()?.into_integer(&self.config, &state)?;
                    write!(self.output, "{}", i).map_err(|e| {
                        InterpreterError::io_error(
                            e,
                            state.current_pos,
                            state.current_lambda_id,
                            state.program_counter,
                        )
                    })?;
                }
                Command::Flush => {
                    self.output.flush().map_err(|e| {
                        InterpreterError::io_error(
                            e,
                            state.current_pos,
                            state.current_lambda_id,
                            state.program_counter,
                        )
                    })?;
                }
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
    use falsec_types::source::LambdaCommand::LambdaReference;
    use falsec_types::source::{Command, Pos, Program, Span};
    use falsec_types::{Config, TypeSafety};
    use std::borrow::Cow;
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

    /// ```
    /// use falsec_interpreter::simple_lambda;
    ///
    /// simple_lambda![];
    /// ```
    #[macro_export]
    macro_rules! simple_lambda {
        ($($command:expr),* $(,)?) => {
            vec![$(($command, Span::new(Pos::at_start(), Pos::at_start(), ""))),*]
        };
    }

    /// ```
    /// use falsec_interpreter::simple_program;
    ///
    /// simple_program![];
    /// ```
    #[macro_export]
    macro_rules! simple_program {
        ($($command:expr),* $(,)?) => {
            Program {
                main_id: 0,
                lambdas: HashMap::from([(
                    0,
                    simple_lambda!($($command),*),
                )]),
                ..Default::default()
            }
        };
    }

    #[test]
    fn empty_program() {
        let program = Program {
            main_id: 0,
            lambdas: HashMap::from([(0, Vec::new())]),
            ..Default::default()
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
            ..Default::default()
        };
        let stack = Rc::<RefCell<_>>::default();
        run_simple(program, stack.clone());
        assert_eq!(stack.borrow().deref(), &[StackValue::Integer(123)]);
    }

    #[test]
    fn simple_program() {
        let program = simple_program![
            Command::IntLiteral(123),
            Command::IntLiteral(321),
            Command::Add,
        ];
        let stack = Rc::<RefCell<_>>::default();
        run_simple(program, stack.clone());
        assert_eq!(stack.borrow().deref(), &[StackValue::Integer(444)]);
    }

    #[test]
    fn store_load() {
        let program = simple_program![
            Command::IntLiteral(123),
            Command::Var('a'),
            Command::Store,
            Command::Var('a'),
            Command::Load,
        ];
        let stack = Rc::<RefCell<_>>::default();
        run_simple(program, stack.clone());
        assert_eq!(stack.borrow().deref(), &[StackValue::Integer(123)]);
    }

    #[test]
    fn basic_lambda() {
        let program = Program {
            main_id: 0,
            lambdas: HashMap::from([
                (
                    0,
                    simple_lambda![
                        Command::IntLiteral(123),
                        Command::Lambda(LambdaReference(1)),
                        Command::Exec,
                    ],
                ),
                (1, simple_lambda![Command::IntLiteral(321), Command::Add]),
            ]),
            ..Default::default()
        };
        let stack = Rc::<RefCell<_>>::default();
        run_simple(program, stack.clone());
        assert_eq!(stack.borrow().deref(), &[StackValue::Integer(444)]);
    }

    #[test]
    fn conditional() {
        let program = Program {
            main_id: 0,
            lambdas: HashMap::from([
                (
                    0,
                    simple_lambda![Command::Lambda(LambdaReference(1)), Command::Conditional],
                ),
                (1, simple_lambda![Command::IntLiteral(123)]),
            ]),
            ..Default::default()
        };
        let stack_false = Rc::new(RefCell::new(vec![StackValue::Integer(0)]));
        run_simple(program.clone(), stack_false.clone());
        assert_eq!(stack_false.borrow().deref(), &[]);
        let stack_true = Rc::new(RefCell::new(vec![StackValue::Integer(-1)]));
        run_simple(program, stack_true.clone());
        assert_eq!(stack_true.borrow().deref(), &[StackValue::Integer(123)]);
    }

    #[test]
    fn factorial() {
        // iterate from stack[0] to 1, multiplying the result into `a`. `a` is initialized to 1.
        let program = Program {
            main_id: 0,
            lambdas: HashMap::from([
                // 1a:[$0>][$a;*a:1-]#%a;.
                (
                    0,
                    simple_lambda![
                        Command::IntLiteral(1),
                        Command::Var('a'),
                        Command::Store,
                        Command::Lambda(LambdaReference(1)),
                        Command::Lambda(LambdaReference(2)),
                        Command::While,
                        Command::Drop,
                        Command::Var('a'),
                        Command::Load,
                    ],
                ),
                // $0>
                (
                    1,
                    simple_lambda![Command::Dup, Command::IntLiteral(0), Command::Gt,],
                ),
                // $a;*a:1-
                (
                    2,
                    simple_lambda![
                        Command::Dup,
                        Command::Var('a'),
                        Command::Load,
                        Command::Mul,
                        Command::Var('a'),
                        Command::Store,
                        Command::IntLiteral(1),
                        Command::Sub
                    ],
                ),
            ]),
            ..Default::default()
        };
        let stack0 = Rc::new(RefCell::new(vec![StackValue::Integer(0)]));
        let stack1 = Rc::new(RefCell::new(vec![StackValue::Integer(1)]));
        let stack2 = Rc::new(RefCell::new(vec![StackValue::Integer(2)]));
        let stack3 = Rc::new(RefCell::new(vec![StackValue::Integer(3)]));
        let stack4 = Rc::new(RefCell::new(vec![StackValue::Integer(4)]));
        let stack5 = Rc::new(RefCell::new(vec![StackValue::Integer(5)]));
        run_simple(program.clone(), stack0.clone());
        run_simple(program.clone(), stack1.clone());
        run_simple(program.clone(), stack2.clone());
        run_simple(program.clone(), stack3.clone());
        run_simple(program.clone(), stack4.clone());
        run_simple(program, stack5.clone());

        assert_eq!(stack0.borrow().deref(), &[StackValue::Integer(1)]);
        assert_eq!(stack1.borrow().deref(), &[StackValue::Integer(1)]);
        assert_eq!(stack2.borrow().deref(), &[StackValue::Integer(2)]);
        assert_eq!(stack3.borrow().deref(), &[StackValue::Integer(6)]);
        assert_eq!(stack4.borrow().deref(), &[StackValue::Integer(24)]);
        assert_eq!(stack5.borrow().deref(), &[StackValue::Integer(120)]);
    }

    #[test]
    fn output_char() {
        let program = simple_program![
            Command::IntLiteral('H' as u64),
            Command::WriteChar,
            Command::IntLiteral('i' as u64),
            Command::WriteChar,
            Command::Flush,
        ];
        let stack = Rc::<RefCell<_>>::default();
        let mut output = Vec::new();
        let interpreter = Interpreter {
            input: &[] as &[u8],
            output: &mut output,
            program,
            config: Default::default(),
            stack,
        };
        interpreter.run().unwrap();
        assert_eq!(output, b"Hi");
    }

    #[test]
    fn output_string() {
        let program = simple_program![
            Command::StringLiteral(Cow::Borrowed("Hello, World!")),
            Command::Flush,
        ];
        let stack = Rc::<RefCell<_>>::default();
        let mut output = Vec::new();
        let interpreter = Interpreter {
            input: &[] as &[u8],
            output: &mut output,
            program,
            config: Default::default(),
            stack,
        };
        interpreter.run().unwrap();
        assert_eq!(output, b"Hello, World!");
    }

    #[test]
    fn output_number() {
        let program = simple_program![
            Command::IntLiteral(123),
            Command::WriteInt,
            Command::IntLiteral(456),
            Command::WriteInt,
            Command::Flush,
        ];
        let stack = Rc::<RefCell<_>>::default();
        let mut output = Vec::new();
        let interpreter = Interpreter {
            input: &[] as &[u8],
            output: &mut output,
            program,
            config: Default::default(),
            stack,
        };
        interpreter.run().unwrap();
        assert_eq!(output, b"123456");
    }

    #[test]
    fn input_char() {
        let program = simple_program![
            Command::ReadChar,
            Command::WriteChar,
            Command::ReadChar,
            Command::WriteChar,
            Command::Flush,
        ];
        let stack = Rc::<RefCell<_>>::default();
        let input = b"Hi";
        let mut output = Vec::new();
        let interpreter = Interpreter {
            input: input.as_ref(),
            output: &mut output,
            program,
            config: Default::default(),
            stack,
        };
        interpreter.run().unwrap();
        assert_eq!(output, b"Hi");
    }

    /// ```false
    /// { read until you see \n, and convert decimal to number: }
    /// [ß0[^$$10=\13=|~][$$'01->\'9>~&['0-\10*+$]?%]#%ß]n:
    /// "A: "n;!$$a:."
    /// B: "n;!$$b:.+"
    /// "a;." + "b;." = "."
    /// "
    /// ```
    #[test]
    fn add() {
        let program = Program {
            main_id: 0,
            lambdas: HashMap::from([
                (
                    0,
                    simple_lambda![
                        Command::Comment(Cow::Borrowed(
                            " read until you see \\n, and convert decimal to number: "
                        )),
                        Command::Lambda(LambdaReference(1)),
                        Command::Var('n'),
                        Command::Store,
                        Command::StringLiteral(Cow::Borrowed("A: ")),
                        Command::Var('n'),
                        Command::Load,
                        Command::Exec,
                        Command::Dup,
                        Command::Dup,
                        Command::Var('a'),
                        Command::Store,
                        Command::WriteInt,
                        Command::StringLiteral(Cow::Borrowed("\nB: ")),
                        Command::Var('n'),
                        Command::Load,
                        Command::Exec,
                        Command::Dup,
                        Command::Dup,
                        Command::Var('b'),
                        Command::Store,
                        Command::WriteInt,
                        Command::Add,
                        Command::StringLiteral(Cow::Borrowed("\n")),
                        Command::Var('a'),
                        Command::Load,
                        Command::WriteInt,
                        Command::StringLiteral(Cow::Borrowed(" + ")),
                        Command::Var('b'),
                        Command::Load,
                        Command::WriteInt,
                        Command::StringLiteral(Cow::Borrowed(" = ")),
                        Command::WriteInt,
                        Command::StringLiteral(Cow::Borrowed("\n")),
                    ],
                ),
                // ß0[^$$10=\13=|~][$$'01->\'9>~&['0-\10*+$]?%]#%ß
                (
                    1,
                    simple_lambda![
                        Command::Flush,
                        Command::IntLiteral(0),
                        Command::Lambda(LambdaReference(2)),
                        Command::Lambda(LambdaReference(3)),
                        Command::While,
                        Command::Drop,
                        Command::Flush,
                    ],
                ),
                // ^$$10=\13=|~
                (
                    2,
                    simple_lambda![
                        Command::ReadChar,
                        Command::Dup,
                        Command::Dup,
                        Command::IntLiteral(10), // \n
                        Command::Eq,
                        Command::Swap,
                        Command::IntLiteral(13), // \r
                        Command::Eq,
                        Command::BitOr,
                        Command::BitNot,
                    ],
                ),
                // $$'01->\'9>~&['0-\10*+$]?%
                (
                    3,
                    simple_lambda![
                        Command::Dup,
                        Command::Dup,
                        Command::CharLiteral('0'),
                        Command::IntLiteral(1),
                        Command::Sub,
                        Command::Gt,
                        Command::Swap,
                        Command::CharLiteral('9'),
                        Command::Gt,
                        Command::BitNot,
                        Command::BitAnd,
                        Command::Lambda(LambdaReference(4)),
                        Command::Conditional,
                        Command::Drop,
                    ],
                ),
                // '0-\10*+$
                (
                    4,
                    simple_lambda![
                        Command::CharLiteral('0'),
                        Command::Sub,
                        Command::Swap,
                        Command::IntLiteral(10),
                        Command::Mul,
                        Command::Add,
                        Command::Dup,
                    ],
                ),
            ]),
            ..Default::default()
        };
        let stack = Rc::<RefCell<_>>::default();
        let input = b"123\n321\n";
        let mut output = Vec::new();
        let interpreter = Interpreter {
            input: input.as_ref(),
            output: &mut output,
            program,
            config: Default::default(),
            stack,
        };
        interpreter.run().unwrap();
        assert_eq!(output, b"A: 123\nB: 321\n123 + 321 = 444\n");
    }

    #[test]
    fn number_as_var() {
        let program = simple_program![
            Command::IntLiteral(123),
            Command::Var('c'),
            Command::Store,
            Command::IntLiteral(2),
            Command::Load,
        ];
        let stack = Rc::<RefCell<_>>::default();
        Interpreter::<&[_], &mut [_]> {
            input: &[],
            output: &mut [],
            program,
            config: Config {
                type_safety: TypeSafety::None,
                ..Default::default()
            },
            stack: stack.clone(),
        }
        .run()
        .unwrap();
        assert_eq!(stack.borrow().deref(), &[StackValue::Integer(123)]);
    }

    #[test]
    fn test_empty_input() {
        let program = simple_program![
            Command::ReadChar,
            Command::WriteInt,
            Command::ReadChar,
            Command::WriteInt,
            Command::ReadChar,
            Command::WriteInt,
        ];
        let stack = Rc::<RefCell<_>>::default();
        let mut output = Vec::<u8>::new();
        Interpreter {
            input: &b"x"[..],
            output: &mut output,
            program,
            config: Config {
                type_safety: TypeSafety::None,
                ..Default::default()
            },
            stack,
        }
        .run()
        .unwrap();
        assert_eq!(output, format!("{}-1-1", b'x').as_bytes());
    }
}
