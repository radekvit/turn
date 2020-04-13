/// A struct containing source location information for a token.
///
/// # Examples
///
/// ```
/// let loc = turn_lexer::Location::new();
/// assert_eq!(loc.row, 1);
/// assert_eq!(loc.col, 1);
/// assert_eq!(loc.filename, Option::None);
///
/// let mut locf = turn_lexer::Location::from_file("stdin");
/// assert_eq!(locf.filename, Option::Some("stdin"));
/// locf.advance('a');
/// assert_eq!(locf.row, 1);
/// assert_eq!(locf.col, 2);
/// locf.advance('\n');
/// assert_eq!(locf.row, 2);
/// assert_eq!(locf.col, 1);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Location<'a> {
    /// The row number.
    pub row: u64,
    /// The column number.
    pub col: u64,
    /// The name of the originator file (optional).
    pub filename: Option<&'a str>,
}

impl<'a> Location<'a> {
    /// Creates a new location without a filename.
    ///
    /// # Examples
    /// ```
    /// let loc = turn_lexer::Location::new();
    /// assert_eq!(loc.row, 1);
    /// assert_eq!(loc.col, 1);
    /// assert_eq!(loc.filename, Option::None);
    /// ```
    pub fn new() -> Self {
        Location {
            row: 1,
            col: 1,
            filename: None,
        }
    }
    /// Creates a new location with a filename.
    ///
    /// # Examples
    /// ```
    /// let loc = turn_lexer::Location::from_file("filename");
    /// assert_eq!(loc.row, 1);
    /// assert_eq!(loc.col, 1);
    /// assert_eq!(loc.filename, Option::Some("filename"));
    /// ```
    pub fn from_file(filename: &'a str) -> Self {
        Location {
            row: 1,
            col: 1,
            filename: Some(filename),
        }
    }
    /// Advances the location according to the read character.
    ///
    /// # Examples
    /// ```
    /// let mut loc = turn_lexer::Location::new();
    /// for _ in 0..10 {
    ///     loc.advance('👍');
    /// }
    /// assert_eq!(loc.row, 1);
    /// assert_eq!(loc.col, 11);
    /// loc.advance('\n');
    /// assert_eq!(loc.row, 2);
    /// assert_eq!(loc.col, 1);
    /// ```
    pub fn advance(&mut self, c: char) {
        if c == '\n' {
            self.row = self.row + 1;
            self.col = 1;
        } else {
            self.col = self.col + 1;
        }
    }
}

/// A struct representing a token returned from a lexer.
///
/// # Examples
/// ```
/// use turn_lexer::{Location, Token};
/// let token = Token {symbol: 0u32, attribute: "", location: Location::new()};
/// assert_eq!(token.symbol, 0);
/// assert_eq!(token.attribute, "");
/// assert_eq!(token.location, Location::new());
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Token<'a, 'b, Symbol> {
    /// The symbol represented by this token.
    pub symbol: Symbol,
    /// The attribute of this token, represented by a slice of its input.
    pub attribute: &'a str,
    /// The location of this token.
    pub location: Location<'b>,
}
