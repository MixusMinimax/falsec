use crate::Config;
use std::borrow::Cow;
use std::fmt;

/// A position in a source file.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Pos {
    /// Offset in bytes
    pub offset: usize,
    /// Line number, starting from 1. If unknown, set to 0.
    pub line: usize,
    /// Column number, starting from 1. If unknown, set to 0.
    pub column: usize,
}

impl Pos {
    pub fn new(offset: usize, line: usize, column: usize) -> Self {
        Self {
            offset,
            line,
            column,
        }
    }

    pub fn advance(&mut self, c: char, config: &Config) {
        self.offset += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else if c == '\t' {
            self.column =
                (self.column - 1 + config.tab_width) / config.tab_width * config.tab_width + 1;
        } else {
            self.column += 1;
        }
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.line, self.column, self.offset)
    }
}

impl Pos {
    /// Create a new position at the start of the file.
    pub fn at_start() -> Self {
        Self {
            offset: 0,
            line: 1,
            column: 1,
        }
    }
}

impl Default for Pos {
    fn default() -> Self {
        Self {
            offset: 0,
            line: 1,
            column: 1,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Span<'source> {
    /// Inclusive
    pub start: Pos,
    /// Exclusive
    pub end: Pos,
    /// Raw source code
    pub source: &'source str,
}

impl<'source> Span<'source> {
    pub fn new(start: Pos, end: Pos, source: &'source str) -> Self {
        Self { start, end, source }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Command<'source> {
    /// **123** put integer 123 on the stack
    IntLiteral(u64),
    /// **'c** put character code of 'c' on the stack
    CharLiteral(char),

    /// **$** duplicate the top of the stack
    Dup,
    /// **%** drop the top of the stack
    Drop,
    /// **\\** swap the top two elements of the stack
    Swap,
    /// **@** rotate the top three elements of the stack
    Rot,
    /// **ø** copy the nth element of the stack to the top, where n is the top of the stack
    Pick,

    /// **+**
    Plus,
    /// **-**
    Minus,
    /// *****
    Mul,
    /// **/**
    Div,
    /// **_** negate (negative numbers are entered "123_")
    Neg,
    /// **&** bitwise and
    BitAnd,
    /// **|** bitwise or
    BitOr,
    /// **~** bitwise not
    BitNot,

    /// **>** greater than. false is 0, true is -1
    Gt,
    /// **=** equal. false is 0, true is -1
    Eq,

    /// **\[...]** define and put a lambda on the stack
    Lambda(Vec<(Command<'source>, Span<'source>)>),
    /// **!** execute a lambda
    Exec,
    /// **?** conditional execution: condition\[true]?
    Conditional,
    /// **\#** while loop: \[condition]\[body]#
    While,

    /// **a-z** put a reference to one of the 26 variables on the stack
    Var(char),
    /// **:** store into a variable
    Store,
    /// **;** fetch from a variable
    Load,

    /// **^** read a character
    ReadChar,
    /// **,** write a character
    WriteChar,
    /// **"..."** write a string (may contain embedded newlines).
    /// Contains the unescaped string, if escape sequences are present and enabled.
    StringLiteral(Cow<'source, str>),
    /// **.** write an integer
    WriteInt,
    /// **ß** flush buffered input/output
    Flush,

    /// **{...}** comment.
    Comment(Cow<'source, str>),
}

#[cfg(test)]
mod tests {
    use crate::source::Pos;
    use crate::Config;

    #[test]
    fn pos_advance() {
        let config = Config { tab_width: 4 };
        let mut pos = Pos::new(0, 1, 1);
        pos.advance('\t', &config);
        assert_eq!(pos, Pos::new(1, 1, 5));
        let mut pos = Pos::new(0, 1, 2);
        pos.advance('\t', &config);
        assert_eq!(pos, Pos::new(1, 1, 5));
        let mut pos = Pos::new(0, 1, 3);
        pos.advance('\t', &config);
        assert_eq!(pos, Pos::new(1, 1, 5));
        let mut pos = Pos::new(0, 1, 4);
        pos.advance('\t', &config);
        assert_eq!(pos, Pos::new(1, 1, 5));
        let mut pos = Pos::new(0, 1, 5);
        pos.advance('\t', &config);
        assert_eq!(pos, Pos::new(1, 1, 9));
        let mut pos = Pos::new(0, 1, 6);
        pos.advance('\t', &config);
        assert_eq!(pos, Pos::new(1, 1, 9));
    }
}
