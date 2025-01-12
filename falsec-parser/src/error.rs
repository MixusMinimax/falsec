use falsec_types::source::Pos;
use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use std::num::ParseIntError;

#[derive(Clone, Debug)]
pub struct ParseError {
    pub pos: Pos,
    pub kind: ParseErrorKind,
}

impl Error for ParseError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseErrorKind {
    UnexpectedToken(char),
    MissingToken(char),
    ParseIntError(ParseIntError),
    EndOfFile,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Parse Error at {}: {}", self.pos, self.kind)
    }
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use ParseErrorKind::*;
        match self {
            UnexpectedToken(token) => write!(f, "Unexpected token: {}", token),
            MissingToken(token) => write!(f, "Missing token: {}", token),
            ParseIntError(err) => write!(f, "Invalid integer: {}", err),
            EndOfFile => write!(f, "End of file reached"),
        }
    }
}

impl ParseError {
    pub fn unexpected_token(pos: Pos, token: char) -> Self {
        Self {
            pos,
            kind: ParseErrorKind::UnexpectedToken(token),
        }
    }

    pub fn missing_token(pos: Pos, token: char) -> Self {
        Self {
            pos,
            kind: ParseErrorKind::MissingToken(token),
        }
    }

    pub fn parse_int_error(pos: Pos, err: ParseIntError) -> Self {
        Self {
            pos,
            kind: ParseErrorKind::ParseIntError(err),
        }
    }

    pub fn end_of_file(pos: Pos) -> Self {
        Self {
            pos,
            kind: ParseErrorKind::EndOfFile,
        }
    }
}
