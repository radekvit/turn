use crate::matchers::{CharacterCategory, Matcher, SingleMatcher};
use std::cmp::Ordering;

/// A partial set ordering trait.
///
/// By implementing `set_ordering`, we get subset and superset predicates for free.
/// In some cases, we may want to provide a total ordering for storing sets in maps, but we may
/// still need to determine when they are subsets of each other.
pub trait SetOrdering<Rhs = Self> {
    /// Determines the ordering of the two sets.
    fn set_ordering(&self, other: &Rhs) -> Option<Ordering>;

    /// Returns true if the left hand side is a subset of the right hand side.
    fn is_subset(&self, other: &Rhs) -> bool {
        match self.set_ordering(other) {
            Some(Ordering::Less) | Some(Ordering::Equal) => true,
            _ => false,
        }
    }

    /// Returns true if the left hand side is a strict subset of the right hand side.
    fn is_strict_subset(&self, other: &Rhs) -> bool {
        self.set_ordering(other) == Some(Ordering::Less)
    }

    /// Returns true if the two sets are not comparable.
    fn is_uncomparable(&self, other: &Rhs) -> bool {
        self.set_ordering(other) == None
    }

    /// Returns true if the two sets are equal.
    fn is_equal(&self, other: &Rhs) -> bool {
        self.set_ordering(other) == Some(Ordering::Equal)
    }

    /// Returns true if the left hand side is a superset of the left hand side.
    fn is_superset(&self, other: &Rhs) -> bool {
        match self.set_ordering(other) {
            Some(Ordering::Greater) | Some(Ordering::Equal) => true,
            _ => false,
        }
    }

    /// Returns true if the left hand side is a strict superset of the left hand side.
    fn is_strict_superset(&self, other: &Rhs) -> bool {
        self.set_ordering(other) == Some(Ordering::Greater)
    }
}

impl SetOrdering for Matcher {
    fn set_ordering(&self, other: &Self) -> Option<Ordering> {
        use crate::matchers::SingleMatcher as SM;
        use Matcher::*;
        match (self, other) {
            (SingleMatcher(lhs), SingleMatcher(rhs)) => lhs.set_ordering(rhs),
            (SingleMatcher(single), NegatedSet(negated_set)) => {
                // if any subset of the category is excluded, the negated set is not comparable
                if negated_set.iter().any(|x| single.set_ordering(x).is_some()) {
                    None
                } else {
                    Some(Ordering::Less)
                }
            }
            (NegatedSet(negated_set), SingleMatcher(single)) => {
                // if any subset of the category is excluded, the negated set is not comparable
                if negated_set.iter().any(|x| single.set_ordering(x).is_some()) {
                    None
                } else {
                    Some(Ordering::Greater)
                }
            }
            (NegatedSet(lhs), NegatedSet(rhs)) => {
                // if lhs excludes equal sets or subsets only,
                // it excludes the same number or fewer characters
                let is_superset = |lhs: &Vec<SM>, rhs: &Vec<SM>| {
                    lhs.iter().all(|x| {
                        rhs.iter().any(|y| match x.set_ordering(y) {
                            Some(Ordering::Equal) | Some(Ordering::Less) => true,
                            _ => false,
                        })
                    })
                };
                match (is_superset(lhs, rhs), is_superset(rhs, lhs)) {
                    (true, true) => Some(Ordering::Equal),
                    (true, false) => Some(Ordering::Greater),
                    (false, true) => Some(Ordering::Less),
                    (false, false) => None,
                }
            }
        }
    }
}

