mod error;

use crate::error::{ParseError, ParseErrorKind};
use falsec_types::Config;
use falsec_types::source::{Command, Pos, Span};
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

impl<'source> Iterator for PosChars<'source> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos.advance(*self.chars.peek()?, &self.config);
        self.chars.next()
    }
}

pub struct Parser<'source> {
    source: &'source str,
    chars: PosChars<'source>,
}

impl<'source> Parser<'source> {
    pub fn new(source: &'source str, config: Config) -> Self {
        Self {
            source,
            chars: PosChars::new(source, config),
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
            c if c.is_ascii_digit() => {
                while self
                    .chars
                    .peek()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or_default()
                {
                    _ = self.chars.next();
                }
                Command::IntLiteral(
                    self.source[pos.offset..self.pos().offset]
                        .parse()
                        .map_err(|err| ParseError::parse_int_error(pos, err))?,
                )
            }
            'a' => Command::BitAnd,
            c => return Err(ParseError::unexpected_token(pos, c)),
        };
        Ok((command, Span {
            start: pos,
            end: self.pos(),
            source: &self.source[pos.offset..self.pos().offset],
        }))
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
    use falsec_types::Config;
    use falsec_types::source::{Command, Pos, Span};

    fn test_config() -> Config {
        Config { tab_width: 4 }
    }

    #[test]
    fn single_int_literal() {
        let code = "123";
        let commands: Result<Vec<_>, _> = Parser::new(code, test_config()).collect();
        assert!(commands.is_ok());
        assert_eq!(commands.unwrap(), [(Command::IntLiteral(123), Span {
            start: Pos {
                offset: 0,
                line: 1,
                column: 1
            },
            end: Pos {
                offset: 3,
                line: 1,
                column: 4
            },
            source: "123"
        })]);
    }

    #[test]
    fn multiple_int_literals() {
        let code = " 123 2  5  ";
        let commands: Result<Vec<_>, _> = Parser::new(code, test_config()).collect();
        assert!(commands.is_ok());
        assert_eq!(commands.unwrap(), [
            (Command::IntLiteral(123), Span {
                start: Pos {
                    offset: 1,
                    line: 1,
                    column: 2
                },
                end: Pos {
                    offset: 4,
                    line: 1,
                    column: 5
                },
                source: "123"
            }),
            (Command::IntLiteral(2), Span {
                start: Pos {
                    offset: 5,
                    line: 1,
                    column: 6
                },
                end: Pos {
                    offset: 6,
                    line: 1,
                    column: 7
                },
                source: "2"
            }),
            (Command::IntLiteral(5), Span {
                start: Pos {
                    offset: 8,
                    line: 1,
                    column: 9
                },
                end: Pos {
                    offset: 9,
                    line: 1,
                    column: 10
                },
                source: "5"
            }),
        ]);
    }

    #[test]
    fn int_literal_linefeed() {
        let code = "\t123\n3\n\t 6\n";
        let commands: Result<Vec<_>, _> = Parser::new(code, test_config()).collect();
        assert!(commands.is_ok());
        assert_eq!(commands.unwrap(), [
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
        ])
    }
}
