use crate::util::{decode_base64, encode_base64, parse_string_literal};
pub(crate) use rusqlite::types::Value;

#[cfg(test)]
mod tests;

pub(crate) fn value_to_string(v: &Value) -> String {
    match v {
        Value::Null => String::from("null"),
        Value::Integer(n) => n.to_string(),
        Value::Real(n) => {
            let mut s = n.to_string();

            if !s.contains(".") {
                s = format!("{s}.0");
            }

            s
        },
        // I don't use `format!("{s:?}")` because...
        // 1. There are so many edge cases with string escapes and
        //    `parse_string_literal` cannot handle them properly.
        //    I hope there's `eval(s)` in rust, but there isn't.
        // 2. We don't have to handle all the edge cases. All we
        //    need is a format that `parse_string_literal` can parse.
        // 3. If we ignore the edge cases, this implementation is faster
        //    than `format!("{s:?}")`.
        Value::Text(s) => {
            let mut chars = Vec::with_capacity(s.len() + 2);
            chars.push('"');

            for ch in s.chars() {
                match ch {
                    '\n' | '\r' | '\t' | '\0' => {
                        chars.push('\\');

                        match ch {
                            '\n' => {
                                chars.push('n');
                            },
                            '\r' => {
                                chars.push('r');
                            },
                            '\t' => {
                                chars.push('t');
                            },
                            '\0' => {
                                chars.push('0');
                            },
                            _ => unreachable!(),
                        }
                    },
                    '"' | '\\' => {
                        chars.push('\\');
                        chars.push(ch);
                    },
                    _ => {
                        chars.push(ch);
                    },
                }
            }

            chars.push('"');
            chars.into_iter().collect()
        },
        Value::Blob(v) => {
            // 1. Blob is not readable anyway. We don't have to try to make it readable.
            // 2. Some 3rd party git tools require a file to be valid utf-8. So I'm using base64.
            // 3. "null" is also a valid base64 output. In order to avoid that, I add a prefix to the output.
            format!("b{}", encode_base64(&v))
        },
    }
}

pub(crate) fn value_from_string(s: &str) -> Option<Value> {
    let b = s.as_bytes();

    match b.get(0) {
        Some(b'n') => {
            if b == b"null" {
                Some(Value::Null)
            } else {
                None
            }
        },
        // `value_to_string(Value::Real(n))` will always contain ".",
        // so `s.parse::<i64>()` can always tell whether it's
        // `Value::Real` or `Value::Integer`.
        Some(b'0'..=b'9' | b'-') => match s.parse::<i64>() {
            Ok(n) => Some(Value::Integer(n)),
            Err(_) => match s.parse::<f64>() {
                Ok(n) => Some(Value::Real(n)),
                Err(_) => None,
            }
        },
        Some(b'"') => match parse_string_literal(b) {
            Some((s, i)) => {
                if i == b.len() - 1 {
                    Some(Value::Text(s))
                }

                else {
                    None
                }
            },
            None => None,
        },
        Some(b'b') => match decode_base64(s.get(1..).unwrap()) {
            Ok(v) => Some(Value::Blob(v)),
            Err(_) => None,
        },
        Some(_) => None,
        None => None,
    }
}
