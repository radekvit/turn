#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Matcher {
    /// Matches a literal character.
    SingleMatcher(SingleMatcher),
    /// Matches any character except those from the set
    NegatedSet(Vec<SingleMatcher>),
}

/// A character matcher for text input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SingleMatcher {
    /// Matches a literal character.
    Character(char),
    /// Matches a category of characters.
    Category(CharacterCategory),
}

/// A category of characters for character matching.
///
/// The ordering of these variants is significant for this enum's total ordering.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum CharacterCategory {
    /// The set of ascii lowercase letters: a-z
    ASCIILowercase,
    /// The set of ASCII uppercase letters: A-Z
    ASCIIUppercase,
    /// The set of ASCII alphabetic characters: a-z, A-Z
    ASCIIAlpha,
    /// The set of ASCII binary digits: 0, 1
    ASCIIBinaryDigit,
    /// The set of ASCII decimal digits: 0-9
    ASCIIDigit,
    /// The set of ASCII hexadecimal digits: 0-9, a-f, A-F
    ASCIIHexDigit,
    /// The set of ASCII alphabetic and numeric characters: 0-9, a-z, A-Z
    ASCIIAlphanumeric,
    /// The set of ASCII whitespace characters: space, horizontal tab, line feed, form feed,
    /// carriage return
    ASCIIWhitespace,
    /// The set of utf-8 lowercase letters
    Utf8Lowercase,
    /// The set of utf-8 uppercase letters
    Utf8Uppercase,
    /// The set of utf-8 alphabetic characters
    Utf8Alpha,
    /// The set of utf-8 numeric characters
    Utf8Numeric,
    /// The set of utf-8 alphabetic and numeric characters
    Utf8Alphanumeric,
    /// The set of utf-8 whitespace characters
    Utf8Whitespace,
    /// Matches any character
    Any,
}

impl SingleMatcher {
    /// A predicate determining whether a character matches with the matcher.
    pub fn is_matching(self, c: char) -> bool {
        match self {
            SingleMatcher::Character(pattern) => c == pattern,
            SingleMatcher::Category(category) => category.is_matching(c),
        }
    }
}

impl Matcher {
    /// A predicate determining whether a character matches with the matcher.
    pub fn is_matching(&self, c: char) -> bool {
        match self {
            Matcher::SingleMatcher(matcher) => matcher.is_matching(c),
            Matcher::NegatedSet(set) => set.iter().all(|x| !x.is_matching(c)),
        }
    }
}

impl CharacterCategory {
    /// A predicate returning true if the presented character belongs in the character category.
    pub fn is_matching(self, c: char) -> bool {
        use CharacterCategory::*;

        match self {
            ASCIILowercase => c.is_ascii_lowercase(),
            ASCIIUppercase => c.is_ascii_uppercase(),
            ASCIIAlpha => c.is_ascii_alphabetic(),
            ASCIIBinaryDigit => c == '0' || c == '1',
            ASCIIDigit => c.is_digit(10),
            ASCIIHexDigit => c.is_digit(16),
            ASCIIAlphanumeric => c.is_ascii_alphanumeric(),
            ASCIIWhitespace => c.is_ascii_whitespace(),
            Utf8Lowercase => c.is_lowercase(),
            Utf8Uppercase => c.is_uppercase(),
            Utf8Alpha => c.is_alphabetic(),
            Utf8Numeric => c.is_numeric(),
            Utf8Alphanumeric => c.is_alphanumeric(),
            Utf8Whitespace => c.is_whitespace(),
            Any => true,
        }
    }
}
