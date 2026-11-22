//! # picojson-safe-rust
//!
//! A 100% Safe Rust (`#![forbid(unsafe_code)]`) drop-in translation of [picojson](https://github.com/kazuho/picojson).
//! Zero external dependencies, zero unsafe blocks, zero manual pointer/heap allocations.
//!
//! ## License & Attribution
//! - Original C++ picojson Copyright (c) 2009-2010 Cybozu Labs, Inc., Copyright (c) 2011-2014 Kazuho Oku
//! - Safe Rust Translation & Modernization Copyright (c) 2026 CobolIQ / DecodeIQ Authors (<https://coboliq.com>, <https://github.com/coboliqcom>)
//! - Licensed under the 2-Clause BSD License. See `LICENSE` for details.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::BTreeMap;
use std::fmt;

/// Array type in picojson (vector of JSON values).
pub type Array = Vec<Value>;

/// Object type in picojson (ordered map of string keys to JSON values).
pub type Object = BTreeMap<String, Value>;

/// Represents a JSON value according to RFC 8259, with optional 64-bit integer tracking.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Value {
    /// JSON null
    #[default]
    Null,
    /// JSON boolean (true / false)
    Bool(bool),
    /// JSON 64-bit floating point number
    Number(f64),
    /// JSON 64-bit integer
    Int64(i64),
    /// JSON string
    String(String),
    /// JSON array
    Array(Array),
    /// JSON object
    Object(Object),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.serialize())
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        assert!(!n.is_nan(), "JSON numbers cannot be NaN");
        assert!(!n.is_infinite(), "JSON numbers cannot be infinite");
        Value::Number(n)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Int64(i)
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::Int64(i as i64)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<Array> for Value {
    fn from(a: Array) -> Self {
        Value::Array(a)
    }
}

impl From<Object> for Value {
    fn from(o: Object) -> Self {
        Value::Object(o)
    }
}

