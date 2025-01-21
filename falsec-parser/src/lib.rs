mod error;

use crate::error::{ParseError, ParseErrorKind};
use falsec_types::source::{Command, LambdaCommand, Pos, Span};
use falsec_types::Config;
use std::borrow::Cow;
use std::iter::Peekable;
use std::str::Chars;

struct PosChars<'source> {
    pub pos: Pos,
    chars: Peekable<Chars<'source>>,
    config: Config,
}

impl<'source> PosChars<'source> {
    pub fn new(source: &'source str, config: Config) -> Self {
        Self {
            pos: Pos::at_start(),
            chars: source.chars().peekable(),
            config,
        }
    }

    pub fn peek(&mut self) -> Option<&<Chars<'source> as Iterator>::Item> {
        self.chars.peek()
    }
}

impl Iterator for PosChars<'_> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos.advance(*self.chars.peek()?, &self.config);
        self.chars.next()
    }
}

pub struct Parser<'source> {
    source: &'source str,
    chars: PosChars<'source>,
    config: Config,
}

impl<'source> Parser<'source> {
    pub fn new(source: &'source str, config: Config) -> Self {
        Self {
            source,
            chars: PosChars::new(source, config.clone()),
            config,
        }
    }
}

impl<'source> Parser<'source> {
    pub fn pos(&self) -> Pos {
        self.chars.pos
    }

    pub fn parse_command(&mut self) -> Result<(Command<'source>, Span<'source>), ParseError> {
        while self
            .chars
            .peek()
            .map(|c| c.is_ascii_whitespace())
            .unwrap_or_default()
        {
            self.chars.next();
        }
        let pos = self.pos();
        let command = match self
            .chars
            .next()
            .ok_or_else(|| ParseError::end_of_file(pos))?
        {
            '\'' => {
                let pos2 = self.pos();
                Command::CharLiteral(
                    self.chars
                        .next()
                        .ok_or_else(|| ParseError::missing_token(pos2, 'c'))?,
                )
            }
            '$' => Command::Dup,
            '%' => Command::Drop,
            '\\' => Command::Swap,
            '@' => Command::Rot,
            'ø' => Command::Pick,
            '+' => Command::Add,
            '-' => Command::Sub,
            '*' => Command::Mul,
            '/' => Command::Div,
            '_' => Command::Neg,
            '&' => Command::BitAnd,
            '|' => Command::BitOr,
            '~' => Command::BitNot,
            '>' => Command::Gt,
            '=' => Command::Eq,
            '[' => {
                let mut lambda = Vec::new();
                loop {
                    match self.chars.peek() {
                        None => return Err(ParseError::missing_token(self.pos(), ']')),
                        Some(']') => {
                            self.chars.next();
                            break;
                        }
                        Some(_) => {
                            lambda.push(self.parse_command()?);
                        }
                    }
                }
                Command::Lambda(LambdaCommand::LambdaDefinition(lambda))
            }
            '!' => Command::Exec,
            '?' => Command::Conditional,
            '#' => Command::While,
            ':' => Command::Store,
            ';' => Command::Load,
            '^' => Command::ReadChar,
            ',' => Command::WriteChar,
            '"' => {
                let start = self.pos();
                let mut unescaped = String::new();
                loop {
                    let p = self.pos();
                    match self
                        .chars
                        .next()
                        .ok_or_else(|| ParseError::missing_token(p, '"'))?
                    {
                        '"' => {
                            break Command::StringLiteral(if !unescaped.is_empty() {
                                Cow::Owned(unescaped)
                            } else {
                                Cow::Borrowed(&self.source[start.offset..p.offset])
                            });
                        }
                        '\\' => {
                            if unescaped.is_empty() {
                                unescaped.push_str(&self.source[start.offset..p.offset]);
                            };
                            let p2 = self.pos();
                            unescaped.push(
                                match self
                                    .chars
                                    .next()
                                    .ok_or_else(|| ParseError::missing_token(p, 'c'))?
                                {
                                    lit @ ('"' | '\\') => lit,
                                    'n' => '\n',
                                    'r' => '\r',
                                    't' => '\t',
                                    '0' => '\0',
                                    '\n' => continue, // ignore newline
                                    c => return Err(ParseError::unexpected_token(p2, c)),
                                },
                            );
                        }
                        c => {
                            if !unescaped.is_empty() {
                                unescaped.push(c)
                            }
                        }
                    };
                }
            }
            '.' => Command::WriteInt,
            'ß' => Command::Flush,
            '{' => {
                let mut level = 1;
                let start = self.pos();
                loop {
                    let p = self.pos();
                    match self
                        .chars
                        .next()
                        .ok_or_else(|| ParseError::missing_token(p, '}'))?
                    {
                        '{' if self.config.balance_comments => level += 1,
                        '}' => {
                            level -= 1;
                            if level == 0 {
                                break Command::Comment(Cow::Borrowed(
                                    &self.source[start.offset..p.offset],
                                ));
                            }
                        }
                        _ => (),
                    };
                }
            }
            c if c.is_ascii_lowercase() => Command::Var(c),
            c if c.is_ascii_digit() => {
                while self
                    .chars
                    .peek()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or_default()
                {
                    self.chars.next();
                }
                Command::IntLiteral(
                    self.source[pos.offset..self.pos().offset]
                        .parse()
                        .map_err(|err| ParseError::parse_int_error(pos, err))?,
                )
            }
            c => return Err(ParseError::unexpected_token(pos, c)),
        };
        Ok((
            command,
            Span {
                start: pos,
                end: self.pos(),
                source: &self.source[pos.offset..self.pos().offset],
            },
        ))
    }
}