impl SetOrdering for SingleMatcher {
    fn set_ordering(&self, other: &Self) -> Option<Ordering> {
        use SingleMatcher::*;
        match (*self, *other) {
            (Character(x), Character(y)) => {
                if x == y {
                    Some(Ordering::Equal)
                } else {
                    None
                }
            }
            (Character(character), Category(category)) => {
                if category.is_matching(character) {
                    Some(Ordering::Less)
                } else {
                    None
                }
            }
            (Category(category), Character(character)) => {
                if category.is_matching(character) {
                    Some(Ordering::Greater)
                } else {
                    None
                }
            }
            (Category(category1), Category(category2)) => category1.set_ordering(&category2),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_alphanumeric_set_ordering() {
        use std::cmp::Ordering::*;
        use CharacterCategory::*;

        assert_eq!(
            ASCIIAlphanumeric.set_ordering(&ASCIIAlphanumeric),
            Some(Equal)
        );
        assert_eq!(ASCIIAlphanumeric.set_ordering(&ASCIIAlpha), Some(Greater));
        assert_eq!(
            ASCIIAlphanumeric.set_ordering(&ASCIIBinaryDigit),
            Some(Greater)
        );
        assert_eq!(ASCIIAlphanumeric.set_ordering(&ASCIIDigit), Some(Greater));
        assert_eq!(
            ASCIIAlphanumeric.set_ordering(&ASCIIHexDigit),
            Some(Greater)
        );
        assert_eq!(
            ASCIIAlphanumeric.set_ordering(&ASCIILowercase),
            Some(Greater)
        );
        assert_eq!(
            ASCIIAlphanumeric.set_ordering(&ASCIIUppercase),
            Some(Greater)
        );
        assert_eq!(ASCIIAlphanumeric.set_ordering(&ASCIIWhitespace), None);
        assert_eq!(
            ASCIIAlphanumeric.set_ordering(&Utf8Alphanumeric),
            Some(Less)
        );
        assert_eq!(ASCIIAlphanumeric.set_ordering(&Utf8Alpha), None);
        assert_eq!(ASCIIAlphanumeric.set_ordering(&Utf8Lowercase), None);
        assert_eq!(ASCIIAlphanumeric.set_ordering(&Utf8Numeric), None);
        assert_eq!(ASCIIAlphanumeric.set_ordering(&Utf8Uppercase), None);
        assert_eq!(ASCIIAlphanumeric.set_ordering(&Utf8Whitespace), None);
    }

    #[test]
    fn ascii_alpha_set_ordering() {
        use std::cmp::Ordering::*;
        use CharacterCategory::*;

        assert_eq!(ASCIIAlpha.set_ordering(&ASCIIAlphanumeric), Some(Less));
        assert_eq!(ASCIIAlpha.set_ordering(&ASCIIAlpha), Some(Equal));
        assert_eq!(ASCIIAlpha.set_ordering(&ASCIIBinaryDigit), None);
        assert_eq!(ASCIIAlpha.set_ordering(&ASCIIDigit), None);
        assert_eq!(ASCIIAlpha.set_ordering(&ASCIIHexDigit), None);
        assert_eq!(ASCIIAlpha.set_ordering(&ASCIILowercase), Some(Greater));
        assert_eq!(ASCIIAlpha.set_ordering(&ASCIIUppercase), Some(Greater));
        assert_eq!(ASCIIAlpha.set_ordering(&ASCIIWhitespace), None);
        assert_eq!(ASCIIAlpha.set_ordering(&Utf8Alphanumeric), Some(Less));
        assert_eq!(ASCIIAlpha.set_ordering(&Utf8Alpha), Some(Less));
        assert_eq!(ASCIIAlpha.set_ordering(&Utf8Lowercase), None);
        assert_eq!(ASCIIAlpha.set_ordering(&Utf8Numeric), None);
        assert_eq!(ASCIIAlpha.set_ordering(&Utf8Uppercase), None);
        assert_eq!(ASCIIAlpha.set_ordering(&Utf8Whitespace), None);
    }

    #[test]
    fn ascii_binary_digit_set_ordering() {
        use std::cmp::Ordering::*;
        use CharacterCategory::*;

        assert_eq!(
            ASCIIBinaryDigit.set_ordering(&ASCIIAlphanumeric),
            Some(Less)
        );
        assert_eq!(ASCIIBinaryDigit.set_ordering(&ASCIIAlpha), None);
        assert_eq!(
            ASCIIBinaryDigit.set_ordering(&ASCIIBinaryDigit),
            Some(Equal)
        );
        assert_eq!(ASCIIBinaryDigit.set_ordering(&ASCIIDigit), Some(Less));
        assert_eq!(ASCIIBinaryDigit.set_ordering(&ASCIIHexDigit), Some(Less));
        assert_eq!(ASCIIBinaryDigit.set_ordering(&ASCIILowercase), None);
        assert_eq!(ASCIIBinaryDigit.set_ordering(&ASCIIUppercase), None);
        assert_eq!(ASCIIBinaryDigit.set_ordering(&ASCIIWhitespace), None);
        assert_eq!(ASCIIBinaryDigit.set_ordering(&Utf8Alphanumeric), Some(Less));
        assert_eq!(ASCIIBinaryDigit.set_ordering(&Utf8Alpha), None);
        assert_eq!(ASCIIBinaryDigit.set_ordering(&Utf8Lowercase), None);
        assert_eq!(ASCIIBinaryDigit.set_ordering(&Utf8Numeric), Some(Less));
        assert_eq!(ASCIIBinaryDigit.set_ordering(&Utf8Uppercase), None);
        assert_eq!(ASCIIBinaryDigit.set_ordering(&Utf8Whitespace), None);
    }

    #[test]
    fn ascii_digit_set_ordering() {
        use std::cmp::Ordering::*;
        use CharacterCategory::*;

        assert_eq!(ASCIIDigit.set_ordering(&ASCIIAlphanumeric), Some(Less));
        assert_eq!(ASCIIDigit.set_ordering(&ASCIIAlpha), None);
        assert_eq!(ASCIIDigit.set_ordering(&ASCIIBinaryDigit), Some(Greater));
        assert_eq!(ASCIIDigit.set_ordering(&ASCIIDigit), Some(Equal));
        assert_eq!(ASCIIDigit.set_ordering(&ASCIIHexDigit), Some(Less));
        assert_eq!(ASCIIDigit.set_ordering(&ASCIILowercase), None);
        assert_eq!(ASCIIDigit.set_ordering(&ASCIIUppercase), None);
        assert_eq!(ASCIIDigit.set_ordering(&ASCIIWhitespace), None);
        assert_eq!(ASCIIDigit.set_ordering(&Utf8Alphanumeric), Some(Less));
        assert_eq!(ASCIIDigit.set_ordering(&Utf8Alpha), None);
        assert_eq!(ASCIIDigit.set_ordering(&Utf8Lowercase), None);
        assert_eq!(ASCIIDigit.set_ordering(&Utf8Numeric), Some(Less));
        assert_eq!(ASCIIDigit.set_ordering(&Utf8Uppercase), None);
        assert_eq!(ASCIIDigit.set_ordering(&Utf8Whitespace), None);
    }

    #[test]
    fn ascii_hex_digit_set_ordering() {
        use std::cmp::Ordering::*;
        use CharacterCategory::*;

        assert_eq!(ASCIIHexDigit.set_ordering(&ASCIIAlphanumeric), Some(Less));
        assert_eq!(ASCIIHexDigit.set_ordering(&ASCIIAlpha), None);
        assert_eq!(ASCIIHexDigit.set_ordering(&ASCIIBinaryDigit), Some(Greater));
        assert_eq!(ASCIIHexDigit.set_ordering(&ASCIIDigit), Some(Greater));
        assert_eq!(ASCIIHexDigit.set_ordering(&ASCIIHexDigit), Some(Equal));
        assert_eq!(ASCIIHexDigit.set_ordering(&ASCIILowercase), None);
        assert_eq!(ASCIIHexDigit.set_ordering(&ASCIIUppercase), None);
        assert_eq!(ASCIIHexDigit.set_ordering(&ASCIIWhitespace), None);
        assert_eq!(ASCIIHexDigit.set_ordering(&Utf8Alphanumeric), Some(Less));
        assert_eq!(ASCIIHexDigit.set_ordering(&Utf8Alpha), None);
        assert_eq!(ASCIIHexDigit.set_ordering(&Utf8Lowercase), None);
        assert_eq!(ASCIIHexDigit.set_ordering(&Utf8Numeric), None);
        assert_eq!(ASCIIHexDigit.set_ordering(&Utf8Uppercase), None);
        assert_eq!(ASCIIHexDigit.set_ordering(&Utf8Whitespace), None);
    }

    #[test]
    fn ascii_lowercase_set_ordering() {
        use std::cmp::Ordering::*;
        use CharacterCategory::*;

        assert_eq!(ASCIILowercase.set_ordering(&ASCIIAlphanumeric), Some(Less));
        assert_eq!(ASCIILowercase.set_ordering(&ASCIIAlpha), Some(Less));
        assert_eq!(ASCIILowercase.set_ordering(&ASCIIBinaryDigit), None);
        assert_eq!(ASCIILowercase.set_ordering(&ASCIIDigit), None);
        assert_eq!(ASCIILowercase.set_ordering(&ASCIIHexDigit), None);
        assert_eq!(ASCIILowercase.set_ordering(&ASCIILowercase), Some(Equal));
        assert_eq!(ASCIILowercase.set_ordering(&ASCIIUppercase), None);
        assert_eq!(ASCIILowercase.set_ordering(&ASCIIWhitespace), None);
        assert_eq!(ASCIILowercase.set_ordering(&Utf8Alphanumeric), Some(Less));
        assert_eq!(ASCIILowercase.set_ordering(&Utf8Alpha), Some(Less));
        assert_eq!(ASCIILowercase.set_ordering(&Utf8Lowercase), Some(Less));
        assert_eq!(ASCIILowercase.set_ordering(&Utf8Numeric), None);
        assert_eq!(ASCIILowercase.set_ordering(&Utf8Uppercase), None);
        assert_eq!(ASCIILowercase.set_ordering(&Utf8Whitespace), None);
    }

    #[test]
    fn ascii_uppercase_set_ordering() {
        use std::cmp::Ordering::*;
        use CharacterCategory::*;

        assert_eq!(ASCIIUppercase.set_ordering(&ASCIIAlphanumeric), Some(Less));
        assert_eq!(ASCIIUppercase.set_ordering(&ASCIIAlpha), Some(Less));
        assert_eq!(ASCIIUppercase.set_ordering(&ASCIIBinaryDigit), None);
        assert_eq!(ASCIIUppercase.set_ordering(&ASCIIDigit), None);
        assert_eq!(ASCIIUppercase.set_ordering(&ASCIIHexDigit), None);
        assert_eq!(ASCIIUppercase.set_ordering(&ASCIILowercase), None);
        assert_eq!(ASCIIUppercase.set_ordering(&ASCIIUppercase), Some(Equal));
        assert_eq!(ASCIIUppercase.set_ordering(&ASCIIWhitespace), None);
        assert_eq!(ASCIIUppercase.set_ordering(&Utf8Alphanumeric), Some(Less));
        assert_eq!(ASCIIUppercase.set_ordering(&Utf8Alpha), Some(Less));
        assert_eq!(ASCIIUppercase.set_ordering(&Utf8Lowercase), None);
        assert_eq!(ASCIIUppercase.set_ordering(&Utf8Numeric), None);
        assert_eq!(ASCIIUppercase.set_ordering(&Utf8Uppercase), Some(Less));
        assert_eq!(ASCIIUppercase.set_ordering(&Utf8Whitespace), None);
    }

    #[test]
    fn ascii_whitespace_set_ordering() {
        use std::cmp::Ordering::*;
        use CharacterCategory::*;

        assert_eq!(ASCIIWhitespace.set_ordering(&ASCIIAlphanumeric), None);
        assert_eq!(ASCIIWhitespace.set_ordering(&ASCIIAlpha), None);
        assert_eq!(ASCIIWhitespace.set_ordering(&ASCIIBinaryDigit), None);
        assert_eq!(ASCIIWhitespace.set_ordering(&ASCIIDigit), None);
        assert_eq!(ASCIIWhitespace.set_ordering(&ASCIIHexDigit), None);
        assert_eq!(ASCIIWhitespace.set_ordering(&ASCIILowercase), None);
        assert_eq!(ASCIIWhitespace.set_ordering(&ASCIIUppercase), None);
        assert_eq!(ASCIIWhitespace.set_ordering(&ASCIIWhitespace), Some(Equal));
        assert_eq!(ASCIIWhitespace.set_ordering(&Utf8Alphanumeric), None);
        assert_eq!(ASCIIWhitespace.set_ordering(&Utf8Alpha), None);
        assert_eq!(ASCIIWhitespace.set_ordering(&Utf8Lowercase), None);
        assert_eq!(ASCIIWhitespace.set_ordering(&Utf8Numeric), None);
        assert_eq!(ASCIIWhitespace.set_ordering(&Utf8Uppercase), None);
        assert_eq!(ASCIIWhitespace.set_ordering(&Utf8Whitespace), Some(Less));
    }

    #[test]
    fn utf8_alphanumeric_set_ordering() {
        use std::cmp::Ordering::*;
        use CharacterCategory::*;

        assert_eq!(
            Utf8Alphanumeric.set_ordering(&ASCIIAlphanumeric),
            Some(Greater)
        );
        assert_eq!(Utf8Alphanumeric.set_ordering(&ASCIIAlpha), Some(Greater));
        assert_eq!(
            Utf8Alphanumeric.set_ordering(&ASCIIBinaryDigit),
            Some(Greater)
        );
        assert_eq!(Utf8Alphanumeric.set_ordering(&ASCIIDigit), Some(Greater));
        assert_eq!(Utf8Alphanumeric.set_ordering(&ASCIIHexDigit), Some(Greater));
        assert_eq!(
            Utf8Alphanumeric.set_ordering(&ASCIILowercase),
            Some(Greater)
        );
        assert_eq!(
            Utf8Alphanumeric.set_ordering(&ASCIIUppercase),
            Some(Greater)
        );
        assert_eq!(Utf8Alphanumeric.set_ordering(&ASCIIWhitespace), None);
        assert_eq!(
            Utf8Alphanumeric.set_ordering(&Utf8Alphanumeric),
            Some(Equal)
        );
        assert_eq!(Utf8Alphanumeric.set_ordering(&Utf8Alpha), Some(Greater));
        assert_eq!(Utf8Alphanumeric.set_ordering(&Utf8Lowercase), Some(Greater));
        assert_eq!(Utf8Alphanumeric.set_ordering(&Utf8Numeric), Some(Greater));
        assert_eq!(Utf8Alphanumeric.set_ordering(&Utf8Uppercase), Some(Greater));
        assert_eq!(Utf8Alphanumeric.set_ordering(&Utf8Whitespace), None);
    }

    #[test]
    fn utf8_alpha_set_ordering() {
        use std::cmp::Ordering::*;
        use CharacterCategory::*;

        assert_eq!(Utf8Alpha.set_ordering(&ASCIIAlphanumeric), None);
        assert_eq!(Utf8Alpha.set_ordering(&ASCIIAlpha), Some(Greater));
        assert_eq!(Utf8Alpha.set_ordering(&ASCIIBinaryDigit), None);
        assert_eq!(Utf8Alpha.set_ordering(&ASCIIDigit), None);
        assert_eq!(Utf8Alpha.set_ordering(&ASCIIHexDigit), None);
        assert_eq!(Utf8Alpha.set_ordering(&ASCIILowercase), Some(Greater));
        assert_eq!(Utf8Alpha.set_ordering(&ASCIIUppercase), Some(Greater));
        assert_eq!(Utf8Alpha.set_ordering(&ASCIIWhitespace), None);
        assert_eq!(Utf8Alpha.set_ordering(&Utf8Alphanumeric), Some(Less));
        assert_eq!(Utf8Alpha.set_ordering(&Utf8Alpha), Some(Equal));
        assert_eq!(Utf8Alpha.set_ordering(&Utf8Lowercase), Some(Greater));
        assert_eq!(Utf8Alpha.set_ordering(&Utf8Numeric), None);
        assert_eq!(Utf8Alpha.set_ordering(&Utf8Uppercase), Some(Greater));
        assert_eq!(Utf8Alpha.set_ordering(&Utf8Whitespace), None);
    }

    #[test]
    fn utf8_lowercase_set_ordering() {
        use std::cmp::Ordering::*;
        use CharacterCategory::*;

        assert_eq!(Utf8Lowercase.set_ordering(&ASCIIAlphanumeric), None);
        assert_eq!(Utf8Lowercase.set_ordering(&ASCIIAlpha), None);
        assert_eq!(Utf8Lowercase.set_ordering(&ASCIIBinaryDigit), None);
        assert_eq!(Utf8Lowercase.set_ordering(&ASCIIDigit), None);
        assert_eq!(Utf8Lowercase.set_ordering(&ASCIIHexDigit), None);
        assert_eq!(Utf8Lowercase.set_ordering(&ASCIILowercase), Some(Greater));
        assert_eq!(Utf8Lowercase.set_ordering(&ASCIIUppercase), None);
        assert_eq!(Utf8Lowercase.set_ordering(&ASCIIWhitespace), None);
        assert_eq!(Utf8Lowercase.set_ordering(&Utf8Alphanumeric), Some(Less));
        assert_eq!(Utf8Lowercase.set_ordering(&Utf8Alpha), Some(Less));
        assert_eq!(Utf8Lowercase.set_ordering(&Utf8Lowercase), Some(Equal));
        assert_eq!(Utf8Lowercase.set_ordering(&Utf8Numeric), None);
        assert_eq!(Utf8Lowercase.set_ordering(&Utf8Uppercase), None);
        assert_eq!(Utf8Lowercase.set_ordering(&Utf8Whitespace), None);
    }

    #[test]
    fn utf8_numeric_set_ordering() {
        use std::cmp::Ordering::*;
        use CharacterCategory::*;

        assert_eq!(Utf8Numeric.set_ordering(&ASCIIAlphanumeric), None);
        assert_eq!(Utf8Numeric.set_ordering(&ASCIIAlpha), None);
        assert_eq!(Utf8Numeric.set_ordering(&ASCIIBinaryDigit), Some(Greater));
        assert_eq!(Utf8Numeric.set_ordering(&ASCIIDigit), Some(Greater));
        assert_eq!(Utf8Numeric.set_ordering(&ASCIIHexDigit), None);
        assert_eq!(Utf8Numeric.set_ordering(&ASCIILowercase), None);
        assert_eq!(Utf8Numeric.set_ordering(&ASCIIUppercase), None);
        assert_eq!(Utf8Numeric.set_ordering(&ASCIIWhitespace), None);
        assert_eq!(Utf8Numeric.set_ordering(&Utf8Alphanumeric), Some(Less));
        assert_eq!(Utf8Numeric.set_ordering(&Utf8Alpha), None);
        assert_eq!(Utf8Numeric.set_ordering(&Utf8Lowercase), None);
        assert_eq!(Utf8Numeric.set_ordering(&Utf8Numeric), Some(Equal));
        assert_eq!(Utf8Numeric.set_ordering(&Utf8Uppercase), None);
        assert_eq!(Utf8Numeric.set_ordering(&Utf8Whitespace), None);
    }

    #[test]
    fn utf8_uppercase_set_ordering() {
        use std::cmp::Ordering::*;
        use CharacterCategory::*;

        assert_eq!(Utf8Uppercase.set_ordering(&ASCIIAlphanumeric), None);
        assert_eq!(Utf8Uppercase.set_ordering(&ASCIIAlpha), None);
        assert_eq!(Utf8Uppercase.set_ordering(&ASCIIBinaryDigit), None);
        assert_eq!(Utf8Uppercase.set_ordering(&ASCIIDigit), None);
        assert_eq!(Utf8Uppercase.set_ordering(&ASCIIHexDigit), None);
        assert_eq!(Utf8Uppercase.set_ordering(&ASCIILowercase), None);
        assert_eq!(Utf8Uppercase.set_ordering(&ASCIIUppercase), Some(Greater));
        assert_eq!(Utf8Uppercase.set_ordering(&ASCIIWhitespace), None);
        assert_eq!(Utf8Uppercase.set_ordering(&Utf8Alphanumeric), Some(Less));
        assert_eq!(Utf8Uppercase.set_ordering(&Utf8Alpha), Some(Less));
        assert_eq!(Utf8Uppercase.set_ordering(&Utf8Lowercase), None);
        assert_eq!(Utf8Uppercase.set_ordering(&Utf8Numeric), None);
        assert_eq!(Utf8Uppercase.set_ordering(&Utf8Uppercase), Some(Equal));
        assert_eq!(Utf8Uppercase.set_ordering(&Utf8Whitespace), None);
    }

    #[test]
    fn utf8_whitespace_set_ordering() {
        use std::cmp::Ordering::*;
        use CharacterCategory::*;

        assert_eq!(Utf8Whitespace.set_ordering(&ASCIIAlphanumeric), None);
        assert_eq!(Utf8Whitespace.set_ordering(&ASCIIAlpha), None);
        assert_eq!(Utf8Whitespace.set_ordering(&ASCIIBinaryDigit), None);
        assert_eq!(Utf8Whitespace.set_ordering(&ASCIIDigit), None);
        assert_eq!(Utf8Whitespace.set_ordering(&ASCIIHexDigit), None);
        assert_eq!(Utf8Whitespace.set_ordering(&ASCIILowercase), None);
        assert_eq!(Utf8Whitespace.set_ordering(&ASCIIUppercase), None);
        assert_eq!(Utf8Whitespace.set_ordering(&ASCIIWhitespace), Some(Greater));
        assert_eq!(Utf8Whitespace.set_ordering(&Utf8Alphanumeric), None);
        assert_eq!(Utf8Whitespace.set_ordering(&Utf8Alpha), None);
        assert_eq!(Utf8Whitespace.set_ordering(&Utf8Lowercase), None);
        assert_eq!(Utf8Whitespace.set_ordering(&Utf8Numeric), None);
        assert_eq!(Utf8Whitespace.set_ordering(&Utf8Uppercase), None);
        assert_eq!(Utf8Whitespace.set_ordering(&Utf8Whitespace), Some(Equal));
    }

    #[test]
    fn character_category_total_ordering() {
        use CharacterCategory::*;
        let expected_ordering = vec![
            ASCIILowercase,
            ASCIIUppercase,
            ASCIIAlpha,
            ASCIIBinaryDigit,
            ASCIIDigit,
            ASCIIHexDigit,
            ASCIIAlphanumeric,
            ASCIIWhitespace,
            Utf8Lowercase,
            Utf8Uppercase,
            Utf8Alpha,
            Utf8Numeric,
            Utf8Alphanumeric,
            Utf8Whitespace,
            Any,
        ];

        let mut reversed = vec![
            Any,
            Utf8Whitespace,
            Utf8Alphanumeric,
            Utf8Numeric,
            Utf8Alpha,
            Utf8Uppercase,
            Utf8Lowercase,
            ASCIIWhitespace,
            ASCIIAlphanumeric,
            ASCIIHexDigit,
            ASCIIDigit,
            ASCIIBinaryDigit,
            ASCIIAlpha,
            ASCIIUppercase,
            ASCIILowercase,
        ];
        reversed.sort_unstable();
        assert_eq!(reversed, expected_ordering);

        let mut shuffled = vec![
            Utf8Whitespace,
            ASCIILowercase,
            ASCIIUppercase,
            Utf8Alpha,
            ASCIIAlpha,
            Utf8Alphanumeric,
            ASCIIDigit,
            Utf8Uppercase,
            ASCIIHexDigit,
            Any,
            ASCIIAlphanumeric,
            ASCIIWhitespace,
            Utf8Lowercase,
            ASCIIBinaryDigit,
            Utf8Numeric,
        ];
        shuffled.sort_unstable();
        assert_eq!(shuffled, expected_ordering);
    }

    #[test]
    fn single_matcher_set_ordering() {
        use CharacterCategory::*;
        use Ordering::*;
        use SingleMatcher::*;
        // two categories
        assert_eq!(Character('x').set_ordering(&Character('x')), Some(Equal));
        assert_eq!(Character('a').set_ordering(&Character('b')), None);
        // character and category
        assert_eq!(
            Character('x').set_ordering(&Category(ASCIILowercase)),
            Some(Less)
        );
        assert_eq!(
            Character('x').set_ordering(&Category(ASCIIWhitespace)),
            None
        );
        // category and character
        assert_eq!(
            Category(Utf8Alphanumeric).set_ordering(&Character('α')),
            Some(Greater)
        );
        assert_eq!(Category(ASCIIHexDigit).set_ordering(&Character('\n')), None);
        // two categories
        assert_eq!(
            Category(ASCIIAlphanumeric).set_ordering(&Category(Utf8Alphanumeric)),
            Some(Less)
        );
        assert_eq!(
            Category(ASCIIWhitespace).set_ordering(&Category(ASCIIWhitespace)),
            Some(Equal)
        );
        assert_eq!(
            Category(ASCIIAlphanumeric).set_ordering(&Category(ASCIIDigit)),
            Some(Greater)
        );
        assert_eq!(
            Category(ASCIIAlphanumeric).set_ordering(&Category(Utf8Whitespace)),
            None
        );
    }

    #[test]
    fn matcher_set_ordering() {
        use super::SingleMatcher as SM;
        use CharacterCategory::*;
        use Matcher::*;
        use Ordering::*;

        // compare SingleMatcher with SingleMatcher
        assert_eq!(
            SingleMatcher(SM::Category(ASCIIHexDigit))
                .set_ordering(&SingleMatcher(SM::Category(Utf8Alphanumeric))),
            Some(Less)
        );
        assert_eq!(
            SingleMatcher(SM::Character('x')).set_ordering(&SingleMatcher(SM::Character('x'))),
            Some(Equal)
        );
        assert_eq!(
            SingleMatcher(SM::Category(ASCIIHexDigit))
                .set_ordering(&SingleMatcher(SM::Character('F'))),
            Some(Greater)
        );
        assert_eq!(
            SingleMatcher(SM::Character('x')).set_ordering(&SingleMatcher(SM::Character('a'))),
            None
        );
        // compare SingleMatcher with NegatedSet
        assert_eq!(
            SingleMatcher(SM::Category(Utf8Numeric)).set_ordering(&NegatedSet(vec![
                SM::Character('a'),
                SM::Category(Utf8Whitespace)
            ])),
            Some(Less)
        );
        assert_eq!(
            SingleMatcher(SM::Category(ASCIIDigit)).set_ordering(&NegatedSet(vec![
                SM::Character('a'),
                SM::Category(Utf8Whitespace),
                SM::Character('8'),
            ])),
            None
        );
        assert_eq!(
            SingleMatcher(SM::Category(ASCIIWhitespace)).set_ordering(&NegatedSet(vec![
                SM::Category(Utf8Whitespace),
                SM::Character(' '),
            ])),
            None
        );
        // compare NegatedSet with SingleMatcher
        assert_eq!(
            NegatedSet(vec![SM::Character('x')]).set_ordering(&SingleMatcher(SM::Character('x'))),
            None
        );
        assert_eq!(
            NegatedSet(vec![SM::Character('x')])
                .set_ordering(&SingleMatcher(SM::Category(Utf8Whitespace))),
            Some(Greater)
        );
        // compare NegatedSet with NegatedSet
        assert_eq!(
            NegatedSet(vec![SM::Character('x')])
                .set_ordering(&NegatedSet(vec![SM::Character('a')])),
            None
        );
        assert_eq!(
            NegatedSet(vec![
                SM::Category(ASCIIAlphanumeric),
                SM::Category(Utf8Whitespace)
            ])
            .set_ordering(&NegatedSet(vec![
                SM::Category(ASCIIWhitespace),
                SM::Category(Utf8Alphanumeric)
            ])),
            None
        );
        assert_eq!(
            NegatedSet(vec![SM::Category(Utf8Whitespace)])
                .set_ordering(&NegatedSet(vec![SM::Category(ASCIIWhitespace)])),
            Some(Less)
        );
        assert_eq!(
            NegatedSet(vec![SM::Category(ASCIIAlpha), SM::Character('α')]).set_ordering(
                &NegatedSet(vec![SM::Category(Utf8Alpha), SM::Category(ASCIILowercase)])
            ),
            Some(Greater)
        );
        assert_eq!(
            NegatedSet(vec![
                SM::Character('a'),
                SM::Character('b'),
                SM::Character('c'),
                SM::Category(Utf8Whitespace)
            ])
            .set_ordering(&NegatedSet(vec![
                SM::Character('c'),
                SM::Category(Utf8Whitespace),
                SM::Character('b'),
                SM::Character('c'),
                SM::Category(ASCIIWhitespace),
                SM::Character('a'),
            ])),
            Some(Equal)
        );
        assert_eq!(
            NegatedSet(vec![
                SM::Character('c'),
                SM::Category(Utf8Whitespace),
                SM::Character('b'),
                SM::Character('c'),
                SM::Category(ASCIIWhitespace),
                SM::Character('a'),
            ])
            .set_ordering(&NegatedSet(vec![
                SM::Character('a'),
                SM::Character('b'),
                SM::Character('c'),
                SM::Category(Utf8Whitespace)
            ])),
            Some(Equal)
        );
    }
}
