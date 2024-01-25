#![allow(dead_code, unused_variables)]

use core::fmt;
use eyre::{Ok, OptionExt};
use std::collections::HashMap;

#[derive(Copy, Clone, PartialEq)]
enum Token {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    NullVal,
    StringVal(usize, usize),
    NumVal(f64),
    BoolVal(bool),
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Token::*;
        let s = match self {
            LeftBrace => "{",
            RightBrace => "}",
            LeftBracket => "[",
            RightBracket => "]",
            Comma => ",",
            Colon => ":",
            NullVal => "null",
            StringVal(i, j) => return write!(f, "Token(str_range:[{}:{}])", i, j),
            BoolVal(b) => {
                if *b {
                    "true"
                } else {
                    "false"
                }
            }
            NumVal(n) => return write!(f, "Token('{}')", n),
        };
        write!(f, "Token('{}')", s)
    }
}

struct Lexer {
    whitespace: Vec<u8>,
    single_char_symbols: (Vec<u8>, Vec<Token>), // mapping
    multi_char_symbols: (Vec<Vec<u8>>, Vec<Token>), // mapping
    num_chars: Vec<u8>,
}

impl Lexer {
    fn new() -> Self {
        let whitespace: Vec<u8> = [' ', '\t', '\r', '\n']
            .into_iter()
            .map(|v| v as u8)
            .collect();
        use Token::*;
        let single_char_symbols = (
            ['{', '}', '[', ']', ',', ':']
                .into_iter()
                .map(|v| v as u8)
                .collect(),
            vec![
                LeftBrace,
                RightBrace,
                LeftBracket,
                RightBracket,
                Comma,
                Colon,
            ],
        );
        let multi_char_symbols = (
            ["null", "true", "fals"]
                .into_iter()
                .map(|s| s.as_bytes().to_vec())
                .collect(),
            vec![NullVal, BoolVal(true), BoolVal(false)],
        );
        let num_chars: Vec<u8> = "0123456789.e".chars().map(|v| v as u8).collect();
        Self {
            whitespace: whitespace,
            single_char_symbols: single_char_symbols,
            multi_char_symbols: multi_char_symbols,
            num_chars: num_chars,
        }
    }
    fn lex(&self, buf: &[u8]) -> eyre::Result<Vec<Token>> {
        let mut tokens = Vec::new();
        let quote_sym: u8 = '"' as u8;
        let minus_sym: u8 = '-' as u8;
        let mut i = 0;
        let buf_len = buf.len();
        'outer: loop {
            if i >= buf_len {
                break;
            }

            let c = buf[i];

            // ignore whitespace
            if self.whitespace.contains(&c) {
                i += 1;
                continue 'outer;
            }

            // handle single len symbols
            if let Some(si) = self.single_char_symbols.0.iter().position(|v| *v == c) {
                tokens.push(self.single_char_symbols.1[si]);
                i += 1;
                continue 'outer;
            }
            // handle strings
            if c == quote_sym {
                let mut j = i + 1;
                loop {
                    if j >= buf_len {
                        eyre::bail!("Missing end quote for string");
                    }
                    if buf[j] == quote_sym {
                        tokens.push(Token::StringVal(i + 1, j));
                        i = j + 1;
                        continue 'outer;
                    }
                    j += 1;
                }
            }

            // handle null and bools
            let end = std::cmp::min(i + 4, buf_len);
            let s = &buf[i..end];
            for j in 0..self.multi_char_symbols.0.len() {
                if s == &self.multi_char_symbols.0[j] {
                    let t = self.multi_char_symbols.1[j];
                    if t == Token::BoolVal(false) {
                        if end < buf_len && buf[end] == 'e' as u8 {
                            i = end + 1;
                        } else {
                            eyre::bail!("Incomplete false value");
                        }
                    } else {
                        i = end;
                    }
                    tokens.push(t);
                    continue 'outer;
                }
            }
            // handle numbers
            if c == minus_sym || (c >= 48 && c <= 57) {
                let mut j = i + 1;
                while j < buf_len {
                    if !self.num_chars.contains(&buf[j]) {
                        let num_as_buf = &buf[i..j];
                        let num: f64 = std::str::from_utf8(num_as_buf)?.parse()?;
                        tokens.push(Token::NumVal(num));
                        i = j;
                        continue 'outer;
                    }
                    j += 1;
                }
            }

            // error
            eyre::bail!(format!("Unexpected value: '{}'", c as char));
        }
        return Ok(tokens);
    }

    fn lex_multichar_symbol(&self, lexeme: &[u8]) -> Option<Token> {
        for i in 0..self.multi_char_symbols.0.len() {
            if lexeme == &self.multi_char_symbols.0[i] {
                return Some(self.multi_char_symbols.1[i]);
            }
        }
        return None;
    }
}

