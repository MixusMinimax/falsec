use falsec_types::source::Pos;
use std::error::Error;
use std::fmt;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct InterpreterError {
    pub pos: Pos,
    pub current_lambda_id: u64,
    pub program_counter: usize,
    pub kind: InterpreterErrorKind,
}

#[derive(Clone, Debug)]
pub enum InterpreterErrorKind {
    InvalidLambdaReference(u64),
    InvalidProgramCounter(usize),
    LambdaDefinitionNotAllowed,
    TriedToPopFromEmptyCallStack,
    TriedToPopFromEmptyDataStack,
    TypeCastError {
        from: &'static str,
        to: &'static str,
    },
    IndexOutOfBounds(i64, usize),
    IO(Rc<std::io::Error>),
}

impl Error for InterpreterError {}

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Interpreter Error: {}", self.kind)
    }
}

impl fmt::Display for InterpreterErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use InterpreterErrorKind::*;
        match self {
            InvalidLambdaReference(id) => write!(f, "Invalid lambda reference: {}", id),
            InvalidProgramCounter(pc) => write!(f, "Invalid program counter: {}", pc),
            LambdaDefinitionNotAllowed => write!(f, "Lambda definition not allowed"),
            TriedToPopFromEmptyCallStack => write!(f, "Tried to pop from empty call stack"),
            TriedToPopFromEmptyDataStack => write!(f, "Tried to pop from empty data stack"),
            TypeCastError { from, to } => write!(f, "Type cast error: {} -> {}", from, to),
            IndexOutOfBounds(index, len) => {
                write!(f, "Index out of bounds: {} must be in 0..{}", index, len)
            }
            IO(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl InterpreterError {
    pub fn invalid_lambda_reference(pos: Pos, current_lambda_id: u64, id: u64) -> Self {
        Self {
            pos,
            current_lambda_id,
            program_counter: 0,
            kind: InterpreterErrorKind::InvalidLambdaReference(id),
        }
    }

    pub fn invalid_program_counter(pos: Pos, current_lambda_id: u64, pc: usize) -> Self {
        Self {
            pos,
            current_lambda_id,
            program_counter: pc,
            kind: InterpreterErrorKind::InvalidProgramCounter(pc),
        }
    }

    pub fn lambda_definition_not_allowed(pos: Pos) -> Self {
        Self {
            pos,
            current_lambda_id: 0,
            program_counter: 0,
            kind: InterpreterErrorKind::LambdaDefinitionNotAllowed,
        }
    }

    pub fn tried_to_pop_from_empty_call_stack(
        pos: Pos,
        current_lambda_id: u64,
        program_counter: usize,
    ) -> Self {
        Self {
            pos,
            current_lambda_id,
            program_counter,
            kind: InterpreterErrorKind::TriedToPopFromEmptyCallStack,
        }
    }

    pub fn tried_to_pop_from_empty_data_stack(
        pos: Pos,
        current_lambda_id: u64,
        program_counter: usize,
    ) -> Self {
        Self {
            pos,
            current_lambda_id,
            program_counter,
            kind: InterpreterErrorKind::TriedToPopFromEmptyDataStack,
        }
    }

    pub fn type_cast_error(
        from: &'static str,
        to: &'static str,
        pos: Pos,
        current_lambda_id: u64,
        program_counter: usize,
    ) -> Self {
        Self {
            pos,
            current_lambda_id,
            program_counter,
            kind: InterpreterErrorKind::TypeCastError { from, to },
        }
    }

    pub fn index_out_of_bounds(
        index: i64,
        len: usize,
        pos: Pos,
        current_lambda_id: u64,
        program_counter: usize,
    ) -> Self {
        Self {
            pos,
            current_lambda_id,
            program_counter,
            kind: InterpreterErrorKind::IndexOutOfBounds(index, len),
        }
    }

    pub fn io_error(
        err: std::io::Error,
        pos: Pos,
        current_lambda_id: u64,
        program_counter: usize,
    ) -> Self {
        Self {
            pos,
            current_lambda_id,
            program_counter,
            kind: InterpreterErrorKind::IO(Rc::new(err)),
        }
    }
}
