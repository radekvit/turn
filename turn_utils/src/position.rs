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

impl Position {
    /// Create a new Position representing the first character of any input.
    ///
    /// # Example
    /// ```
    /// # use turn_utils::position::Position;
    /// let position = Position::new();
    /// assert_eq!(position, Position { row: 1, col: 1, index: 0 });
    /// ```
    pub fn new() -> Self {
        Default::default()
    }

    /// Advance the position after reading a specific character.
    ///
    /// Advancing the position after a newline character
    /// will increment the row and set the column to 1.
    ///
    /// # Example
    /// ```
    /// # use turn_utils::position::Position;
    /// let mut position = Position { row: 55, col: 66, index: 77 };
    /// position.advance('€');
    /// assert_eq!(position, Position { row: 55, col: 67, index: 80 });
    /// position.advance('\n');
    /// assert_eq!(position, Position { row: 56, col: 1, index: 81 });
    /// ```
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
}