#[derive(Debug)]
enum JSONValue {
    Null,
    Bool(bool),
    Str(String),
    Num(f64),
    Array(Vec<JSONValue>),
    Dict(HashMap<String, JSONValue>),
}

fn parse_array<'a, 'b>(
    tokens: &'a [Token],
    buf: &'b [u8],
) -> eyre::Result<(JSONValue, &'a [Token])> {
    let mut entries = Vec::new();
    // handle empty array
    let t = *tokens.get(0).ok_or_eyre("Expected value")?;
    if t == Token::RightBracket {
        return Ok((JSONValue::Array(entries), &tokens[1..]));
    }
    // handle non-empty
    let mut tokens = tokens;
    loop {
        let (val, rest) = parse_value(tokens, buf)?;
        entries.push(val);
        tokens = rest;
        let token = *tokens.get(0).ok_or_eyre("Expected value")?;
        match token {
            Token::RightBracket => {
                return Ok((JSONValue::Array(entries), &tokens[1..]));
            }
            Token::Comma => {
                tokens = &tokens[1..];
                continue;
            }
            _ => eyre::bail!("Unexpected value for array"),
        }
    }
}

fn parse_dict_entry<'a, 'b>(
    tokens: &'a [Token],
    buf: &'b [u8],
) -> eyre::Result<((String, JSONValue), &'a [Token])> {
    if tokens.len() < 3 {
        eyre::bail!("Object entry incomplete")
    }
    // get key
    let key: String;
    if let Token::StringVal(i, j) = tokens[0] {
        key = String::from_utf8((&buf[i..j]).to_vec())?;
    } else {
        eyre::bail!("Expected string for key")
    }
    // handle colon
    if tokens[1] != Token::Colon {
        eyre::bail!("Expected colon")
    }
    // get val
    let (val, rest) = parse_value(&tokens[2..], buf)?;
    return Ok(((key, val), rest));
}

fn parse_dict<'a, 'b>(
    tokens: &'a [Token],
    buf: &'b [u8],
) -> eyre::Result<(JSONValue, &'a [Token])> {
    let mut entries = HashMap::new();
    // handle empty dict
    let t = *tokens.get(0).ok_or_eyre("Expected value")?;
    if t == Token::RightBracket {
        return Ok((JSONValue::Dict(entries), &tokens[1..]));
    }
    // handle rest
    let mut tokens = tokens;
    loop {
        let ((key, val), rest) = parse_dict_entry(tokens, buf)?;
        entries.insert(key, val);
        tokens = rest;
        let token = *tokens.get(0).ok_or_eyre("Expected value")?;
        match token {
            Token::RightBrace => {
                return Ok((JSONValue::Dict(entries), &tokens[1..]));
            }
            Token::Comma => {
                tokens = &tokens[1..];
                continue;
            }
            _ => eyre::bail!("Unexpected value for dict"),
        }
    }
}

fn parse_value<'a, 'b>(
    tokens: &'a [Token],
    buf: &'b [u8],
) -> eyre::Result<(JSONValue, &'a [Token])> {
    let t = tokens.get(0).ok_or_eyre("Expected value")?;
    let rest = &tokens[1..];
    let v = match t {
        Token::BoolVal(b) => JSONValue::Bool(*b),
        Token::NullVal => JSONValue::Null,
        Token::NumVal(n) => JSONValue::Num(*n),
        Token::StringVal(i, j) => {
            let s = String::from_utf8((&buf[*i..*j]).to_vec())?;
            JSONValue::Str(s)
        }
        Token::LeftBrace => return parse_dict(rest, buf),
        Token::LeftBracket => return parse_array(rest, buf),
        _ => {
            println!("bozo tok: {:?}", t);
            todo!("parse gen")
        }
    };
    Ok((v, rest))
}

fn parse(json: &[u8]) -> eyre::Result<JSONValue> {
    let lexer = Lexer::new();
    let tokens = lexer.lex(json)?;
    let (json_val, rest) = parse_value(&tokens, json)?;
    if rest.len() > 0 {
        eyre::bail!("Invalid JSON contains extra content")
    };
    return Ok(json_val);
}

fn main() {
    let json_val = "{\"foo\":[1,2,3]}".as_bytes();
    let parsed = parse(json_val).unwrap();
    println!("{:?}", parsed);
}