impl<'source> Iterator for Parser<'source> {
    type Item = Result<(Command<'source>, Span<'source>), ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parse_command() {
            Err(ParseError {
                kind: ParseErrorKind::EndOfFile,
                ..
            }) => None,
            result => Some(result),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Parser;
    use falsec_types::source::{Command, LambdaCommand, Pos, Span};
    use falsec_types::Config;
    use std::borrow::Cow;

    fn test_config() -> Config {
        Config {
            tab_width: 4.into(),
            ..Default::default()
        }
    }

    #[test]
    fn single_int_literal() {
        let code = "123";
        let commands: Result<Vec<_>, _> = Parser::new(code, test_config()).collect();
        assert!(commands.is_ok());
        assert_eq!(
            commands.unwrap(),
            [(
                Command::IntLiteral(123),
                Span::new(Pos::new(0, 1, 1), Pos::new(3, 1, 4), "123")
            )]
        );
    }

    #[test]
    fn multiple_int_literals() {
        let code = " 123 2  5  ";
        let commands: Result<Vec<_>, _> = Parser::new(code, test_config()).collect();
        assert!(commands.is_ok());
        assert_eq!(
            commands.unwrap(),
            [
                (
                    Command::IntLiteral(123),
                    Span::new(Pos::new(1, 1, 2), Pos::new(4, 1, 5), "123")
                ),
                (
                    Command::IntLiteral(2),
                    Span::new(Pos::new(5, 1, 6), Pos::new(6, 1, 7), "2")
                ),
                (
                    Command::IntLiteral(5),
                    Span::new(Pos::new(8, 1, 9), Pos::new(9, 1, 10), "5")
                ),
            ]
        );
    }

    #[test]
    fn int_literal_linefeed() {
        let code = "\t123\n3\n\t 6\n";
        let commands: Result<Vec<_>, _> = Parser::new(code, test_config()).collect();
        assert!(commands.is_ok());
        assert_eq!(
            commands.unwrap(),
            [
                (
                    Command::IntLiteral(123),
                    Span::new(Pos::new(1, 1, 5), Pos::new(4, 1, 8), "123")
                ),
                (
                    Command::IntLiteral(3),
                    Span::new(Pos::new(5, 2, 1), Pos::new(6, 2, 2), "3")
                ),
                (
                    Command::IntLiteral(6),
                    Span::new(Pos::new(9, 3, 6), Pos::new(10, 3, 7), "6")
                ),
            ]
        )
    }

    #[test]
    fn string_escape_sequences() {
        let code = r###"0_"asd\n\r\t asd \\\"asd"#"###;
        let commands: Result<Vec<_>, _> = Parser::new(
            code,
            Config {
                string_escape_sequences: true,
                ..Default::default()
            },
        )
        .collect();
        let commands: Vec<_> = commands.unwrap().into_iter().map(|(com, _)| com).collect();
        assert!(matches!(
            commands[..],
            [
                Command::IntLiteral(0),
                Command::Neg,
                Command::StringLiteral(Cow::Owned(ref str)),
                Command::While
            ] if str == "asd\n\r\t asd \\\"asd"
        ));
    }

    #[test]
    fn string_escaped_newline() {
        let code = "\"a\\nsd\\\nqwe\"";
        let commands: Result<Vec<_>, _> = Parser::new(
            code,
            Config {
                string_escape_sequences: true,
                ..Default::default()
            },
        )
        .collect();
        let commands: Vec<_> = commands.unwrap().into_iter().map(|(com, _)| com).collect();
        assert!(matches!(
            commands[..],
            [Command::StringLiteral(Cow::Owned(ref str))] if str == "a\nsdqwe"
        ));
    }

    #[test]
    fn complex() {
        let code = r###"
            { read until you see \n, and convert decimal to number: }
            [ß0[^$$10=\13=|~][$$'01->\'9>~&['0-\10*+$]?%]#%ß]n:
            "A: "n;!$$a:."
            B: "n;!$$b:.+"
            "a;." + "b;." = "."
            "
        "###;
        let commands: Result<Vec<_>, _> = Parser::new(
            code,
            Config {
                tab_width: 2.into(),
                balance_comments: false,
                string_escape_sequences: false,
                ..Default::default()
            },
        )
        .collect();
        assert!(commands.is_ok());
        let without_spans: Vec<_> = commands.unwrap().into_iter().map(|(com, _)| com).collect();
        // assert_matches is experimental and requires nightly
        assert!(matches!(
            without_spans[..],
            [
                Command::Comment(Cow::Borrowed(
                    " read until you see \\n, and convert decimal to number: ",
                )),
                Command::Lambda(..),
                Command::Var('n'),
                Command::Store,
                Command::StringLiteral(Cow::Borrowed("A: ")),
                Command::Var('n'),
                Command::Load,
                Command::Exec,
                Command::Dup,
                Command::Dup,
                Command::Var('a'),
                Command::Store,
                Command::WriteInt,
                ..
            ]
        ));
        let Command::Lambda(LambdaCommand::LambdaDefinition(ref l1)) = without_spans[1] else {
            panic!()
        };
        let l1: Vec<_> = l1.iter().map(|(com, _)| com).collect();
        assert!(matches!(
            l1[..],
            [
                Command::Flush,
                Command::IntLiteral(0),
                Command::Lambda(..),
                Command::Lambda(..),
                Command::While,
                Command::Drop,
                Command::Flush
            ]
        ));
    }
}
