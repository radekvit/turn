use crate::hir::SetMember;
use std::fmt;
use turn_utils::position::Position;
use turn_utils::text_reader::TextReader;
use turn_utils::token;

pub type RegexToken<'a> = token::Token<'a, RegexResult<'a>>;
pub type CategoryToken<'a> = token::Token<'a, CategoryResult<'a>>;

pub type RegexResult<'a> = Result<RegexTerminal<'a>, LexicalError>;
pub type CategoryResult<'a> = Result<CategoryNonterminal<'a>, LexicalError>;

const END_OF_INPUT: Option<char> = None;
const ESCAPE: char = '\\';
const ANY_CHAR: char = '_';
const ALTERNATION: char = '|';
const REPETITION_STAR: char = '*';
const REPETITION_PLUS: char = '+';
const REPETITION_OPTIONAL: char = '?';
const REPETITION_START: char = '{';
const REPETITION_END: char = '}';
const REPETITION_DIVIDER: char = '-';

const LEFT_PARENTHESIS: char = '(';
const RIGHT_PARENTHESIS: char = ')';
const SUBEXPRESSION_START: char = '<';
const SUBEXPRESSION_END: char = '>';
const SET_START: char = '[';
const SET_END: char = ']';
const SET_NEGATOR: char = '!';

#[derive(Debug)]
pub struct Lexer<'a> {
    input: TextReader<'a>,
}

