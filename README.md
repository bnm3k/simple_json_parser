# JSON Parser

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
<char> ::= [a-z] | [A-Z] | [0-9]

<boolean> ::= ("true" | "false" )

<whitespace> ::= (" " | "\n" | "\t" | "\r")*

<object> ::= "{" (<whitespace> | <key> ":" <value> ("," <key> ":" <value>)* ) "}"
<key> ::= <whitespace> <string> <whitespace>
<value> ::= <whitespace> (<primitive> | <array> | <object>) <whitespace>

<array> ::= "[" (<value> ("," <value>)*)? "]"
```
