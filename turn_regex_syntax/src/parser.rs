use crate::lexer;
use crate::lexer::Token;
use crate::HIR;
use std::convert::From;
use std::fmt;

#[derive(Debug)]
pub enum ParserError {
    StandaloneRepetition,
    UnexpectedRParenthesis,
}

impl From<ParserError> for Error {
    fn from(error: ParserError) -> Error {
        Error::ParserError(error)
    }
}

#[derive(Debug)]
pub enum Error {
    LexerError(lexer::Error),
    ParserError(ParserError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parsing error")
    }
}

impl std::error::Error for Error {}

impl From<lexer::Error> for Error {
    fn from(error: lexer::Error) -> Error {
        Error::LexerError(error)
    }
}

pub fn parse<'a, Iter>(mut input: Iter) -> Result<HIR<'a>, Error>
where
    Iter: Iterator<Item = Result<Token<'a>, lexer::Error>>,
{
    parse_regex(&mut input, &match_end)
}

fn match_end<'a>(token: &Option<Result<Token<'a>, lexer::Error>>) -> bool {
    token.is_none()
}

fn match_right_parenthesis<'a>(token: &Option<Result<Token<'a>, lexer::Error>>) -> bool {
    *token == Some(Ok(Token::RParenthesis))
}

fn parse_regex<'a, Iter, F>(input: &mut Iter, terminate: &F) -> Result<HIR<'a>, Error>
where
    Iter: Iterator<Item = Result<Token<'a>, lexer::Error>>,
    F: Fn(&Option<Result<Token<'a>, lexer::Error>>) -> bool,
{
    let mut regexes = vec![];
    loop {
        let token = input.next();
        if terminate(&token) {
            break;
        }
        let token = token.unwrap();
        match token? {
            Token::Sequence(sequence) => regexes.push(HIR::Sequence(sequence)),
            Token::AnyChar => regexes.push(HIR::AnyChar),
            Token::Repetition { min, max } => {
                if regexes.is_empty() {
                    return Err(ParserError::StandaloneRepetition.into());
                }
                let last = regexes.remove(regexes.len() - 1);
                regexes.push(HIR::Repetition {
                    regex: Box::new(last),
                    min,
                    max,
                });
            }
            Token::Set(members) => regexes.push(HIR::Set(members)),
            Token::NegatedSet(members) => regexes.push(HIR::NegatedSet(members)),
            Token::Alternation => {
                let mut left_alternative = Vec::new();
                std::mem::swap(&mut regexes, &mut left_alternative);
                let right_alternative = parse_regex(input, terminate)?;
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
            Token::LParenthesis => regexes.push(parse_regex(input, &match_right_parenthesis)?),
            Token::RParenthesis => return Err(ParserError::UnexpectedRParenthesis.into()),
            Token::Subexpression(subexpression) => regexes.push(HIR::SubRegex(subexpression)),
        }
    }
    if regexes.len() == 1 {
        Ok(regexes.remove(0))
    } else {
        Ok(HIR::Concatenation(regexes))
    }
}
