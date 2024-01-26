# JSON Parser

A simple JSON parser based on [ECMA-404](https://www.json.org/json-en.html).

## JSON format

eBNF for JSON

```
<json> ::= (<primitive> | <object>)

<primitive> ::= <number> | <string> | <boolean> | "null"

<number> ::= "-"? <integral_part> <fractional_part>? <exponent>?
<integral_part> ::= ("0" | [1-9] [0-9]*)
<fractional_part> ::= "." [0-9]+
<exponent> ::= ("e" | "E" ) ("-" | "+")? [0-9]+

<string> ::= "\"" <char>* "\""
<char> ::= Any codepoint except " or \ or control characters.

<boolean> ::= ("true" | "false" )

<whitespace> ::= (" " | "\n" | "\t" | "\r")*

<object> ::= "{" (<whitespace> | <key> ":" <value> ("," <key> ":" <value>)* ) "}"
<key> ::= <whitespace> <string> <whitespace>
<value> ::= <whitespace> (<primitive> | <array> | <object>) <whitespace>

<array> ::= "[" (<value> ("," <value>)*)? "]"
```

## Usage

## TODO

- Check validity of strings (should not have '"' or '\' or control characters)
- Handle exponents in numbers correctly