#[derive(Debug)]
pub struct CategoryLexer<'a> {
    input: TextReader<'a>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RegexTerminal<'a> {
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
pub enum CategoryNonterminal<'a> {
    Sequence(&'a str),
    Category(&'a str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexicalError {
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

    pub fn token(
        &self,
        begin: Position,
        token: Result<RegexTerminal<'a>, LexicalError>,
    ) -> RegexToken<'a> {
        token::Token::from_reader(token, begin, &self.input)
    }

    pub fn next_token(&mut self) -> Option<RegexToken<'a>> {
        let position = self.input.current_position();
        let token = match self.input.next()? {
            ANY_CHAR => Ok(RegexTerminal::AnyChar),
            REPETITION_STAR => Ok(RegexTerminal::Repetition { min: 0, max: None }),
            REPETITION_PLUS => Ok(RegexTerminal::Repetition { min: 1, max: None }),
            REPETITION_OPTIONAL => Ok(RegexTerminal::Repetition {
                min: 0,
                max: Some(1),
            }),
            LEFT_PARENTHESIS => Ok(RegexTerminal::LParenthesis),
            RIGHT_PARENTHESIS => Ok(RegexTerminal::RParenthesis),
            ALTERNATION => Ok(RegexTerminal::Alternation),
            REPETITION_START => self.repetition(position),
            SET_START => self.set(position),
            SUBEXPRESSION_START => self.subexpression(position),
            ESCAPE => self.escaped(),
            _ => self.sequence(position),
        };
        Some(self.token(position, token))
    }

    fn repetition(&mut self, position: Position) -> RegexResult<'a> {
        let min = self.integer(position)?;
        let char_position = self.input.current_position();
        match self.input.next() {
            Some(REPETITION_DIVIDER) => {
                let max = self.integer(self.input.current_position())?;
                let char_position = self.input.current_position();
                match self.input.next() {
                    Some(REPETITION_END) => {
                        let min = min.unwrap_or(0);
                        if let Some(max) = max {
                            if min > max {
                                return Err(LexicalError::InvalidRepetitionRange { min, max });
                            }
                        }
                        Ok(RegexTerminal::Repetition { min, max })
                    }
                    Some(c) => Err(LexicalError::InvalidRepetitionCharacter {
                        position: char_position,
                        character: c,
                    }),
                    END_OF_INPUT => Err(LexicalError::UnclosedRepetition { position }),
                }
            }
            Some(REPETITION_END) => {
                let min = min.unwrap_or(0);
                Ok(RegexTerminal::Repetition {
                    min,
                    max: Some(min),
                })
            }
            Some(c) => Err(LexicalError::InvalidRepetitionCharacter {
                position: char_position,
                character: c,
            }),
            END_OF_INPUT => Err(LexicalError::UnclosedRepetition { position }),
        }
    }

    fn set(&mut self, position: Position) -> RegexResult<'a> {
        let mut members = Vec::new();
        // check if first character is '!'
        let negated = if self.input.peek() == Some(SET_NEGATOR) {
            self.input.next();
            true
        } else {
            false
        };
        // read set characters or categories until ']'
        loop {
            match self.input.peek() {
                // end set
                Some(SET_END) => {
                    self.input.next();
                    break;
                }
                // process subexpression (assuming category)
                Some(SUBEXPRESSION_START) => {
                    let start = self.input.current_position();
                    self.input.next();
                    let category = self.subexpression(start)?;
                    if let RegexTerminal::Subexpression(category) = category {
                        members.push(SetMember::Category(category));
                    } else {
                        unreachable!();
                    }
                }
                // escaped characters within sets
                Some(ESCAPE) => {
                    self.input.next();
                    let escaped_position = self.input.current_position();
                    match self.input.next() {
                        c if c == Some(ESCAPE)
                            || c == Some(SUBEXPRESSION_START)
                            || c == Some(SET_END) =>
                        {
                            members.push(SetMember::Character(c.unwrap()))
                        }
                        Some(c) => {
                            return Err(LexicalError::InvalidSetEscape {
                                position: escaped_position,
                                character: c,
                            })
                        }
                        END_OF_INPUT => return Err(LexicalError::UnclosedSet { position }),
                    }
                }
                Some(x) => {
                    self.input.next();
                    members.push(SetMember::Character(x));
                }
                END_OF_INPUT => {
                    return Err(LexicalError::UnclosedSet { position });
                }
            }
        }
        if negated {
            Ok(RegexTerminal::NegatedSet(members))
        } else {
            Ok(RegexTerminal::Set(members))
        }
    }

    fn subexpression(&mut self, position: Position) -> RegexResult<'a> {
        let start = self.input.current_position();
        let mut end = self.input.current_position();
        loop {
            match self.input.next() {
                Some(SUBEXPRESSION_END) => {
                    return Ok(RegexTerminal::Subexpression(
                        self.input.input_slice(start..end),
                    ))
                }
                Some(_) => end = self.input.current_position(),
                END_OF_INPUT => return Err(LexicalError::UnclosedSubexpression { position }),
            }
        }
    }

    fn sequence(&mut self, position: Position) -> RegexResult<'a> {
        loop {
            match self.input.peek() {
                Some(ANY_CHAR)
                | Some(REPETITION_STAR)
                | Some(REPETITION_PLUS)
                | Some(REPETITION_OPTIONAL)
                | Some(LEFT_PARENTHESIS)
                | Some(RIGHT_PARENTHESIS)
                | Some(ALTERNATION)
                | Some(REPETITION_START)
                | Some(SET_START)
                | Some(SUBEXPRESSION_START)
                | Some(ESCAPE) => break,
                Some(_) => {
                    self.input.next();
                }
                END_OF_INPUT => break,
            }
        }
        Ok(RegexTerminal::Sequence(
            self.input.input_slice_from(position),
        ))
    }

    fn escaped(&mut self) -> RegexResult<'a> {
        let start = self.input.current_position();
        match self.input.next() {
            Some(ANY_CHAR)
            | Some(REPETITION_STAR)
            | Some(REPETITION_PLUS)
            | Some(REPETITION_OPTIONAL)
            | Some(REPETITION_START)
            | Some(REPETITION_END)
            | Some(ESCAPE)
            | Some(SET_START)
            | Some(SET_END)
            | Some(ALTERNATION)
            | Some(LEFT_PARENTHESIS)
            | Some(RIGHT_PARENTHESIS)
            | Some(SUBEXPRESSION_START)
            | Some(SUBEXPRESSION_END) => {
                Ok(RegexTerminal::Sequence(self.input.input_slice_from(start)))
            }
            Some(c) => Err(LexicalError::InvalidEscape {
                position: self.input.current_position(),
                character: Some(c),
            }),
            END_OF_INPUT => Err(LexicalError::InvalidEscape {
                position: self.input.current_position(),
                character: None,
            }),
        }
    }

    fn integer(&mut self, position: Position) -> Result<Option<u16>, LexicalError> {
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
                        .ok_or(LexicalError::RangeIntegerOverflow { position })?;
                    self.input.next();
                }
                _ => return Ok(Some(number)),
            }
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = RegexToken<'a>;

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

    pub fn next_token(&mut self) -> Option<CategoryToken<'a>> {
        let position = self.input.current_position();
        let token = match self.input.next()? {
            ESCAPE => self.escaped(position),
            SUBEXPRESSION_START => self.subexpression(position),
            _ => self.sequence(position),
        };
        Some(CategoryToken::from_reader(token, position, &self.input))
    }

    fn escaped(&mut self, position: Position) -> CategoryResult<'a> {
        let start = self.input.current_position();
        match self.input.next() {
            Some(SUBEXPRESSION_START) => Ok(CategoryNonterminal::Sequence(
                self.input.input_slice_from(start),
            )),
            Some(c) => Err(LexicalError::InvalidEscape {
                position: self.input.current_position(),
                character: Some(c),
            }),
            END_OF_INPUT => Err(LexicalError::InvalidEscape {
                position: self.input.current_position(),
                character: None,
            }),
        }
    }

    fn subexpression(&mut self, position: Position) -> CategoryResult<'a> {
        let start = self.input.current_position();
        let mut end = self.input.current_position();
        loop {
            match self.input.next() {
                Some(SUBEXPRESSION_END) => {
                    return Ok(CategoryNonterminal::Category(
                        self.input.input_slice(start..end),
                    ))
                }
                Some(_) => end = self.input.current_position(),
                END_OF_INPUT => return Err(LexicalError::UnclosedSubexpression { position }),
            }
        }
    }

    fn sequence(&mut self, position: Position) -> CategoryResult<'a> {
        loop {
            match self.input.peek() {
                Some(SUBEXPRESSION_START) | Some(ESCAPE) | END_OF_INPUT => break,
                Some(_) => {
                    self.input.next();
                }
            }
        }
        Ok(CategoryNonterminal::Sequence(
            self.input.input_slice_from(position),
        ))
    }
}

