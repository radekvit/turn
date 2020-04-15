use std::cmp::Ordering;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Matcher {
    Character(char),
    Category(CharacterCategory),
}

impl Matcher {
    pub fn is_matching(self, c: char) -> bool {
        match self {
            Matcher::Character(pattern) => c == pattern,
            Matcher::Category(category) => category.is_matching(c),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CharacterCategory {
    // ascii categories
    AsciiAlphanumeric,
    AsciiAlpha,
    AsciiBinaryDigit,
    AsciiDigit,
    AsciiHexDigit,
    AsciiLowercase,
    AsciiUppercase,
    AsciiWhitespace,
    // utf8 categories
    Utf8Alphanumeric,
    Utf8Alpha,
    Utf8Lowercase,
    Utf8Numeric,
    Utf8Uppercase,
    Utf8Whitespace,
}

impl CharacterCategory {
    pub fn is_matching(self, c: char) -> bool {
        use CharacterCategory::*;

        match self {
            AsciiAlphanumeric => c.is_ascii_alphanumeric(),
            AsciiAlpha => c.is_ascii_alphabetic(),
            AsciiBinaryDigit => c.is_digit(2),
            AsciiDigit => c.is_digit(10),
            AsciiHexDigit => c.is_digit(16),
            AsciiLowercase => c.is_ascii_lowercase(),
            AsciiUppercase => c.is_ascii_uppercase(),
            AsciiWhitespace => c.is_ascii_whitespace(),
            Utf8Alphanumeric => c.is_alphanumeric(),
            Utf8Alpha => c.is_alphabetic(),
            Utf8Lowercase => c.is_lowercase(),
            Utf8Numeric => c.is_numeric(),
            Utf8Uppercase => c.is_uppercase(),
            Utf8Whitespace => c.is_whitespace(),
        }
    }

    pub fn subset_ordering(self, other: CharacterCategory) -> Option<Ordering> {
        use CharacterCategory::*;

        match (self, other) {
            (x, y) if x == y => Some(Ordering::Equal),
            // AsciiAlphanumeric
            (AsciiAlphanumeric, Utf8Alphanumeric) => Some(Ordering::Less),
            (AsciiAlphanumeric, AsciiAlpha)
            | (AsciiAlphanumeric, AsciiBinaryDigit)
            | (AsciiAlphanumeric, AsciiDigit)
            | (AsciiAlphanumeric, AsciiHexDigit)
            | (AsciiAlphanumeric, AsciiLowercase)
            | (AsciiAlphanumeric, AsciiUppercase) => Some(Ordering::Greater),
            // AsciiAlpha
            (AsciiAlpha, AsciiAlphanumeric)
            | (AsciiAlpha, Utf8Alphanumeric)
            | (AsciiAlpha, Utf8Alpha) => Some(Ordering::Less),
            (AsciiAlpha, AsciiLowercase) | (AsciiAlpha, AsciiUppercase) => Some(Ordering::Greater),
            // AsciiBinaryDigit
            (AsciiBinaryDigit, AsciiAlphanumeric)
            | (AsciiBinaryDigit, AsciiDigit)
            | (AsciiBinaryDigit, AsciiHexDigit)
            | (AsciiBinaryDigit, Utf8Alphanumeric)
            | (AsciiBinaryDigit, Utf8Numeric) => Some(Ordering::Less),
            // AsciiDigit
            (AsciiDigit, AsciiAlphanumeric)
            | (AsciiDigit, AsciiHexDigit)
            | (AsciiDigit, Utf8Alphanumeric)
            | (AsciiDigit, Utf8Numeric) => Some(Ordering::Less),
            (AsciiDigit, AsciiBinaryDigit) => Some(Ordering::Greater),
            // AsciiHexDigit
            (AsciiHexDigit, AsciiAlphanumeric) | (AsciiHexDigit, Utf8Alphanumeric) => {
                Some(Ordering::Less)
            }
            (AsciiHexDigit, AsciiBinaryDigit) | (AsciiHexDigit, AsciiDigit) => {
                Some(Ordering::Greater)
            }
            // AsciiLowercase
            (AsciiLowercase, AsciiAlphanumeric)
            | (AsciiLowercase, AsciiAlpha)
            | (AsciiLowercase, Utf8Alphanumeric)
            | (AsciiLowercase, Utf8Alpha)
            | (AsciiLowercase, Utf8Lowercase) => Some(Ordering::Less),
            // AsciiUppercase
            (AsciiUppercase, AsciiAlphanumeric)
            | (AsciiUppercase, AsciiAlpha)
            | (AsciiUppercase, Utf8Alphanumeric)
            | (AsciiUppercase, Utf8Alpha)
            | (AsciiUppercase, Utf8Uppercase) => Some(Ordering::Less),
            // AsciiWhitespace
            (AsciiWhitespace, Utf8Whitespace) => Some(Ordering::Less),
            // Utf8Alphanumeric
            (Utf8Alphanumeric, AsciiAlphanumeric)
            | (Utf8Alphanumeric, AsciiAlpha)
            | (Utf8Alphanumeric, AsciiBinaryDigit)
            | (Utf8Alphanumeric, AsciiDigit)
            | (Utf8Alphanumeric, AsciiHexDigit)
            | (Utf8Alphanumeric, AsciiLowercase)
            | (Utf8Alphanumeric, AsciiUppercase)
            | (Utf8Alphanumeric, Utf8Alpha)
            | (Utf8Alphanumeric, Utf8Lowercase)
            | (Utf8Alphanumeric, Utf8Numeric)
            | (Utf8Alphanumeric, Utf8Uppercase) => Some(Ordering::Greater),
            // Utf8Alpha
            (Utf8Alpha, Utf8Alphanumeric) => Some(Ordering::Less),
            (Utf8Alpha, AsciiAlpha)
            | (Utf8Alpha, AsciiLowercase)
            | (Utf8Alpha, AsciiUppercase)
            | (Utf8Alpha, Utf8Lowercase)
            | (Utf8Alpha, Utf8Uppercase) => Some(Ordering::Greater),
            // Utf8Lowercase
            (Utf8Lowercase, Utf8Alphanumeric) | (Utf8Lowercase, Utf8Alpha) => Some(Ordering::Less),
            (Utf8Lowercase, AsciiLowercase) => Some(Ordering::Greater),
            // Utf8Numeric
            (Utf8Numeric, Utf8Alphanumeric) => Some(Ordering::Less),
            (Utf8Numeric, AsciiBinaryDigit) | (Utf8Numeric, AsciiDigit) => Some(Ordering::Greater),
            // Utf8Uppercase
            (Utf8Uppercase, Utf8Alphanumeric) | (Utf8Uppercase, Utf8Alpha) => Some(Ordering::Less),
            (Utf8Uppercase, AsciiUppercase) => Some(Ordering::Greater),
            // Utf8Whitespace
            (Utf8Whitespace, AsciiWhitespace) => Some(Ordering::Greater),
            _ => None,
        }
    }
    pub fn is_subset(self, other: CharacterCategory) -> bool {
        self.subset_ordering(other) == Some(Ordering::Less)
    }

    pub fn is_superset(self, other: CharacterCategory) -> bool {
        self.subset_ordering(other) == Some(Ordering::Greater)
    }
}

impl PartialOrd for CharacterCategory {
    fn partial_cmp(&self, other: &CharacterCategory) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CharacterCategory {
    fn cmp(&self, other: &CharacterCategory) -> Ordering {
        match self.subset_ordering(*other) {
            Some(ordering) => ordering,
            None => (*self as usize).cmp(&(*other as usize)),
        }
    }
}
