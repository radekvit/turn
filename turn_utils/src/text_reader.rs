use std::ops::Range;
use std::str::Chars;

/// A position in an input string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Position {
    /// The row of the read character
    pub row: usize,
    // The col of the last read character
    pub col: usize,
    /// The character position based on the read character's utf8 widths.
    pub index: usize,
}

/// Input text reader. Acts as an iterator over the input characters and allows peeking.
/// Provides built-in functionality for obtaining input slices from the read characters.
///
/// # Example
/// ```
/// # use turn_utils::text_reader::TextReader;
/// let input = "you da 💣".to_owned();
///
/// let mut reader = TextReader::new(&input);
/// for _ in 0..4 {
///     reader.next();
/// }
/// assert_eq!(reader.peek(), Some('d'));
/// let position = reader.current_position();
/// for _ in 0..3 {
///     reader.next();
/// }
/// assert_eq!(reader.next(), Some('💣'));
/// assert_eq!(reader.next(), None);
/// assert_eq!(reader.input_slice_from(position), "da 💣");
/// ```
#[derive(Debug, Clone)]
pub struct TextReader<'a> {
    input: &'a str,
    peek: Option<char>,
    iter: Chars<'a>,
    position: Position,
}

impl Position {
    /// Create a new Position representing the first character of any input.
    pub fn new() -> Self {
        Default::default()
    }

    /// Advance the position after reading a specific character.
    ///
    /// Advancing the position after a newline character
    /// will increment the row and set the column to 1.
    pub fn advance(&mut self, character: char) {
        if character == '\n' {
            self.row += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        self.index += character.len_utf8();
    }
}

impl Default for Position {
    fn default() -> Position {
        Position {
            row: 1,
            col: 1,
            index: 0,
        }
    }
}

impl<'a> TextReader<'a> {
    /// Create a new TextReader from an input slice.
    #[inline]
    pub fn new(input: &str) -> TextReader {
        let mut iter = input.chars();
        let peek = iter.next();
        TextReader {
            input,
            peek,
            iter,
            position: Default::default(),
        }
    }

    /// Peek the next character from the input.
    ///
    /// # Example
    /// ```
    /// # use turn_utils::text_reader::TextReader;
    /// let mut reader = TextReader::new("-_-");
    /// reader.next();
    ///
    /// assert_eq!(reader.peek(), Some('_'));
    /// assert_eq!(reader.peek(), Some('_'));
    /// ```
    #[inline]
    pub fn peek(&self) -> Option<char> {
        self.peek
    }

    /// Read the next character from the input.
    fn read_next(&mut self) -> Option<char> {
        let next = self.peek;
        if let Some(c) = self.peek {
            self.position.advance(c);
        }
        self.peek = self.iter.next();
        next
    }

    /// Get the current position of the read text.
    #[inline]
    pub fn current_position(&self) -> Position {
        self.position
    }

    /// Get a slice of the input between the two positions.
    ///
    /// # Example
    /// ```
    /// # use turn_utils::text_reader::TextReader;
    /// let mut reader = TextReader::new("--_--");
    /// reader.next();
    /// let from = reader.current_position();
    /// for _ in 0..3 {
    ///     reader.next();
    /// }
    /// let to = reader.current_position();
    /// assert_eq!(reader.input_slice(from..to), "-_-");
    /// ```
    #[inline]
    pub fn input_slice(&self, range: Range<Position>) -> &'a str {
        &self.input[range.start.index..range.end.index]
    }

    /// Get a slice of the input between the supplied position
    /// and the position of the last read character.
    ///
    /// # Example
    /// ```
    /// # use turn_utils::text_reader::TextReader;
    /// let mut reader = TextReader::new("--_--");
    /// reader.next();
    /// let from = reader.current_position();
    /// for _ in 0..3 {
    ///     reader.next();
    /// }
    /// assert_eq!(reader.input_slice_from(from), "-_-");
    /// ```
    #[inline]
    pub fn input_slice_from(&self, from: Position) -> &'a str {
        self.input_slice(from..self.position)
    }
}

impl Iterator for TextReader<'_> {
    type Item = char;

    /// Read the next character from the input.
    ///
    /// # Example
    /// ```
    /// # use turn_utils::text_reader::TextReader;
    /// let mut reader = TextReader::new("-_-");
    ///
    /// assert_eq!(reader.next(), Some('-'));
    /// assert_eq!(reader.next(), Some('_'));
    /// assert_eq!(reader.next(), Some('-'));
    /// assert_eq!(reader.next(), None);
    /// ```
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.read_next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advance_position() {
        let mut position = Position {
            row: 42,
            col: 42,
            index: 69,
        };
        position.advance('ß');
        assert_eq!(
            position,
            Position {
                row: 42,
                col: 43,
                index: 71
            }
        );
        position.advance('\n');
        assert_eq!(
            position,
            Position {
                row: 43,
                col: 1,
                index: 72
            }
        );
    }

    #[test]
    fn peek_characters() {
        let mut reader = TextReader::new("ℝbcd");
        assert_eq!(reader.peek(), Some('ℝ'));
        assert_eq!(reader.peek(), Some('ℝ'));

        reader.next();

        assert_eq!(reader.peek(), Some('b'));
        assert_eq!(reader.peek(), Some('b'));
    }

    #[test]
    fn read_characters() {
        let mut reader = TextReader::new("ℝb💣");

        assert_eq!(reader.next(), Some('ℝ'));
        assert_eq!(reader.next(), Some('b'));
        assert_eq!(reader.next(), Some('💣'));
        assert_eq!(reader.next(), None);
    }

    #[test]
    fn reader_positions() {
        let mut reader = TextReader::new("ℝb\n💣");
        assert_eq!(
            reader.current_position(),
            Position {
                row: 1,
                col: 1,
                index: 0
            }
        );
        reader.next();
        assert_eq!(
            reader.current_position(),
            Position {
                row: 1,
                col: 2,
                index: 3
            }
        );
        reader.next();
        assert_eq!(
            reader.current_position(),
            Position {
                row: 1,
                col: 3,
                index: 4
            }
        );
        reader.next();
        assert_eq!(
            reader.current_position(),
            Position {
                row: 2,
                col: 1,
                index: 5
            }
        );
        reader.next();
        assert_eq!(
            reader.current_position(),
            Position {
                row: 2,
                col: 2,
                index: 9
            }
        );
        reader.next();
        assert_eq!(
            reader.current_position(),
            Position {
                row: 2,
                col: 2,
                index: 9
            }
        );
    }

    #[test]
    fn read_input_slices() {
        let mut reader = TextReader::new("abcℝb💣def");
        assert_eq!(
            reader.input_slice(
                Position {
                    col: 4,
                    row: 1,
                    index: 3
                }..Position {
                    col: 7,
                    row: 1,
                    index: 11
                }
            ),
            "ℝb💣"
        );
        for _ in 0..3 {
            reader.next();
        }
        let from = reader.current_position();
        for _ in 0..3 {
            reader.next();
        }
        assert_eq!(reader.input_slice_from(from), "ℝb💣");
    }

    #[test]
    fn peek_matches_read() {
        let mut reader = TextReader::new("xℝy");

        for _ in 0..6 {
            let peek = reader.peek();
            let next = reader.next();
            assert_eq!(peek, next);
        }
    }
}