impl<'a> Iterator for CategoryLexer<'a> {
    type Item = CategoryToken<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            LexicalError::UnclosedSet { position } => write!(
                f,
                "the set starting at {}:{} is missing the closing character ']'",
                position.row, position.col
            ),
            LexicalError::UnclosedSubexpression { position } => write!(
                f,
                "the subexpression or category starting at {}:{}\
                 is missing the closing character '>'",
                position.row, position.col
            ),
            LexicalError::InvalidRepetitionRange { min, max } => write!(
                f,
                "invalid repetition range: {} is greater than {}",
                min, max
            ),
            LexicalError::InvalidRepetitionCharacter {
                position,
                character,
            } => write!(
                f,
                "invalid character '{}' inside a repetition range at position {}:{}",
                character, position.row, position.col
            ),
            LexicalError::InvalidSetEscape {
                position,
                character,
            } => write!(
                f,
                "invalid escaped character '{}' inside a set at position {}:{}",
                character, position.row, position.col
            ),
            LexicalError::InvalidEscape {
                position,
                character,
            } => {
                if let Some(c) = character {
                    write!(
                        f,
                        "invalid escaped character '{}' at position {}:{}",
                        c, position.row, position.col
                    )
                } else {
                    write!(
                        f,
                        "unexpected end of input after '\\' at position {}:{}",
                        position.row, position.col
                    )
                }
            }
            LexicalError::UnclosedRepetition { position } => write!(
                f,
                "unexpected end of input inside range specifier at position {}:{}",
                position.row, position.col
            ),
            LexicalError::RangeIntegerOverflow { position } => write!(
                f,
                "integer range over 65_536 at position {}:{}",
                position.row, position.col
            ),
        }
    }
}

