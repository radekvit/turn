# turn
Parsing and translation framework for rust.

## Goals
* Procedural macro-based lexers (turn_lexer_derive)
* Procedural macro-based grammars
* ielr parsing

### turn_lexer_derive
* minimal DFA implementation
* hierarchical token descriptions
    * character categories
    * sub-expressions
* Two character skipping modes: Regex (regex; defaults to skipping all whitespace; invokes errors)
/ Permissive (reads until any token starts matching)
* Two regex modes: Token (literal string) / Regex (regular expression)

#### Regex syntax
```
- `_`: any character
- `*`: repetition of preceding charecter or group 0-`n` times
- `+`: repetition of preceding charecter or group 1-`n` times
- `?`: repetition of preceding charecter or group 0-1 times
- `{m}`: repetition of preceding charecter or group `m` times
- `{m-n}`: repetition of preceding charecter or group `m`-`n` times
- `{-n}`: repetition of preceding charecter or group at most `n` times
- `{m-}`: repetition of preceding charecter or group at least `m` times
- `\`: escape special characters
- `[...]`: set of characters or categories
- `[!...]`: set of forbidden characters or categories
- `A|B`: will match either regex A or B
- `()`: groups regular expression
- `<...>`: reference sub-expression or category
```

#### Built-in character categories
```
utf-8
- `<lower>`: lowercase utf-8 character
- `<upper>`: uppercase utf-8 character
- `<alpha>`: alphabetic utf-8 character
- `<alnum>`: alphanumeric utf-8 character
- `<digit>`: matches any utf-8 digit
- `<whitespace>`: matches utf-8 whitespace
ASCII
- `<a-z>`: lowercase ASCII letter
- `<A-Z>`: uppercase ASCII letter
- `<a-Z>`: alphabetic ASCII
- `<0-Z>`: alphanumeric ASCII
- `<0b>`: matches binary digits
- `<0-9>`: matches decimal digits
- `<0x>`: matches hex digits
- `< >`: `[ \t\n\r\f\v]`
```

#### Example: JSON usage
```rust
use turn::Lexer;

#[derive(Lexer)]
#[lexer::skip(regex, r"< >*")]
enum JSONToken {
    #[lexer::regex(string_chars, r#"[!"\\]|\\<escaped_chars>"#)]
    #[lexer::regex(escaped_chars, r#"["\\/bfnrt]|u(<0x>){4}"#)]
    #[regex = r#""<string_chars>*""#]
    String,
    #[lexer::category(nonzero_digit, "123456789")]
    #[lexer::regex(integer, "-?<nonzero_digit><0-9>*|0")]
    #[lexer::regex(fraction, ".<0-9>*")]
    #[lexer::regex(exponent, "(E|e)(\+|-)?<0-9>+")]
    #[regex = r"<integer><fraction>?<exponent>?"]
    Number,
    #[token = "true"]
    True,
    #[token = "false"]
    False,
    #[token = "null"]
    Null,
    #[token = '{']
    LBrace,
    #[token = '}']
    RBrace,
    #[token = ',']
    Comma,
    #[token = ':']
    Colon,
    #[token = '[']
    LBracket,
    #[token = ']']
    RBracket,
}
```

#### Examples of regular expressions
- C identifier: `"[_<a-Z>][_<0-Z>]*"`
- CamelCaseIndentifier: `"<A-Z><a-Z>*"`
- utf-8 CamelCaseIndentifier: `"<upper><alpha>*"`
- JSON number: `"-?(([123456789]<0-9>*)|0)(.<0-9>+)?([eE][+-]?<0-9>+)?"`
- ASCII text: `"<a-Z>*"`
