use crate::position::Position;
use crate::text_reader::TextReader;
use std::ops::Range;

/// A token struct encoding the token itself, its position in the source,
/// and the slice of the input the token was obtained from.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token<'a, T> {
    pub token: T,
    pub position: Range<Position>,
    pub slice: &'a str,
}

impl<'a, T> Token<'a, T> {
    /// Create a Token using an input reader.
    ///
    /// The input reader is expected to have read the last character of the token
    /// in order to correctly extract the token's position and input slice.
    ///
    /// # Example
    /// ```
    /// # use turn_utils::token::Token;
    /// # use turn_utils::position::Position;
    /// # use turn_utils::text_reader::TextReader;
    /// let mut reader = TextReader::new("/*-");
    /// let position = reader.current_position();
    /// // read two characters
    /// reader.next();
    /// reader.next();
    /// let token = Token::from_reader("*/", position, &reader);
    /// assert_eq!(token.token, "*/");
    /// assert_eq!(token.slice, "/*");
    /// assert_eq!(token.position,
    ///     Position {
    ///         row: 1,
    ///         col: 1,
    ///         index: 0
    ///     }..Position{
    ///         row: 1,
    ///         col: 3,
    ///         index: 2
    ///     }
    /// );
    /// ```
    pub fn from_reader(token: T, begin: Position, reader: &TextReader<'a>) -> Token<'a, T> {
        let position = begin..reader.current_position();
        let slice = reader.input_slice(position.clone());
        Token {
            token,
            position,
            slice,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_reader() {
        let mut reader = TextReader::new("abcdef");
        reader.next();
        let position = reader.current_position();
        for _ in 0..3 {
            reader.next();
        }
        let token = Token::from_reader('x', position, &reader);

        assert_eq!(
            token,
            Token {
                token: 'x',
                slice: "bcd",
                position: Position {
                    row: 1,
                    col: 2,
                    index: 1
                }..Position {
                    row: 1,
                    col: 5,
                    index: 4
                },
            }
        );
    }
}
