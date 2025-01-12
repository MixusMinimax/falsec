use std::borrow::Cow;

pub struct Span<'source> {
    /// Offset in bytes
    pub start: usize,
    /// Offset in bytes, past the end of the span
    pub end: usize,
    /// Raw source code
    pub source: &'source str,
}

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