impl Value {
    /// Returns true if value is null.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Returns true if value is a boolean.
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    /// Returns true if value is a number (either f64 or int64).
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_) | Value::Int64(_))
    }

    /// Returns true if value is a 64-bit integer.
    pub fn is_int64(&self) -> bool {
        matches!(self, Value::Int64(_))
    }

    /// Returns true if value is a string.
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// Returns true if value is an array.
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    /// Returns true if value is an object.
    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    /// Returns boolean value if type matches.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns f64 value if type is Number or Int64.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            Value::Int64(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Returns i64 value if type is Int64.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int64(i) => Some(*i),
            _ => None,
        }
    }

    /// Returns string slice if type matches.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Returns array reference if type matches.
    pub fn as_array(&self) -> Option<&Array> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Returns mutable array reference if type matches.
    pub fn as_array_mut(&mut self) -> Option<&mut Array> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Returns object reference if type matches.
    pub fn as_object(&self) -> Option<&Object> {
        match self {
            Value::Object(o) => Some(o),
            _ => None,
        }
    }

    /// Returns mutable object reference if type matches.
    pub fn as_object_mut(&mut self) -> Option<&mut Object> {
        match self {
            Value::Object(o) => Some(o),
            _ => None,
        }
    }

    /// Checks whether an object contains the given key, or an array contains the given index.
    pub fn contains_key(&self, key: &str) -> bool {
        match self {
            Value::Object(o) => o.contains_key(key),
            _ => false,
        }
    }

    /// Checks whether an array contains the given index.
    pub fn contains_index(&self, idx: usize) -> bool {
        match self {
            Value::Array(a) => idx < a.len(),
            _ => false,
        }
    }

    /// Retrieves an element from an object by key.
    pub fn get_key(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(o) => o.get(key),
            _ => None,
        }
    }

    /// Retrieves an element from an array by index.
    pub fn get_index(&self, idx: usize) -> Option<&Value> {
        match self {
            Value::Array(a) => a.get(idx),
            _ => None,
        }
    }

    /// Evaluates value as a boolean according to picojson truthiness rules:
    /// - null -> false
    /// - bool -> b
    /// - number -> n != 0.0
    /// - int64 -> i != 0
    /// - string -> !s.is_empty()
    /// - array -> !a.is_empty()
    /// - object -> !o.is_empty()
    pub fn evaluate_as_boolean(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::Int64(i) => *i != 0,
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Object(o) => !o.is_empty(),
        }
    }

    /// Serializes value into a compact JSON string without formatting whitespace.
    pub fn serialize(&self) -> String {
        let mut out = String::new();
        self.serialize_internal(&mut out, None, 0);
        out
    }

    /// Serializes value into a pretty-printed JSON string with 2-space indentation.
    pub fn serialize_pretty(&self) -> String {
        let mut out = String::new();
        self.serialize_internal(&mut out, Some(2), 0);
        out.push('\n');
        out
    }

    fn serialize_internal(&self, out: &mut String, indent: Option<usize>, depth: usize) {
        match self {
            Value::Null => out.push_str("null"),
            Value::Bool(true) => out.push_str("true"),
            Value::Bool(false) => out.push_str("false"),
            Value::Int64(i) => out.push_str(&i.to_string()),
            Value::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 1e16 && !n.is_sign_negative() {
                    // Picojson prints integer floats like 42.0 as 42
                    out.push_str(&format!("{n:.0}"));
                } else if *n == 0.0 && n.is_sign_negative() {
                    out.push_str("-0");
                } else {
                    let s = n.to_string();
                    out.push_str(&s);
                }
            }
            Value::String(s) => {
                out.push('"');
                for ch in s.chars() {
                    match ch {
                        '"' => out.push_str("\\\""),
                        '\\' => out.push_str("\\\\"),
                        '\x08' => out.push_str("\\b"),
                        '\x0C' => out.push_str("\\f"),
                        '\n' => out.push_str("\\n"),
                        '\r' => out.push_str("\\r"),
                        '\t' => out.push_str("\\t"),
                        c if (c as u32) < 0x20 => {
                            out.push_str(&format!("\\u{:04x}", c as u32));
                        }
                        c => out.push(c),
                    }
                }
                out.push('"');
            }
            Value::Array(arr) => {
                if arr.is_empty() {
                    out.push_str("[]");
                    return;
                }
                out.push('[');
                if let Some(step) = indent {
                    out.push('\n');
                    for (i, item) in arr.iter().enumerate() {
                        if i > 0 {
                            out.push_str(",\n");
                        }
                        out.push_str(&" ".repeat((depth + 1) * step));
                        item.serialize_internal(out, Some(step), depth + 1);
                    }
                    out.push('\n');
                    out.push_str(&" ".repeat(depth * step));
                } else {
                    for (i, item) in arr.iter().enumerate() {
                        if i > 0 {
                            out.push(',');
                        }
                        item.serialize_internal(out, None, depth);
                    }
                }
                out.push(']');
            }
            Value::Object(obj) => {
                if obj.is_empty() {
                    out.push_str("{}");
                    return;
                }
                out.push('{');
                if let Some(step) = indent {
                    out.push('\n');
                    for (i, (k, v)) in obj.iter().enumerate() {
                        if i > 0 {
                            out.push_str(",\n");
                        }
                        out.push_str(&" ".repeat((depth + 1) * step));
                        Value::String(k.clone()).serialize_internal(out, None, 0);
                        out.push_str(": ");
                        v.serialize_internal(out, Some(step), depth + 1);
                    }
                    out.push('\n');
                    out.push_str(&" ".repeat(depth * step));
                } else {
                    for (i, (k, v)) in obj.iter().enumerate() {
                        if i > 0 {
                            out.push(',');
                        }
                        Value::String(k.clone()).serialize_internal(out, None, 0);
                        out.push(':');
                        v.serialize_internal(out, None, depth);
                    }
                }
                out.push('}');
            }
        }
    }
}

/// Parses a JSON string into a `Value` with default depth limit of 512.
pub fn parse(input: &str) -> Result<Value, String> {
    parse_with_depth(input, 512)
}

/// Parses a JSON string into a `Value` with a specified maximum nesting depth.
pub fn parse_with_depth(input: &str, max_depth: usize) -> Result<Value, String> {
    let (val, rest) = parse_with_rest(input, max_depth)?;
    let remaining = rest.trim_start();
    if !remaining.is_empty() {
        return Err(format!("trailing characters after JSON at: '{remaining}'"));
    }
    Ok(val)
}

/// Parses a JSON string, returning the parsed `Value` and the remaining unconsumed slice.
pub fn parse_with_rest(input: &str, max_depth: usize) -> Result<(Value, &str), String> {
    let mut parser = Parser::new(input, max_depth);
    let val = parser.parse_value()?;
    Ok((val, &input[parser.pos..]))
}

