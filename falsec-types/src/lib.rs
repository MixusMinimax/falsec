extern crate core;

pub mod source;

#[derive(Clone, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Config {
    pub tab_width: usize,
    pub balance_comments: bool,
    pub string_escape_sequences: bool,
}