impl std::error::Error for LexicalError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Range;

    fn position_range(cols: Range<usize>, indices: Range<usize>) -> Range<Position> {
        Position {
            row: 1,
            col: cols.start,
            index: indices.start,
        }..Position {
            row: 1,
            col: cols.end,
            index: indices.end,
        }
    }

    #[test]
    fn parse_empty() {
        let mut lexer = Lexer::new("");
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_sequence() {
        let mut lexer = Lexer::new("💣b東x#e#ß");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::Sequence("💣b東x#e#ß")),
                position: position_range(1..9, 0..14),
                slice: "💣b東x#e#ß",
            })
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_anychar() {
        let mut lexer = Lexer::new("_");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::AnyChar),
                position: position_range(1..2, 0..1),
                slice: "_",
            })
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_repetition_star() {
        let mut lexer = Lexer::new("*");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::Repetition { min: 0, max: None }),
                position: position_range(1..2, 0..1),
                slice: "*",
            })
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_repetition_plus() {
        let mut lexer = Lexer::new("+");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::Repetition { min: 1, max: None }),
                position: position_range(1..2, 0..1),
                slice: "+",
            })
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_repetition_question_mark() {
        let mut lexer = Lexer::new("?");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::Repetition {
                    min: 0,
                    max: Some(1)
                }),
                position: position_range(1..2, 0..1),
                slice: "?",
            })
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_repetition_variants() {
        let mut lexer = Lexer::new("{99}{-1}{2-}{2-7}{7-2}");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::Repetition {
                    min: 99,
                    max: Some(99)
                }),
                position: position_range(1..5, 0..4),
                slice: "{99}",
            })
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::Repetition {
                    min: 0,
                    max: Some(1)
                }),
                position: position_range(5..9, 4..8),
                slice: "{-1}",
            })
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::Repetition { min: 2, max: None }),
                position: position_range(9..13, 8..12),
                slice: "{2-}",
            })
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::Repetition {
                    min: 2,
                    max: Some(7)
                }),
                position: position_range(13..18, 12..17),
                slice: "{2-7}",
            })
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Err(LexicalError::InvalidRepetitionRange { min: 7, max: 2 }),
                position: position_range(18..23, 17..22),
                slice: "{7-2}",
            })
        );
    }

    #[test]
    fn parse_repetition_errors() {
        let mut lexer = Lexer::new("{z}");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Err(LexicalError::InvalidRepetitionCharacter {
                    position: Position {
                        row: 1,
                        col: 2,
                        index: 1
                    },
                    character: 'z'
                }),
                position: position_range(1..3, 0..2),
                slice: "{z",
            })
        );
        let mut lexer = Lexer::new("{-y}");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Err(LexicalError::InvalidRepetitionCharacter {
                    position: Position {
                        row: 1,
                        col: 3,
                        index: 2
                    },
                    character: 'y'
                }),
                position: position_range(1..4, 0..3),
                slice: "{-y",
            })
        );
        let mut lexer = Lexer::new("{");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Err(LexicalError::UnclosedRepetition {
                    position: Position {
                        row: 1,
                        col: 1,
                        index: 0
                    },
                }),
                slice: "{",
                position: position_range(1..2, 0..1)
            })
        );
        let mut lexer = Lexer::new("{0-");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Err(LexicalError::UnclosedRepetition {
                    position: Position {
                        row: 1,
                        col: 1,
                        index: 0
                    },
                }),
                slice: "{0-",
                position: position_range(1..4, 0..3)
            })
        );
        let mut lexer = Lexer::new("{5-1}");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Err(LexicalError::InvalidRepetitionRange { min: 5, max: 1 }),
                slice: "{5-1}",
                position: position_range(1..6, 0..5)
            })
        );
    }

    #[test]
    fn parse_set() {
        let mut lexer = Lexer::new("[se<t>\\<[\\]!\\\\]");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::Set(vec![
                    SetMember::Character('s'),
                    SetMember::Character('e'),
                    SetMember::Category("t"),
                    SetMember::Character('<'),
                    SetMember::Character('['),
                    SetMember::Character(']'),
                    SetMember::Character('!'),
                    SetMember::Character('\\'),
                ])),
                position: position_range(1..16, 0..15),
                slice: "[se<t>\\<[\\]!\\\\]",
            })
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_set_errors() {
        let mut lexer = Lexer::new("[se<t>\\<[\\]!\\\\");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Err(LexicalError::UnclosedSet {
                    position: Position {
                        row: 1,
                        col: 1,
                        index: 0
                    }
                }),
                slice: "[se<t>\\<[\\]!\\\\",
                position: position_range(1..15, 0..14)
            })
        );
        let mut lexer = Lexer::new("[\\");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Err(LexicalError::UnclosedSet {
                    position: Position {
                        row: 1,
                        col: 1,
                        index: 0
                    }
                }),
                slice: "[\\",
                position: position_range(1..3, 0..2)
            })
        );
        let mut lexer = Lexer::new("[\\x");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Err(LexicalError::InvalidSetEscape {
                    character: 'x',
                    position: Position {
                        row: 1,
                        col: 3,
                        index: 2
                    }
                }),
                slice: "[\\x",
                position: position_range(1..4, 0..3)
            })
        );
        let mut lexer = Lexer::new("[<");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Err(LexicalError::UnclosedSubexpression {
                    position: Position {
                        row: 1,
                        col: 2,
                        index: 1
                    }
                }),
                slice: "[<",
                position: position_range(1..3, 0..2)
            })
        );
    }

    #[test]
    fn parse_negated_set() {
        let mut lexer = Lexer::new("[!se<t>\\<[\\]!\\\\]");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::NegatedSet(vec![
                    SetMember::Character('s'),
                    SetMember::Character('e'),
                    SetMember::Category("t"),
                    SetMember::Character('<'),
                    SetMember::Character('['),
                    SetMember::Character(']'),
                    SetMember::Character('!'),
                    SetMember::Character('\\'),
                ])),
                position: position_range(1..17, 0..16),
                slice: "[!se<t>\\<[\\]!\\\\]"
            })
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_alternation() {
        let mut lexer = Lexer::new("|");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::Alternation),
                position: position_range(1..2, 0..1),
                slice: "|",
            })
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_left_parenthesis() {
        let mut lexer = Lexer::new("(");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::LParenthesis),
                position: position_range(1..2, 0..1),
                slice: "(",
            })
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_right_parenthesis() {
        let mut lexer = Lexer::new(")");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::RParenthesis),
                position: position_range(1..2, 0..1),
                slice: ")",
            })
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_subexpression() {
        let mut lexer = Lexer::new("<subexpr>");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::Subexpression("subexpr")),
                position: position_range(1..10, 0..9),
                slice: "<subexpr>",
            })
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_subexpression_unclosed() {
        let mut lexer = Lexer::new("<subexpr");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Err(LexicalError::UnclosedSubexpression {
                    position: Position {
                        row: 1,
                        col: 1,
                        index: 0
                    }
                }),
                position: position_range(1..9, 0..8),
                slice: "<subexpr",
            })
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_escaped_chars() {
        let mut lexer = Lexer::new("\\_\\*\\+\\?\\{\\}\\\\\\[\\]\\|\\(\\)\\<\\>");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                slice: "\\_",
                position: position_range(1..3, 0..2),
                token: Ok(RegexTerminal::Sequence("_")),
            })
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                slice: "\\*",
                position: position_range(3..5, 2..4),
                token: Ok(RegexTerminal::Sequence("*")),
            })
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                slice: "\\+",
                position: position_range(5..7, 4..6),
                token: Ok(RegexTerminal::Sequence("+")),
            })
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                slice: "\\?",
                position: position_range(7..9, 6..8),
                token: Ok(RegexTerminal::Sequence("?")),
            })
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                slice: "\\{",
                position: position_range(9..11, 8..10),
                token: Ok(RegexTerminal::Sequence("{")),
            })
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                slice: "\\}",
                position: position_range(11..13, 10..12),
                token: Ok(RegexTerminal::Sequence("}")),
            })
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                slice: "\\\\",
                position: position_range(13..15, 12..14),
                token: Ok(RegexTerminal::Sequence("\\")),
            })
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                slice: "\\[",
                position: position_range(15..17, 14..16),
                token: Ok(RegexTerminal::Sequence("[")),
            })
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                slice: "\\]",
                position: position_range(17..19, 16..18),
                token: Ok(RegexTerminal::Sequence("]")),
            })
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                slice: "\\|",
                position: position_range(19..21, 18..20),
                token: Ok(RegexTerminal::Sequence("|")),
            })
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                slice: "\\(",
                position: position_range(21..23, 20..22),
                token: Ok(RegexTerminal::Sequence("(")),
            })
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                slice: "\\)",
                position: position_range(23..25, 22..24),
                token: Ok(RegexTerminal::Sequence(")")),
            })
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                slice: "\\<",
                position: position_range(25..27, 24..26),
                token: Ok(RegexTerminal::Sequence("<")),
            })
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                slice: "\\>",
                position: position_range(27..29, 26..28),
                token: Ok(RegexTerminal::Sequence(">")),
            })
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_multiple() {
        let mut lexer = Lexer::new("a💣bß>ř_*+?{42}{5-}{-5}{4-89}[ab<cat><ccat>][!x]|()<sub>");
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::Sequence("a💣bß>ř")),
                position: position_range(1..7, 0..11),
                slice: "a💣bß>ř",
            }),
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::AnyChar),
                position: position_range(7..8, 11..12),
                slice: "_",
            }),
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::Repetition { min: 0, max: None }),
                position: position_range(8..9, 12..13),
                slice: "*",
            }),
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::Repetition { min: 1, max: None }),
                position: position_range(9..10, 13..14),
                slice: "+",
            }),
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::Repetition {
                    min: 0,
                    max: Some(1)
                }),
                position: position_range(10..11, 14..15),
                slice: "?",
            }),
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::Repetition {
                    min: 42,
                    max: Some(42)
                }),
                position: position_range(11..15, 15..19),
                slice: "{42}",
            }),
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::Repetition { min: 5, max: None }),
                position: position_range(15..19, 19..23),
                slice: "{5-}",
            }),
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::Repetition {
                    min: 0,
                    max: Some(5)
                }),
                position: position_range(19..23, 23..27),
                slice: "{-5}",
            }),
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::Repetition {
                    min: 4,
                    max: Some(89)
                }),
                position: position_range(23..29, 27..33),
                slice: "{4-89}",
            }),
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::Set(vec![
                    SetMember::Character('a'),
                    SetMember::Character('b'),
                    SetMember::Category("cat"),
                    SetMember::Category("ccat"),
                ])),
                position: position_range(29..44, 33..48),
                slice: "[ab<cat><ccat>]",
            }),
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::NegatedSet(vec![SetMember::Character('x'),])),
                position: position_range(44..48, 48..52),
                slice: "[!x]",
            }),
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::Alternation),
                position: position_range(48..49, 52..53),
                slice: "|",
            }),
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::LParenthesis),
                position: position_range(49..50, 53..54),
                slice: "(",
            }),
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::RParenthesis),
                position: position_range(50..51, 54..55),
                slice: ")",
            }),
        );
        assert_eq!(
            lexer.next(),
            Some(RegexToken {
                token: Ok(RegexTerminal::Subexpression("sub")),
                position: position_range(51..56, 55..60),
                slice: "<sub>",
            }),
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_empty_category() {
        let mut lexer = CategoryLexer::new("");
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_sequence_category() {
        let mut lexer = CategoryLexer::new("abcd");
        assert_eq!(
            lexer.next(),
            Some(CategoryToken {
                token: Ok(CategoryNonterminal::Sequence("abcd")),
                position: position_range(1..5, 0..4),
                slice: "abcd",
            })
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_sequence_escape() {
        let mut lexer = CategoryLexer::new("\\<\\[");
        assert_eq!(
            lexer.next(),
            Some(CategoryToken {
                token: Ok(CategoryNonterminal::Sequence("<")),
                position: position_range(1..3, 0..2),
                slice: "\\<",
            })
        );
        assert_eq!(
            lexer.next(),
            Some(CategoryToken {
                token: Err(LexicalError::InvalidEscape {
                    position: Position {
                        row: 1,
                        col: 5,
                        index: 4,
                    },
                    character: Some(SET_START)
                }),
                position: position_range(3..5, 2..4),
                slice: "\\["
            })
        );
    }

    #[test]
    fn parse_category_category() {
        let mut lexer = CategoryLexer::new("<eyo>");
        assert_eq!(
            lexer.next(),
            Some(CategoryToken {
                token: Ok(CategoryNonterminal::Category("eyo")),
                position: position_range(1..6, 0..5),
                slice: "<eyo>"
            })
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn parse_full_category() {
        let mut lexer = CategoryLexer::new("xx\\<<cat1><cat2>yy");
        assert_eq!(
            lexer.next(),
            Some(CategoryToken {
                token: Ok(CategoryNonterminal::Sequence("xx")),
                position: position_range(1..3, 0..2),
                slice: "xx",
            })
        );
        assert_eq!(
            lexer.next(),
            Some(CategoryToken {
                token: Ok(CategoryNonterminal::Sequence("<")),
                position: position_range(3..5, 2..4),
                slice: "\\<",
            })
        );
        assert_eq!(
            lexer.next(),
            Some(CategoryToken {
                token: Ok(CategoryNonterminal::Category("cat1")),
                position: position_range(5..11, 4..10),
                slice: "<cat1>",
            })
        );
        assert_eq!(
            lexer.next(),
            Some(CategoryToken {
                token: Ok(CategoryNonterminal::Category("cat2")),
                position: position_range(11..17, 10..16),
                slice: "<cat2>"
            })
        );
        assert_eq!(
            lexer.next(),
            Some(CategoryToken {
                token: Ok(CategoryNonterminal::Sequence("yy")),
                position: position_range(17..19, 16..18),
                slice: "yy"
            })
        );
        assert_eq!(lexer.next(), None);
    }
}
