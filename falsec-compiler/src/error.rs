use falsec_types::source::Pos;
use std::error::Error;
use std::fmt::Formatter;
use std::rc::Rc;
use std::{fmt, io};

#[derive(Clone, Debug)]
pub struct CompilerError {
    pub source_location: Option<Pos>,
    pub kind: CompilerErrorKind,
}

#[derive(Clone, Debug)]
pub enum CompilerErrorKind {
    IO(Rc<std::io::Error>),
}

impl Error for CompilerError {}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Compiler error")?;
        if let Some(source_location) = self.source_location {
            write!(f, " at {}", source_location)?;
        }
        write!(f, ": {}", self.kind)
    }
}

impl fmt::Display for CompilerErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "...")
    }
}

impl From<io::Error> for CompilerError {
    fn from(err: io::Error) -> Self {
        Self {
            source_location: None,
            kind: CompilerErrorKind::IO(Rc::new(err)),
        }
    }
}
