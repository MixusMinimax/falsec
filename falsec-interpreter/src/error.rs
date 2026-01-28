use falsec_types::source::Pos;
use std::error::Error;
use std::fmt;
use std::rc::Rc;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct ProgramPos {
    pub pos: Pos,
    pub program_counter: usize,
    pub lambda_id: u64,
}

#[derive(Clone, Debug)]
pub struct InterpreterError {
    pub backtrace: Vec<ProgramPos>,
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
        let pos = self.backtrace[0].pos;
        write!(
            f,
            "Interpreter Error at {}:{}: {}",
            pos.line, pos.column, self.kind
        )
    }
}

impl InterpreterError {
    pub fn fmt_backtrace(&self, path: &str) -> impl fmt::Display {
        struct BT<'s, 'i>(&'s InterpreterError, &'i str);

        impl fmt::Display for BT<'_, '_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                macro_rules! write_pos {
                    ($p:expr) => {{
                        let p = $p;
                        write!(f, "{}:{}:{}", self.1, p.line, p.column)
                    }};
                }
                write!(f, "  raised at   ")?;
                write_pos!(self.0.backtrace[0].pos)?;
                for sf in self.0.backtrace.iter().skip(1) {
                    writeln!(f)?;
                    write!(f, "  called from ")?;
                    write_pos!(sf.pos)?;
                }
                Ok(())
            }
        }

        BT(self, path)
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
    pub fn invalid_lambda_reference(backtrace: Vec<ProgramPos>, id: u64) -> Self {
        Self {
            backtrace,
            kind: InterpreterErrorKind::InvalidLambdaReference(id),
        }
    }

    pub fn invalid_program_counter(backtrace: Vec<ProgramPos>, pc: usize) -> Self {
        Self {
            backtrace,
            kind: InterpreterErrorKind::InvalidProgramCounter(pc),
        }
    }

    pub fn lambda_definition_not_allowed(backtrace: Vec<ProgramPos>) -> Self {
        Self {
            backtrace,
            kind: InterpreterErrorKind::LambdaDefinitionNotAllowed,
        }
    }

    pub fn tried_to_pop_from_empty_call_stack(backtrace: Vec<ProgramPos>) -> Self {
        Self {
            backtrace,
            kind: InterpreterErrorKind::TriedToPopFromEmptyCallStack,
        }
    }

    pub fn tried_to_pop_from_empty_data_stack(backtrace: Vec<ProgramPos>) -> Self {
        Self {
            backtrace,
            kind: InterpreterErrorKind::TriedToPopFromEmptyDataStack,
        }
    }

    pub fn type_cast_error(
        backtrace: Vec<ProgramPos>,
        from: &'static str,
        to: &'static str,
    ) -> Self {
        Self {
            backtrace,
            kind: InterpreterErrorKind::TypeCastError { from, to },
        }
    }

    pub fn index_out_of_bounds(backtrace: Vec<ProgramPos>, index: i64, len: usize) -> Self {
        Self {
            backtrace,
            kind: InterpreterErrorKind::IndexOutOfBounds(index, len),
        }
    }

    pub fn io_error(backtrace: Vec<ProgramPos>, err: std::io::Error) -> Self {
        Self {
            backtrace,
            kind: InterpreterErrorKind::IO(Rc::new(err)),
        }
    }
}
