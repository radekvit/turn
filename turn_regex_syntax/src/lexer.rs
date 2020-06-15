use crate::hir::SetMember;
use std::fmt;
use turn_utils::text_reader::Position;
use turn_utils::text_reader::TextReader;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token<'a> {
    Sequence(&'a str),
    AnyChar,
    Repetition { min: u16, max: Option<u16> },
    Set(Vec<SetMember<'a>>),
    NegatedSet(Vec<SetMember<'a>>),
    Alternation,
    LParenthesis,
    RParenthesis,
    Subexpression(&'a str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    UnclosedSet {
        position: Position,
    },
    UnclosedSubexpression {
        position: Position,
    },
    UnclosedRepetition {
        position: Position,
    },
    InvalidRepetitionRange {
        min: u16,
        max: u16,
    },
    InvalidRepetitionCharacter {
        position: Position,
        character: char,
    },
    InvalidEscape {
        position: Position,
        character: Option<char>,
    },
    InvalidSetEscape {
        position: Position,
        character: char,
    },
    RangeIntegerOverflow {
        position: Position,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::UnclosedSet { position } => write!(
                f,
                "Set starting at {}:{} is missing closing character ']'.",
                position.row, position.col
            ),
            Error::UnclosedSubexpression { position } => write!(
                f,
                "Subexpression or category starting at {}:{} is missing closing character '>'.",
                position.row, position.col
            ),
            Error::InvalidRepetitionRange { min, max } => write!(
                f,
                "Invalid repetition range: {} is greater than {}.",
                min, max
            ),
            Error::InvalidRepetitionCharacter {
                position,
                character,
            } => write!(
                f,
                "Invalid character '{}' inside repetition range at position {}:{}",
                character, position.row, position.col
            ),
            Error::InvalidSetEscape {
                position,
                character,
            } => write!(
                f,
                "Invalid escaped character '{}' inside set at position {}:{}",
                character, position.row, position.col
            ),
            Error::InvalidEscape {
                position,
                character,
            } => {
                if let Some(c) = character {
                    write!(
                        f,
                        "Invalid escaped character '{}' at position {}:{}",
                        c, position.row, position.col
                    )
                } else {
                    write!(
                        f,
                        "Unexpected end of input after '\\' at position {}:{}",
                        position.row, position.col
                    )
                }
            }
            Error::UnclosedRepetition { position } => write!(
                f,
                "Unexpected end of input inside range specifier at position {}:{}",
                position.row, position.col
            ),
            Error::RangeIntegerOverflow { position } => write!(
                f,
                "Integer range over 65_536 at position {}:{}",
                position.row, position.col
            ),
        }
    }
}

impl std::error::Error for Error {}

pub struct Lexer<'a> {
    input: TextReader<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer {
        Lexer {
            input: TextReader::new(input),
        }
    }

    pub fn next_token(&mut self) -> Option<Result<Token<'a>, Error>> {
        let position = self.input.current_position();
        match self.input.next() {
            Some('_') => Some(Ok(Token::AnyChar)),
            Some('*') => Some(Ok(Token::Repetition { min: 0, max: None })),
            Some('+') => Some(Ok(Token::Repetition { min: 1, max: None })),
            Some('?') => Some(Ok(Token::Repetition {
                min: 0,
                max: Some(1),
            })),
            Some('(') => Some(Ok(Token::LParenthesis)),
            Some(')') => Some(Ok(Token::RParenthesis)),
            Some('|') => Some(Ok(Token::Alternation)),
            Some('{') => Some(self.repetition(position)),
            Some('[') => Some(self.set(position)),
            Some('<') => Some(self.subexpression(position)),
            Some('\\') => Some(self.escaped()),
            Some(_) => Some(Ok(self.sequence(position))),
            None => None,
        }
    }

    fn repetition(&mut self, position: Position) -> Result<Token<'a>, Error> {
        let min = self.integer(position)?;
        match self.input.peek() {
            Some('-') => {
                self.input.next();
                let max = self.integer(self.input.current_position())?;
                match self.input.peek() {
                    Some('}') => {
                        self.input.next();
                        let min = min.unwrap_or(0);
                        if let Some(max) = max {
                            if min > max {
                                return Err(Error::InvalidRepetitionRange { min, max });
                            }
                        }
                        Ok(Token::Repetition { min, max })
                    }
                    Some(c) => Err(Error::InvalidRepetitionCharacter {
                        position: self.input.current_position(),
                        character: c,
                    }),
                    None => Err(Error::UnclosedRepetition { position }),
                }
            }
            Some('}') => {
                self.input.next();
                let min = min.unwrap_or(0);
                Ok(Token::Repetition {
                    min,
                    max: Some(min),
                })
            }
            Some(c) => Err(Error::InvalidRepetitionCharacter {
                position: self.input.current_position(),
                character: c,
            }),
            None => Err(Error::UnclosedRepetition { position }),
        }
    }

    fn set(&mut self, starting_position: Position) -> Result<Token<'a>, Error> {
        let mut members = Vec::new();
        // check if first character is '!'
        let negated = if self.input.peek() == Some('!') {
            self.input.next();
            true
        } else {
            false
        };
        // read set characters or categories until ']'
        loop {
            match self.input.peek() {
                // end set
                Some(']') => {
                    self.input.next();
                    break;
                }
                // process subexpression (assuming category)
                Some('<') => {
                    let start = self.input.current_position();
                    self.input.next();
                    let category = self.subexpression(start)?;
                    if let Token::Subexpression(category) = category {
                        members.push(SetMember::Category(category));
                    } else {
                        unreachable!();
                    }
                }
                // escaped characters within sets
                Some('\\') => {
                    self.input.next();
                    match self.input.next() {
                        Some(c) if c == '\\' || c == '<' || c == ']' => {
                            members.push(SetMember::Character(c))
                        }
                        Some(c) => {
                            return Err(Error::InvalidSetEscape {
                                position: self.input.current_position(),
                                character: c,
                            })
                        }
                        None => {
                            return Err(Error::UnclosedSet {
                                position: starting_position,
                            })
                        }
                    }
                }
                Some(x) => {
                    self.input.next();
                    members.push(SetMember::Character(x));
                }
                None => {
                    return Err(Error::UnclosedSet {
                        position: starting_position,
                    });
                }
            }
        }
        if negated {
            Ok(Token::NegatedSet(members))
        } else {
            Ok(Token::Set(members))
        }
    }

    fn subexpression(&mut self, position: Position) -> Result<Token<'a>, Error> {
        let start = self.input.current_position();
        let mut end = self.input.current_position();
        loop {
            match self.input.next() {
                Some('>') => return Ok(Token::Subexpression(self.input.input_slice(start..end))),
                Some(_) => end = self.input.current_position(),
                None => return Err(Error::UnclosedSubexpression { position }),
            }
        }
    }

    fn sequence(&mut self, start: Position) -> Token<'a> {
        loop {
            match self.input.peek() {
                Some('_') | Some('*') | Some('+') | Some('?') | Some('(') | Some(')')
                | Some('|') | Some('{') | Some('[') | Some('<') | Some('\\') => break,
                Some(_) => {
                    self.input.next();
                }
                None => break,
            }
        }
        Token::Sequence(self.input.input_slice_from(start))
    }

    fn escaped(&mut self) -> Result<Token<'a>, Error> {
        let start = self.input.current_position();
        match self.input.next() {
            Some('_') | Some('*') | Some('+') | Some('?') | Some('{') | Some('}') | Some('\\')
            | Some('[') | Some(']') | Some('|') | Some('(') | Some(')') | Some('<') | Some('>') => {
                Ok(Token::Sequence(self.input.input_slice_from(start)))
            }
            Some(c) => Err(Error::InvalidEscape {
                position: self.input.current_position(),
                character: Some(c),
            }),
            None => Err(Error::InvalidEscape {
                position: self.input.current_position(),
                character: None,
            }),
        }
    }

    fn integer(&mut self, position: Position) -> Result<Option<u16>, Error> {
        let mut number = match self.input.peek() {
            Some(c) if c.is_ascii_digit() => c.to_digit(10).unwrap() as u16,
            _ => return Ok(None),
        };
        self.input.next();
        loop {
            match self.input.peek() {
                Some(c) if c.is_ascii_digit() => {
                    number = number
                        .checked_mul(10)
                        .and_then(|x| x.checked_add(c.to_digit(10).unwrap() as u16))
                        .ok_or(Error::RangeIntegerOverflow { position })?;
                    self.input.next();
                }
                _ => return Ok(Some(number)),
            }
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty() {
        let mut lexer = Lexer::new("");
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_sequence() {
        let mut lexer = Lexer::new("💣b東x#e#ß");
        assert_eq!(lexer.next(), Some(Ok(Token::Sequence("💣b東x#e#ß"))));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_anychar() {
        let mut lexer = Lexer::new("_");
        assert_eq!(lexer.next(), Some(Ok(Token::AnyChar)));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_repetition_star() {
        let mut lexer = Lexer::new("*");
        assert_eq!(
            lexer.next(),
            Some(Ok(Token::Repetition { min: 0, max: None }))
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_repetition_plus() {
        let mut lexer = Lexer::new("+");
        assert_eq!(
            lexer.next(),
            Some(Ok(Token::Repetition { min: 1, max: None }))
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_repetition_question_mark() {
        let mut lexer = Lexer::new("?");
        assert_eq!(
            lexer.next(),
            Some(Ok(Token::Repetition {
                min: 0,
                max: Some(1)
            }))
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_repetition_variants() {
        let mut lexer = Lexer::new("{99}{-1}{2-}{2-7}{7-2}{0-z2}");
        assert_eq!(
            lexer.next(),
            Some(Ok(Token::Repetition {
                min: 99,
                max: Some(99),
            }))
        );
        assert_eq!(
            lexer.next(),
            Some(Ok(Token::Repetition {
                min: 0,
                max: Some(1),
            }))
        );
        assert_eq!(
            lexer.next(),
            Some(Ok(Token::Repetition { min: 2, max: None }))
        );
        assert_eq!(
            lexer.next(),
            Some(Ok(Token::Repetition {
                min: 2,
                max: Some(7),
            }))
        );
        assert_eq!(
            lexer.next(),
            Some(Err(Error::InvalidRepetitionRange { min: 7, max: 2 }))
        );
        assert_eq!(
            lexer.next(),
            Some(Err(Error::InvalidRepetitionCharacter {
                position: Position {
                    row: 1,
                    col: 26,
                    index: 25,
                },
                character: 'z'
            }))
        );
    }

    #[test]
    fn parse_set() {
        let mut lexer = Lexer::new("[se<t>\\<[\\]!\\\\]");
        assert_eq!(
            lexer.next(),
            Some(Ok(Token::Set(vec![
                SetMember::Character('s'),
                SetMember::Character('e'),
                SetMember::Category("t"),
                SetMember::Character('<'),
                SetMember::Character('['),
                SetMember::Character(']'),
                SetMember::Character('!'),
                SetMember::Character('\\'),
            ])))
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_negated_set() {
        let mut lexer = Lexer::new("[!se<t>\\<[\\]!\\\\]");
        assert_eq!(
            lexer.next(),
            Some(Ok(Token::NegatedSet(vec![
                SetMember::Character('s'),
                SetMember::Character('e'),
                SetMember::Category("t"),
                SetMember::Character('<'),
                SetMember::Character('['),
                SetMember::Character(']'),
                SetMember::Character('!'),
                SetMember::Character('\\'),
            ])))
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_alternation() {
        let mut lexer = Lexer::new("|");
        assert_eq!(lexer.next(), Some(Ok(Token::Alternation)));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_left_parenthesis() {
        let mut lexer = Lexer::new("(");
        assert_eq!(lexer.next(), Some(Ok(Token::LParenthesis)));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_right_parenthesis() {
        let mut lexer = Lexer::new(")");
        assert_eq!(lexer.next(), Some(Ok(Token::RParenthesis)));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_subexpression() {
        let mut lexer = Lexer::new("<subexpr>");
        assert_eq!(lexer.next(), Some(Ok(Token::Subexpression("subexpr"))));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_subexpression_unclosed() {
        let mut lexer = Lexer::new("<subexpr");
        assert_eq!(
            lexer.next(),
            Some(Err(Error::UnclosedSubexpression {
                position: Position {
                    row: 1,
                    col: 1,
                    index: 0,
                }
            }))
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_escaped_chars() {
        let lexer = Lexer::new("\\_\\*\\+\\?\\{\\}\\\\\\[\\]\\|\\(\\)\\<\\>");
        let tokens: Vec<_> = lexer.collect();
        assert_eq!(
            tokens,
            vec![
                Ok(Token::Sequence("_")),
                Ok(Token::Sequence("*")),
                Ok(Token::Sequence("+")),
                Ok(Token::Sequence("?")),
                Ok(Token::Sequence("{")),
                Ok(Token::Sequence("}")),
                Ok(Token::Sequence("\\")),
                Ok(Token::Sequence("[")),
                Ok(Token::Sequence("]")),
                Ok(Token::Sequence("|")),
                Ok(Token::Sequence("(")),
                Ok(Token::Sequence(")")),
                Ok(Token::Sequence("<")),
                Ok(Token::Sequence(">")),
            ]
        );
    }

    #[test]
    fn parse_multiple() {
        let lexer = Lexer::new("a💣bß>ř_*+?{42}{5-}{-5}{4-89}[ab<cat><ccat>][!x]|()<sub>");
        let tokens: Vec<_> = lexer.collect();
        assert_eq!(
            tokens,
            vec![
                Ok(Token::Sequence("a💣bß>ř")),
                Ok(Token::AnyChar),
                Ok(Token::Repetition { min: 0, max: None }),
                Ok(Token::Repetition { min: 1, max: None }),
                Ok(Token::Repetition {
                    min: 0,
                    max: Some(1)
                }),
                Ok(Token::Repetition {
                    min: 42,
                    max: Some(42)
                }),
                Ok(Token::Repetition { min: 5, max: None }),
                Ok(Token::Repetition {
                    min: 0,
                    max: Some(5)
                }),
                Ok(Token::Repetition {
                    min: 4,
                    max: Some(89)
                }),
                Ok(Token::Set(vec![
                    SetMember::Character('a'),
                    SetMember::Character('b'),
                    SetMember::Category("cat"),
                    SetMember::Category("ccat"),
                ])),
                Ok(Token::NegatedSet(vec![SetMember::Character('x'),])),
                Ok(Token::Alternation),
                Ok(Token::LParenthesis),
                Ok(Token::RParenthesis),
                Ok(Token::Subexpression("sub")),
            ]
        );
    }
}
