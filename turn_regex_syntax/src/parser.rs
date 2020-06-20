use crate::hir::SetMember;
use crate::hir::HIR;
use crate::lexer::{CategoryTerminal, CategoryToken, LexicalError, RegexTerminal, RegexToken};
use std::convert::From;
use std::fmt;

#[derive(Debug)]
pub enum ParsingError {
    StandaloneRepetition,
    UnexpectedRParenthesis,
}

#[derive(Debug)]
pub enum Error {
    LexicalError(LexicalError),
    ParsingError(ParsingError),
}

pub fn parse_regex<'a, Iter>(mut input: Iter) -> Result<HIR<'a>, Error>
where
    Iter: Iterator<Item = RegexToken<'a>>,
{
    parse_regex_to(&mut input, &match_end)
}

pub fn parse_category<'a, Iter>(mut input: Iter) -> Result<HIR<'a>, Error>
where
    Iter: Iterator<Item = CategoryToken<'a>>,
{
    let mut set_members = vec![];
    loop {
        let token = input.next();
        if token.is_none() {
            break;
        }
        let token = token.unwrap();
        match token.token? {
            CategoryTerminal::Sequence(members) => members
                .chars()
                .for_each(|c| set_members.push(SetMember::Character(c))),
            CategoryTerminal::Category(category) => set_members.push(SetMember::Category(category)),
        }
    }
    Ok(HIR::Set(set_members))
}

fn match_end<'a>(token: &Option<RegexToken<'a>>) -> bool {
    token.is_none()
}

fn match_right_parenthesis<'a>(token: &Option<RegexToken<'a>>) -> bool {
    if let Some(token) = token {
        if let Ok(ref token) = token.token {
            *token == RegexTerminal::RParenthesis
        } else {
            false
        }
    } else {
        false
    }
}

fn parse_regex_to<'a, Iter, F>(input: &mut Iter, terminate: &F) -> Result<HIR<'a>, Error>
where
    Iter: Iterator<Item = RegexToken<'a>>,
    F: Fn(&Option<RegexToken<'a>>) -> bool,
{
    let mut regexes = vec![];
    loop {
        let token = input.next();
        if terminate(&token) {
            break;
        }
        let token = token.unwrap();
        match token.token? {
            RegexTerminal::Sequence(sequence) => regexes.push(HIR::Sequence(sequence)),
            RegexTerminal::AnyChar => regexes.push(HIR::AnyChar),
            RegexTerminal::Repetition { min, max } => {
                if regexes.is_empty() {
                    return Err(ParsingError::StandaloneRepetition.into());
                }
                let last = regexes.remove(regexes.len() - 1);
                regexes.push(HIR::Repetition {
                    regex: Box::new(last),
                    min,
                    max,
                });
            }
            RegexTerminal::Set(members) => regexes.push(HIR::Set(members)),
            RegexTerminal::NegatedSet(members) => regexes.push(HIR::NegatedSet(members)),
            RegexTerminal::Alternation => {
                let mut left_alternative = Vec::new();
                std::mem::swap(&mut regexes, &mut left_alternative);
                let right_alternative = parse_regex_to(input, terminate)?;
                let left_alternative = if left_alternative.len() == 1 {
                    left_alternative.remove(0)
                } else {
                    HIR::Concatenation(left_alternative)
                };
                match right_alternative {
                    HIR::Alternation(mut alternatives) => {
                        alternatives.insert(0, left_alternative);
                        regexes.push(HIR::Alternation(alternatives))
                    }
                    right_alternative => {
                        regexes.push(HIR::Alternation(vec![left_alternative, right_alternative]))
                    }
                }
                break;
            }
            RegexTerminal::LParenthesis => {
                regexes.push(parse_regex_to(input, &match_right_parenthesis)?)
            }
            RegexTerminal::RParenthesis => return Err(ParsingError::UnexpectedRParenthesis.into()),
            RegexTerminal::Subexpression(subexpression) => {
                regexes.push(HIR::SubRegex(subexpression))
            }
        }
    }
    if regexes.len() == 1 {
        Ok(regexes.remove(0))
    } else {
        Ok(HIR::Concatenation(regexes))
    }
}

impl fmt::Display for ParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ParsingError::StandaloneRepetition => write!(f, "todo"),
            ParsingError::UnexpectedRParenthesis => write!(f, "todo"),
        }
    }
}

impl std::error::Error for ParsingError {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ParsingError(error) => error.fmt(f),
            Error::LexicalError(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<LexicalError> for Error {
    fn from(error: LexicalError) -> Error {
        Error::LexicalError(error)
    }
}

impl From<ParsingError> for Error {
    fn from(error: ParsingError) -> Error {
        Error::ParsingError(error)
    }
}
