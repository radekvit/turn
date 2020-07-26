/// A member of a set. Represents either a single character or a category of characters.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SetMember<'a> {
    Character(char),
    Category(&'a str),
}

/// A high-level representation of a hierarchical regular expression.
#[derive(Debug, PartialEq, Eq)]
pub enum HIR<'a> {
    /// Matches any character
    AnyChar,
    /// A sequence of simple characters
    Sequence(&'a str),
    /// A subexpression or character category
    SubRegex(&'a str),
    /// Repetition of a regular expression
    Repetition {
        regex: Box<HIR<'a>>,
        min: u16,
        max: Option<u16>,
    },
    /// Regular expression alternatives
    Alternation(Vec<HIR<'a>>),
    /// A set of characters or character categories
    Set(Vec<SetMember<'a>>),
    /// A set of all characters excluding specific characters or categories
    NegatedSet(Vec<SetMember<'a>>),
    /// Concatenation of regular expressions
    Concatenation(Vec<HIR<'a>>),
}
