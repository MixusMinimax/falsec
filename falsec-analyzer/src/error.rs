use std::error::Error;
use std::fmt;

#[derive(Clone, Debug)]
pub struct AnalyzerError {
    pub kind: AnalyzerErrorKind,
}

#[derive(Clone, Debug)]
pub enum AnalyzerErrorKind {
    InvalidInput(String),
}

impl Error for AnalyzerError {}

impl AnalyzerError {
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self {
            kind: AnalyzerErrorKind::InvalidInput(message.into()),
        }
    }
}

impl fmt::Display for AnalyzerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Implement this
        write!(f, "Analyzer Error: {:?}", self.kind)
    }
}
