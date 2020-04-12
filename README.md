# turn
Parsing and translation framework for rust.

## Goals
* Procedural macro-based lexers (turn_lexer_derive)
* Procedural macro-based grammars
* ielr parsing

### turn_lexer_derive
* minimal DFA implementation
* Python-based regex syntax + unicode categories
* custom symbol skipping
* ? indentation syntax support
* split lexems into tokens (only literal strings) and regex
* allow custom regex for ignored

Regex syntax:
```
- `.`: any character
- `*`: repetition of preceding charecter or group 0-n times
- `+`: repetition of preceding charecter or group 1-n times
- `?`: repetition of preceding charecter or group 0-1 times
- `{m}`: repetition of preceding charecter or group m times
- `{m-n}`: repetition of preceding charecter or group m-n times
- `\\`: escape special characters
- `[]`: set of characters or categories
- `A|B`: will match either regex A or B
- `()`: groups regular expression
```

Categories:
```
utf-8:
- `<lower>`: lowercase utf-8 character
- `<upper>`: uppercase utf-8 character
- `<alpha>`: alphabetic utf-8 character
- `<alnum>`: alphanumeric utf-8 character
- `<digit>`: matches any utf-8 digit
- `<whitespace>`: matches utf-8 whitespace
ascii:
- `<a-z>`: lowercase ASCII letter
- `<A-Z>`: uppercase ASCII letter
- `<a-Z>`: alphabetic ASCII
- `<0-Z>`: alphanumeric ASCII
- `<0b>`: matches binary digits
- `<0-9>`: matches decimal digits
- `<0x>`: matches hex digits
- `< >`: `[ \t\n\r\f\v]`
- `\t`: tab
- `\n`: newline
- `\r`: carriage return
- `\f`: form feed
- `\v`: vertical tab
```

Examples:
- C identifier: `"[_<a-Z>][_<a-Z><0-9>]*"`
- CamelCaseIndentifier: `"<A-Z><a-Z>*"`
- utf-8 CamelCaseIndentifier: `"<upper><alpha>*"`
- JSON number: `"-?(([123456789]<0-9>*)|0)(.<0-9>+)?([eE][+-]?<0-9>+)?"`
- ASCII text: `"<a-Z>*"`