use crate::matchers::CharacterCategory;
use std::collections::HashMap;

pub fn builtin_categories() -> HashMap<&'static str, CharacterCategory> {
    let mut categories = HashMap::new();
    categories.insert("lower", CharacterCategory::Utf8Lowercase);
    categories.insert("upper", CharacterCategory::Utf8Uppercase);
    categories.insert("alpha", CharacterCategory::Utf8Alpha);
    categories.insert("alnum", CharacterCategory::Utf8Alphanumeric);
    categories.insert("digit", CharacterCategory::Utf8Numeric);
    categories.insert("whitespace", CharacterCategory::Utf8Whitespace);
    categories.insert("a-z", CharacterCategory::ASCIILowercase);
    categories.insert("A-Z", CharacterCategory::ASCIIUppercase);
    categories.insert("a-Z", CharacterCategory::ASCIIAlpha);
    categories.insert("0-Z", CharacterCategory::ASCIIAlphanumeric);
    categories.insert("0b", CharacterCategory::ASCIIBinaryDigit);
    categories.insert("0-9", CharacterCategory::ASCIIDigit);
    categories.insert("0x", CharacterCategory::ASCIIHexDigit);
    categories.insert(" ", CharacterCategory::ASCIIWhitespace);
    categories
}
