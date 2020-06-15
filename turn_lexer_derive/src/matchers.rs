/// A character matcher for text input.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Matcher {
    /// Matches a literal character.
    Character(char),
    /// Matches a category of characters.
    Category(CharacterCategory),
    /// Matches any single character
    Any,
}

impl Matcher {
    /// A predicate determining whether a character matches with the matcher.
    pub fn is_matching(self, c: char) -> bool {
        match self {
            Matcher::Character(pattern) => c == pattern,
            Matcher::Category(category) => category.is_matching(c),
            Matcher::Any => true,
        }
    }
}

/// A category of characters for character matching.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CharacterCategory {
    /// The set of ASCII alphabetic and numeric characters: 0-9, a-z, A-Z
    ASCIIAlphanumeric,
    /// The set of ASCII alphabetic characters: a-z, A-Z
    ASCIIAlpha,
    /// The set of ASCII binary digits: 0, 1
    ASCIIBinaryDigit,
    /// The set of ASCII decimal digits: 0-9
    ASCIIDigit,
    /// The set of ASCII hexadecimal digits: 0-9, a-f, A-F
    ASCIIHexDigit,
    /// The set of ascii lowercase letters: a-z
    ASCIILowercase,
    /// The set of ASCII uppercase letters: A-Z
    ASCIIUppercase,
    /// The set of ASCII whitespace characters: space, horizontal tab, line feed, form feed,
    /// carriage return
    ASCIIWhitespace,
    /// The set of utf-8 alphabetic and numeric characters
    Utf8Alphanumeric,
    /// The set of utf-8 alphabetic characters
    Utf8Alpha,
    /// The set of utf-8 lowercase letters
    Utf8Lowercase,
    /// The set of utf-8 numeric characters
    Utf8Numeric,
    /// The set of utf-8 uppercase letters
    Utf8Uppercase,
    /// The set of utf-8 whitespace characters
    Utf8Whitespace,
}

impl CharacterCategory {
    /// A predicate returning true if the presented character belongs in the character category.
    pub fn is_matching(self, c: char) -> bool {
        use CharacterCategory::*;

        match self {
            ASCIIAlphanumeric => c.is_ascii_alphanumeric(),
            ASCIIAlpha => c.is_ascii_alphabetic(),
            ASCIIBinaryDigit => c == '0' || c == '1',
            ASCIIDigit => c.is_digit(10),
            ASCIIHexDigit => c.is_digit(16),
            ASCIILowercase => c.is_ascii_lowercase(),
            ASCIIUppercase => c.is_ascii_uppercase(),
            ASCIIWhitespace => c.is_ascii_whitespace(),
            Utf8Alphanumeric => c.is_alphanumeric(),
            Utf8Alpha => c.is_alphabetic(),
            Utf8Lowercase => c.is_lowercase(),
            Utf8Numeric => c.is_numeric(),
            Utf8Uppercase => c.is_uppercase(),
            Utf8Whitespace => c.is_whitespace(),
        }
    }
}

impl SetOrdering for InputMatcher {
    fn set_ordering(&self, other: &Self) -> Option<Ordering> {
        use InputMatcher::*;
        match (self, other) {
            (Simple(x), Simple(y)) => x.set_ordering(y),
            (Simple(x), Excluding(matchers)) => unimplemented!(),
            (Excluding(matchers), Simple(x)) => unimplemented!(),
            (Excluding(x), Excluding(y)) => unimplemented!(),
        }
    }
}

impl SetOrdering for Matcher {
    /// Character matchers are only comparable if they match the same character.
    /// A character matcher is a subset of a category if the matched character belongs
    /// to the category.
    /// Finally, two categories are compared based on their character sets.
    fn set_ordering(&self, other: &Self) -> Option<Ordering> {
        use Matcher::*;

        match (self, other) {
            (Character(x), Character(y)) => {
                if x == y {
                    Some(Ordering::Equal)
                } else {
                    None
                }
            }
            (Character(x), Category(y)) => {
                if y.is_matching(*x) {
                    Some(Ordering::Less)
                } else {
                    None
                }
            }
            (Category(x), Character(y)) => {
                if x.is_matching(*y) {
                    Some(Ordering::Greater)
                } else {
                    None
                }
            }
            (Category(x), Category(y)) => x.set_ordering(y),
        }
    }
}

