extern crate core;

pub mod source;

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Config {
    pub tab_width: usize,
    pub balance_comments: bool,
    pub string_escape_sequences: bool,
}
