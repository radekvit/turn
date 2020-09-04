use crate::matchers::CharacterCategory;

/// A member of a set. Represents either a single character or a category of characters.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SetMember {
    Character(char),
    Category(CharacterCategory),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MIR<'a> {
    /// Matches any character
    Category(CharacterCategory),
    /// A sequence of simple characters
    Sequence(&'a str),
    /// Repetition of a regular expression
    Repetition {
        regex: Box<MIR<'a>>,
        min: u16,
        max: Option<u16>,
    },
    /// Regular expression alternatives
    Alternation(Vec<MIR<'a>>),
    /// A set of characters or character categories
    Set(Vec<SetMember>),
    /// A set of all characters excluding specific characters or categories
    NegatedSet(Vec<SetMember>),
    /// Concatenation of regular expressions
    Concatenation(Vec<MIR<'a>>),
}