impl SetOrdering for CharacterCategory {
    /// The set ordering for character categories.
    fn set_ordering(&self, other: &Self) -> Option<Ordering> {
        use CharacterCategory::*;

        match (self, other) {
            (x, y) if x == y => Some(Ordering::Equal),
            // ASCIIAlphanumeric
            (ASCIIAlphanumeric, Utf8Alphanumeric) => Some(Ordering::Less),
            (ASCIIAlphanumeric, ASCIIAlpha)
            | (ASCIIAlphanumeric, ASCIIBinaryDigit)
            | (ASCIIAlphanumeric, ASCIIDigit)
            | (ASCIIAlphanumeric, ASCIIHexDigit)
            | (ASCIIAlphanumeric, ASCIILowercase)
            | (ASCIIAlphanumeric, ASCIIUppercase) => Some(Ordering::Greater),
            // ASCIIAlpha
            (ASCIIAlpha, ASCIIAlphanumeric)
            | (ASCIIAlpha, Utf8Alphanumeric)
            | (ASCIIAlpha, Utf8Alpha) => Some(Ordering::Less),
            (ASCIIAlpha, ASCIILowercase) | (ASCIIAlpha, ASCIIUppercase) => Some(Ordering::Greater),
            // ASCIIBinaryDigit
            (ASCIIBinaryDigit, ASCIIAlphanumeric)
            | (ASCIIBinaryDigit, ASCIIDigit)
            | (ASCIIBinaryDigit, ASCIIHexDigit)
            | (ASCIIBinaryDigit, Utf8Alphanumeric)
            | (ASCIIBinaryDigit, Utf8Numeric) => Some(Ordering::Less),
            // ASCIIDigit
            (ASCIIDigit, ASCIIAlphanumeric)
            | (ASCIIDigit, ASCIIHexDigit)
            | (ASCIIDigit, Utf8Alphanumeric)
            | (ASCIIDigit, Utf8Numeric) => Some(Ordering::Less),
            (ASCIIDigit, ASCIIBinaryDigit) => Some(Ordering::Greater),
            // ASCIIHexDigit
            (ASCIIHexDigit, ASCIIAlphanumeric) | (ASCIIHexDigit, Utf8Alphanumeric) => {
                Some(Ordering::Less)
            }
            (ASCIIHexDigit, ASCIIBinaryDigit) | (ASCIIHexDigit, ASCIIDigit) => {
                Some(Ordering::Greater)
            }
            // ASCIILowercase
            (ASCIILowercase, ASCIIAlphanumeric)
            | (ASCIILowercase, ASCIIAlpha)
            | (ASCIILowercase, Utf8Alphanumeric)
            | (ASCIILowercase, Utf8Alpha)
            | (ASCIILowercase, Utf8Lowercase) => Some(Ordering::Less),
            // ASCIIUppercase
            (ASCIIUppercase, ASCIIAlphanumeric)
            | (ASCIIUppercase, ASCIIAlpha)
            | (ASCIIUppercase, Utf8Alphanumeric)
            | (ASCIIUppercase, Utf8Alpha)
            | (ASCIIUppercase, Utf8Uppercase) => Some(Ordering::Less),
            // ASCIIWhitespace
            (ASCIIWhitespace, Utf8Whitespace) => Some(Ordering::Less),
            // Utf8Alphanumeric
            (Utf8Alphanumeric, ASCIIAlphanumeric)
            | (Utf8Alphanumeric, ASCIIAlpha)
            | (Utf8Alphanumeric, ASCIIBinaryDigit)
            | (Utf8Alphanumeric, ASCIIDigit)
            | (Utf8Alphanumeric, ASCIIHexDigit)
            | (Utf8Alphanumeric, ASCIILowercase)
            | (Utf8Alphanumeric, ASCIIUppercase)
            | (Utf8Alphanumeric, Utf8Alpha)
            | (Utf8Alphanumeric, Utf8Lowercase)
            | (Utf8Alphanumeric, Utf8Numeric)
            | (Utf8Alphanumeric, Utf8Uppercase) => Some(Ordering::Greater),
            // Utf8Alpha
            (Utf8Alpha, Utf8Alphanumeric) => Some(Ordering::Less),
            (Utf8Alpha, ASCIIAlpha)
            | (Utf8Alpha, ASCIILowercase)
            | (Utf8Alpha, ASCIIUppercase)
            | (Utf8Alpha, Utf8Lowercase)
            | (Utf8Alpha, Utf8Uppercase) => Some(Ordering::Greater),
            // Utf8Lowercase
            (Utf8Lowercase, Utf8Alphanumeric) | (Utf8Lowercase, Utf8Alpha) => Some(Ordering::Less),
            (Utf8Lowercase, ASCIILowercase) => Some(Ordering::Greater),
            // Utf8Numeric
            (Utf8Numeric, Utf8Alphanumeric) => Some(Ordering::Less),
            (Utf8Numeric, ASCIIBinaryDigit) | (Utf8Numeric, ASCIIDigit) => Some(Ordering::Greater),
            // Utf8Uppercase
            (Utf8Uppercase, Utf8Alphanumeric) | (Utf8Uppercase, Utf8Alpha) => Some(Ordering::Less),
            (Utf8Uppercase, ASCIIUppercase) => Some(Ordering::Greater),
            // Utf8Whitespace
            (Utf8Whitespace, ASCIIWhitespace) => Some(Ordering::Greater),
            _ => None,
        }
    }
}

impl PartialOrd for CharacterCategory {
    /// Th
    fn partial_cmp(&self, other: &CharacterCategory) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CharacterCategory {
    /// We order the categories in an ascending order based on their set ordering.
    /// We provide full ordering by ordering uncomparable categories by their usize representation.
    fn cmp(&self, other: &CharacterCategory) -> Ordering {
        match self.set_ordering(other) {
            Some(ordering) => ordering,
            None => (*self as usize).cmp(&(*other as usize)),
        }
    }
}