struct Parser<'a> {
    input: &'a str,
    bytes: &'a [u8],
    pos: usize,
    depth: usize,
    max_depth: usize,
    line: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str, max_depth: usize) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            pos: 0,
            depth: 0,
            max_depth,
            line: 1,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b' ' | b'\t' | b'\r' => self.pos += 1,
                b'\n' => {
                    self.pos += 1;
                    self.line += 1;
                }
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn parse_value(&mut self) -> Result<Value, String> {
        self.skip_whitespace();
        let ch = match self.peek() {
            Some(c) => c,
            None => return Err(format!("syntax error at line {} near: empty input", self.line)),
        };

        match ch {
            b'n' => self.parse_null(),
            b't' | b'f' => self.parse_bool(),
            b'"' => self.parse_string().map(Value::String),
            b'[' => self.parse_array(),
            b'{' => self.parse_object(),
            b'-' | b'0'..=b'9' => self.parse_number(),
            _ => Err(format!(
                "syntax error at line {} near: {}",
                self.line,
                &self.input[self.pos..std::cmp::min(self.pos + 10, self.input.len())]
            )),
        }
    }

    fn parse_null(&mut self) -> Result<Value, String> {
        if self.bytes[self.pos..].starts_with(b"null") {
            self.pos += 4;
            Ok(Value::Null)
        } else {
            Err(format!("syntax error at line {} near: null expected", self.line))
        }
    }

    fn parse_bool(&mut self) -> Result<Value, String> {
        if self.bytes[self.pos..].starts_with(b"true") {
            self.pos += 4;
            Ok(Value::Bool(true))
        } else if self.bytes[self.pos..].starts_with(b"false") {
            self.pos += 5;
            Ok(Value::Bool(false))
        } else {
            let snippet = &self.input[self.pos..std::cmp::min(self.pos + 5, self.input.len())];
            Err(format!("syntax error at line {} near: {}", self.line, snippet))
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.pos += 1; // skip opening quote
        let mut s = String::new();

        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b == b'"' {
                self.pos += 1;
                return Ok(s);
            }
            if b == b'\n' || b == b'\r' {
                return Err(format!("syntax error at line {} near: unescaped newline in string", self.line));
            }
            if b == b'\\' {
                self.pos += 1;
                if self.pos >= self.bytes.len() {
                    return Err("unexpected EOF in escape sequence".to_string());
                }
                match self.bytes[self.pos] {
                    b'"' => {
                        s.push('"');
                        self.pos += 1;
                    }
                    b'\\' => {
                        s.push('\\');
                        self.pos += 1;
                    }
                    b'/' => {
                        s.push('/');
                        self.pos += 1;
                    }
                    b'b' => {
                        s.push('\x08');
                        self.pos += 1;
                    }
                    b'f' => {
                        s.push('\x0C');
                        self.pos += 1;
                    }
                    b'n' => {
                        s.push('\n');
                        self.pos += 1;
                    }
                    b'r' => {
                        s.push('\r');
                        self.pos += 1;
                    }
                    b't' => {
                        s.push('\t');
                        self.pos += 1;
                    }
                    b'u' => {
                        self.pos += 1;
                        let cp = self.parse_hex4()?;
                        if (0xD800..=0xDBFF).contains(&cp) {
                            // High surrogate, look for \uDC00..\uDFFF
                            if self.pos + 2 <= self.bytes.len()
                                && self.bytes[self.pos] == b'\\'
                                && self.bytes[self.pos + 1] == b'u'
                            {
                                self.pos += 2;
                                let cp2 = self.parse_hex4()?;
                                if (0xDC00..=0xDFFF).contains(&cp2) {
                                    let combined = 0x10000 + (((cp - 0xD800) << 10) | (cp2 - 0xDC00));
                                    if let Some(c) = char::from_u32(combined) {
                                        s.push(c);
                                        continue;
                                    }
                                }
                            }
                            return Err("invalid unicode surrogate pair".to_string());
                        } else if let Some(c) = char::from_u32(cp) {
                            s.push(c);
                        } else {
                            return Err("invalid unicode codepoint".to_string());
                        }
                    }
                    _ => return Err(format!("invalid escape sequence at line {}", self.line)),
                }
            } else {
                // Decode utf-8 character
                let start = self.pos;
                let ch_len = if b < 0x80 { 1 }
                else if (b & 0xE0) == 0xC0 { 2 }
                else if (b & 0xF0) == 0xE0 { 3 }
                else if (b & 0xF8) == 0xF0 { 4 }
                else { return Err("invalid utf-8 sequence in string".to_string()); };

                if start + ch_len > self.bytes.len() {
                    return Err("truncated utf-8 sequence".to_string());
                }
                let slice = std::str::from_utf8(&self.bytes[start..start + ch_len])
                    .map_err(|e| e.to_string())?;
                s.push_str(slice);
                self.pos += ch_len;
            }
        }
        Err("unexpected EOF inside string".to_string())
    }

    fn parse_hex4(&mut self) -> Result<u32, String> {
        if self.pos + 4 > self.bytes.len() {
            return Err("truncated \\u escape sequence".to_string());
        }
        let hex_str = std::str::from_utf8(&self.bytes[self.pos..self.pos + 4])
            .map_err(|_| "invalid hex in \\u escape".to_string())?;
        let val = u32::from_str_radix(hex_str, 16)
            .map_err(|_| "invalid hex characters in \\u escape".to_string())?;
        self.pos += 4;
        Ok(val)
    }

    fn parse_number(&mut self) -> Result<Value, String> {
        let start = self.pos;
        if self.bytes[self.pos] == b'-' {
            self.pos += 1;
        }

        let mut is_float = false;
        let num_start = self.pos;
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
            self.pos += 1;
        }

        if self.pos == num_start {
            return Err(format!("syntax error at line {} near: invalid number", self.line));
        }

        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'.' {
            is_float = true;
            self.pos += 1;
            let frac_start = self.pos;
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
            if self.pos == frac_start {
                return Err(format!("syntax error at line {} near: missing fraction digits", self.line));
            }
        }

        if self.pos < self.bytes.len() && (self.bytes[self.pos] == b'e' || self.bytes[self.pos] == b'E') {
            is_float = true;
            self.pos += 1;
            if self.pos < self.bytes.len() && (self.bytes[self.pos] == b'+' || self.bytes[self.pos] == b'-') {
                self.pos += 1;
            }
            let exp_start = self.pos;
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
            if self.pos == exp_start {
                return Err(format!("syntax error at line {} near: missing exponent digits", self.line));
            }
        }

        let num_str = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|e| e.to_string())?;

        if is_float {
            let f: f64 = num_str.parse().map_err(|e: std::num::ParseFloatError| e.to_string())?;
            if f.is_nan() || f.is_infinite() {
                return Err("number overflowed to infinity".to_string());
            }
            Ok(Value::Number(f))
        } else if let Ok(i) = num_str.parse::<i64>() {
            Ok(Value::Int64(i))
        } else {
            let f: f64 = num_str.parse().map_err(|e: std::num::ParseFloatError| e.to_string())?;
            if f.is_nan() || f.is_infinite() {
                return Err("number overflowed to infinity".to_string());
            }
            Ok(Value::Number(f))
        }
    }

    fn parse_array(&mut self) -> Result<Value, String> {
        if self.depth >= self.max_depth {
            return Err("maximum nesting depth exceeded".to_string());
        }
        self.depth += 1;
        self.pos += 1; // skip '['

        let mut arr = Vec::new();
        loop {
            self.skip_whitespace();
            if self.pos >= self.bytes.len() {
                return Err("unexpected EOF inside array".to_string());
            }
            if self.bytes[self.pos] == b']' {
                self.pos += 1;
                self.depth -= 1;
                return Ok(Value::Array(arr));
            }
            let item = self.parse_value()?;
            arr.push(item);

            self.skip_whitespace();
            if self.pos >= self.bytes.len() {
                return Err("unexpected EOF after array item".to_string());
            }
            if self.bytes[self.pos] == b',' {
                self.pos += 1;
            } else if self.bytes[self.pos] == b']' {
                self.pos += 1;
                self.depth -= 1;
                return Ok(Value::Array(arr));
            } else {
                return Err(format!(
                    "syntax error at line {} near: {}",
                    self.line,
                    &self.input[self.pos..std::cmp::min(self.pos + 10, self.input.len())]
                ));
            }
        }
    }

    fn parse_object(&mut self) -> Result<Value, String> {
        if self.depth >= self.max_depth {
            return Err("maximum nesting depth exceeded".to_string());
        }
        self.depth += 1;
        self.pos += 1; // skip '{'

        let mut obj = BTreeMap::new();
        loop {
            self.skip_whitespace();
            if self.pos >= self.bytes.len() {
                return Err("unexpected EOF inside object".to_string());
            }
            if self.bytes[self.pos] == b'}' {
                self.pos += 1;
                self.depth -= 1;
                return Ok(Value::Object(obj));
            }
            if self.bytes[self.pos] != b'"' {
                return Err(format!(
                    "syntax error at line {} near: {}",
                    self.line,
                    &self.input[self.pos..std::cmp::min(self.pos + 10, self.input.len())]
                ));
            }
            let key = self.parse_string()?;
            self.skip_whitespace();
            if self.pos >= self.bytes.len() || self.bytes[self.pos] != b':' {
                return Err(format!("syntax error at line {} near: ':' expected", self.line));
            }
            self.pos += 1; // skip ':'

            let val = self.parse_value()?;
            obj.insert(key, val);

            self.skip_whitespace();
            if self.pos >= self.bytes.len() {
                return Err("unexpected EOF after object value".to_string());
            }
            if self.bytes[self.pos] == b',' {
                self.pos += 1;
            } else if self.bytes[self.pos] == b'}' {
                self.pos += 1;
                self.depth -= 1;
                return Ok(Value::Object(obj));
            } else {
                return Err(format!(
                    "syntax error at line {} near: {}",
                    self.line,
                    &self.input[self.pos..std::cmp::min(self.pos + 10, self.input.len())]
                ));
            }
        }
    }
}
