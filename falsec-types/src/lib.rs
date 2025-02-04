extern crate core;

pub use tab_width::TabWidth;

pub mod source;

#[derive(Clone, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Config {
    #[cfg_attr(feature = "serde", serde(default))]
    pub tab_width: TabWidth,

    /// if true, comments { ... } can contain balanced brackets, like {{}}.
    /// if not, a comment {{}} ends at the first closing bracket.
    #[cfg_attr(feature = "serde", serde(default))]
    pub balance_comments: bool,

    /// if true, string literals that contain escape sequences are unescaped.
    /// if false, escape sequences are kept as is.
    #[cfg_attr(feature = "serde", serde(default))]
    pub string_escape_sequences: bool,

    #[cfg_attr(feature = "serde", serde(default))]
    pub type_safety: TypeSafety,

    /// if true, the assembly file will contain the original source code as comments.
    #[cfg_attr(feature = "serde", serde(default))]
    pub write_command_comments: bool,

    /// stdout buffer size in bytes
    #[cfg_attr(feature = "serde", serde(default))]
    pub stdout_buffer_size: StdoutBufferSize,

    /// stack size in bytes
    #[cfg_attr(feature = "serde", serde(default))]
    pub stack_size: StackSize,
}

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum TypeSafety {
    #[default]
    /// No type safety checks are performed.
    None,
    /// When trying to execute a lambda, make sure that the popped value is a lambda.
    Lambda,
    /// Include all checks from [TypeSafety::Lambda], and make sure that when storing or loading
    /// a variable, the popped value is a variable name.
    LambdaAndVar,
    /// Include all checks from [TypeSafety::LambdaAndVar], and ensure that only integers can be
    /// used for arithmetic operations.
    Full,
}

mod tab_width {
    use std::ops::{Add, Div, Mul};

    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
    pub struct TabWidth(pub usize);

    impl Default for TabWidth {
        fn default() -> Self {
            Self(2)
        }
    }

    impl From<usize> for TabWidth {
        fn from(value: usize) -> Self {
            Self(value)
        }
    }

    impl From<TabWidth> for usize {
        fn from(value: TabWidth) -> Self {
            value.0
        }
    }

    impl Add<TabWidth> for usize {
        type Output = usize;

        fn add(self, rhs: TabWidth) -> Self::Output {
            self + rhs.0
        }
    }

    impl Add<usize> for TabWidth {
        type Output = usize;

        fn add(self, rhs: usize) -> Self::Output {
            self.0 + rhs
        }
    }

    impl Div<usize> for TabWidth {
        type Output = usize;

        fn div(self, rhs: usize) -> Self::Output {
            self.0 / rhs
        }
    }

    impl Div<TabWidth> for usize {
        type Output = usize;

        fn div(self, rhs: TabWidth) -> Self::Output {
            self / rhs.0
        }
    }

    impl Mul<usize> for TabWidth {
        type Output = usize;

        fn mul(self, rhs: usize) -> Self::Output {
            self.0 * rhs
        }
    }

    impl Mul<TabWidth> for usize {
        type Output = usize;

        fn mul(self, rhs: TabWidth) -> Self::Output {
            self * rhs.0
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct StdoutBufferSize(pub u64);

impl Default for StdoutBufferSize {
    fn default() -> Self {
        Self(8192)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct StackSize(pub u64);

impl Default for StackSize {
    fn default() -> Self {
        Self(65_536)
    }
}
