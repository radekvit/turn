use crate::hir::SetMember;
use std::fmt;
use turn_utils::text_reader::Position;
use turn_utils::text_reader::TextReader;

const END_OF_INPUT: Option<char> = None;
const ESCAPE: Option<char> = Some('\\');
const ANY_CHAR: Option<char> = Some('_');
const ALTERNATION: Option<char> = Some('|');
const REPETITION_STAR: Option<char> = Some('*');
const REPETITION_PLUS: Option<char> = Some('+');
const REPETITION_OPTIONAL: Option<char> = Some('?');
const REPETITION_START: Option<char> = Some('{');
const REPETITION_END: Option<char> = Some('}');
const REPETITION_DIVIDER: Option<char> = Some('-');

const LEFT_PARENTHESIS: Option<char> = Some('(');
const RIGHT_PARENTHESIS: Option<char> = Some(')');
const SUBEXPRESSION_START: Option<char> = Some('<');
const SUBEXPRESSION_END: Option<char> = Some('>');
const SET_START: Option<char> = Some('[');
const SET_END: Option<char> = Some(']');
const SET_NEGATOR: Option<char> = Some('!');

#[derive(Debug)]
pub struct Lexer<'a> {
    input: TextReader<'a>,
}

#[derive(Debug)]
pub struct CategoryLexer<'a> {
    input: TextReader<'a>,
}

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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CategoryToken<'a> {
    Sequence(&'a str),
    Category(&'a str),
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

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer {
        Lexer {
            input: TextReader::new(input),
        }
    }

    pub fn next_token(&mut self) -> Option<Result<Token<'a>, Error>> {
        let position = self.input.current_position();
        match self.input.next() {
            ANY_CHAR => Some(Ok(Token::AnyChar)),
            REPETITION_STAR => Some(Ok(Token::Repetition { min: 0, max: None })),
            REPETITION_PLUS => Some(Ok(Token::Repetition { min: 1, max: None })),
            REPETITION_OPTIONAL => Some(Ok(Token::Repetition {
                min: 0,
                max: Some(1),
            })),
            LEFT_PARENTHESIS => Some(Ok(Token::LParenthesis)),
            RIGHT_PARENTHESIS => Some(Ok(Token::RParenthesis)),
            ALTERNATION => Some(Ok(Token::Alternation)),
            REPETITION_START => Some(self.repetition(position)),
            SET_START => Some(self.set(position)),
            SUBEXPRESSION_START => Some(self.subexpression(position)),
            ESCAPE => Some(self.escaped()),
            Some(_) => Some(Ok(self.sequence(position))),
            END_OF_INPUT => None,
        }
    }

    fn repetition(&mut self, position: Position) -> Result<Token<'a>, Error> {
        let min = self.integer(position)?;
        match self.input.peek() {
            REPETITION_DIVIDER => {
                self.input.next();
                let max = self.integer(self.input.current_position())?;
                match self.input.peek() {
                    REPETITION_END => {
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
                    END_OF_INPUT => Err(Error::UnclosedRepetition { position }),
                }
            }
            REPETITION_END => {
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
            END_OF_INPUT => Err(Error::UnclosedRepetition { position }),
        }
    }

    fn set(&mut self, starting_position: Position) -> Result<Token<'a>, Error> {
        let mut members = Vec::new();
        // check if first character is '!'
        let negated = if self.input.peek() == SET_NEGATOR {
            self.input.next();
            true
        } else {
            false
        };
        // read set characters or categories until ']'
        loop {
            match self.input.peek() {
                // end set
                SET_END => {
                    self.input.next();
                    break;
                }
                // process subexpression (assuming category)
                SUBEXPRESSION_START => {
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
                ESCAPE => {
                    self.input.next();
                    match self.input.peek() {
                        c if c == ESCAPE || c == SUBEXPRESSION_START || c == SET_END => {
                            self.input.next();
                            members.push(SetMember::Character(c.unwrap()))
                        }
                        Some(c) => {
                            return Err(Error::InvalidSetEscape {
                                position: self.input.current_position(),
                                character: c,
                            })
                        }
                        END_OF_INPUT => {
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
                END_OF_INPUT => {
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
                SUBEXPRESSION_END => {
                    return Ok(Token::Subexpression(self.input.input_slice(start..end)))
                }
                Some(_) => end = self.input.current_position(),
                END_OF_INPUT => return Err(Error::UnclosedSubexpression { position }),
            }
        }
    }

    fn sequence(&mut self, start: Position) -> Token<'a> {
        loop {
            match self.input.peek() {
                ANY_CHAR | REPETITION_STAR | REPETITION_PLUS | REPETITION_OPTIONAL
                | LEFT_PARENTHESIS | RIGHT_PARENTHESIS | ALTERNATION | REPETITION_START
                | SET_START | SUBEXPRESSION_START | ESCAPE => break,
                Some(_) => {
                    self.input.next();
                }
                END_OF_INPUT => break,
            }
        }
        Token::Sequence(self.input.input_slice_from(start))
    }

    fn escaped(&mut self) -> Result<Token<'a>, Error> {
        let start = self.input.current_position();
        match self.input.next() {
            ANY_CHAR | REPETITION_STAR | REPETITION_PLUS | REPETITION_OPTIONAL
            | REPETITION_START | REPETITION_END | ESCAPE | SET_START | SET_END | ALTERNATION
            | LEFT_PARENTHESIS | RIGHT_PARENTHESIS | SUBEXPRESSION_START | SUBEXPRESSION_END => {
                Ok(Token::Sequence(self.input.input_slice_from(start)))
            }
            Some(c) => Err(Error::InvalidEscape {
                position: self.input.current_position(),
                character: Some(c),
            }),
            END_OF_INPUT => Err(Error::InvalidEscape {
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

impl<'a> CategoryLexer<'a> {
    pub fn new(input: &str) -> CategoryLexer {
        CategoryLexer {
            input: TextReader::new(input),
        }
    }

    pub fn next_token(&mut self) -> Option<Result<CategoryToken<'a>, Error>> {
        let position = self.input.current_position();
        match self.input.next() {
            ESCAPE => Some(self.escaped()),
            SUBEXPRESSION_START => Some(self.subexpression(position)),
            Some(_) => Some(Ok(self.sequence(position))),
            END_OF_INPUT => None,
        }
    }

    fn escaped(&mut self) -> Result<CategoryToken<'a>, Error> {
        let start = self.input.current_position();
        match self.input.next() {
            SUBEXPRESSION_START => Ok(CategoryToken::Sequence(self.input.input_slice_from(start))),
            Some(c) => Err(Error::InvalidEscape {
                position: self.input.current_position(),
                character: Some(c),
            }),
            END_OF_INPUT => Err(Error::InvalidEscape {
                position: self.input.current_position(),
                character: None,
            }),
        }
    }

    fn subexpression(&mut self, position: Position) -> Result<CategoryToken<'a>, Error> {
        let start = self.input.current_position();
        let mut end = self.input.current_position();
        loop {
            match self.input.next() {
                SUBEXPRESSION_END => {
                    return Ok(CategoryToken::Category(self.input.input_slice(start..end)))
                }
                Some(_) => end = self.input.current_position(),
                END_OF_INPUT => return Err(Error::UnclosedSubexpression { position }),
            }
        }
    }

    fn sequence(&mut self, position: Position) -> CategoryToken<'a> {
        loop {
            match self.input.peek() {
                SUBEXPRESSION_START | ESCAPE | END_OF_INPUT => break,
                Some(_) => {
                    self.input.next();
                }
            }
        }
        CategoryToken::Sequence(self.input.input_slice_from(position))
    }
}

impl<'a> Iterator for CategoryLexer<'a> {
    type Item = Result<CategoryToken<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
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
        let mut lexer = Lexer::new("{99}{-1}{2-}{2-7}{7-2}");
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
    }

    #[test]
    fn parse_repetition_errors() {
        let mut lexer = Lexer::new("{z}");
        assert_eq!(
            lexer.next(),
            Some(Err(Error::InvalidRepetitionCharacter {
                position: Position {
                    row: 1,
                    col: 2,
                    index: 1
                },
                character: 'z'
            }))
        );
        let mut lexer = Lexer::new("{-y}");
        assert_eq!(
            lexer.next(),
            Some(Err(Error::InvalidRepetitionCharacter {
                position: Position {
                    row: 1,
                    col: 3,
                    index: 2
                },
                character: 'y'
            }))
        );
        let mut lexer = Lexer::new("{");
        assert_eq!(
            lexer.next(),
            Some(Err(Error::UnclosedRepetition {
                position: Position {
                    row: 1,
                    col: 1,
                    index: 0
                },
            }))
        );
        let mut lexer = Lexer::new("{0-");
        assert_eq!(
            lexer.next(),
            Some(Err(Error::UnclosedRepetition {
                position: Position {
                    row: 1,
                    col: 1,
                    index: 0
                },
            }))
        );
        let mut lexer = Lexer::new("{5-1}");
        assert_eq!(
            lexer.next(),
            Some(Err(Error::InvalidRepetitionRange { min: 5, max: 1 }))
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
    fn parse_set_errors() {
        let mut lexer = Lexer::new("[se<t>\\<[\\]!\\\\");
        assert_eq!(
            lexer.next(),
            Some(Err(Error::UnclosedSet {
                position: Position {
                    row: 1,
                    col: 1,
                    index: 0
                }
            }))
        );
        let mut lexer = Lexer::new("[\\");
        assert_eq!(
            lexer.next(),
            Some(Err(Error::UnclosedSet {
                position: Position {
                    row: 1,
                    col: 1,
                    index: 0
                }
            }))
        );
        let mut lexer = Lexer::new("[\\x");
        assert_eq!(
            lexer.next(),
            Some(Err(Error::InvalidSetEscape {
                character: 'x',
                position: Position {
                    row: 1,
                    col: 3,
                    index: 2
                }
            }))
        );
        let mut lexer = Lexer::new("[<");
        assert_eq!(
            lexer.next(),
            Some(Err(Error::UnclosedSubexpression {
                position: Position {
                    row: 1,
                    col: 2,
                    index: 1
                }
            }))
        );
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

    #[test]
    fn parse_empty_category() {
        let mut lexer = CategoryLexer::new("");
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_sequence_category() {
        let mut lexer = CategoryLexer::new("abcd");
        assert_eq!(lexer.next(), Some(Ok(CategoryToken::Sequence("abcd"))));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_sequence_escape() {
        let mut lexer = CategoryLexer::new("\\<\\[");
        assert_eq!(lexer.next(), Some(Ok(CategoryToken::Sequence("<"))));
        assert_eq!(
            lexer.next(),
            Some(Err(Error::InvalidEscape {
                position: Position {
                    row: 1,
                    col: 5,
                    index: 4,
                },
                character: SET_START
            }))
        );
    }

    #[test]
    fn parse_category_category() {
        let mut lexer = CategoryLexer::new("<eyo>");
        assert_eq!(lexer.next(), Some(Ok(CategoryToken::Category("eyo"))));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_full_category() {
        let mut lexer = CategoryLexer::new("xx\\<<cat1><cat2>yy");
        assert_eq!(lexer.next(), Some(Ok(CategoryToken::Sequence("xx"))));
        assert_eq!(lexer.next(), Some(Ok(CategoryToken::Sequence("<"))));
        assert_eq!(lexer.next(), Some(Ok(CategoryToken::Category("cat1"))));
        assert_eq!(lexer.next(), Some(Ok(CategoryToken::Category("cat2"))));
        assert_eq!(lexer.next(), Some(Ok(CategoryToken::Sequence("yy"))));
        assert_eq!(lexer.next(), None);
    }
}
