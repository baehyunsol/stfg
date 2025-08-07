use base64::Engine;
use crate::error::Error;

pub(crate) fn encode_base64(bytes: &[u8]) -> String {
    base64::prelude::BASE64_STANDARD.encode(bytes)
}

pub(crate) fn decode_base64(s: &str) -> Result<Vec<u8>, Error> {
    Ok(base64::prelude::BASE64_STANDARD.decode(s)?)
}

// `bytes` must start with '"'. The string literal may
// end earlier than `bytes`. It returns the index of '"' that
// finishes the string literal.
pub(crate) fn parse_string_literal(bytes: &[u8]) -> Option<(String, usize)> {
    match bytes.get(0) {
        Some(b'"') => {},
        _ => {
            return None;
        },
    }

    let mut buffer = vec![];
    let mut escaped = false;
    let mut ended_at = 0;

    for (i, b) in bytes[1..].iter().enumerate() {
        if escaped {
            match b {
                b'n' => {
                    buffer.push(b'\n');
                },
                b'r' => {
                    buffer.push(b'\r');
                },
                b't' => {
                    buffer.push(b'\t');
                },
                b'0' => {
                    buffer.push(b'\0');
                },
                _ => {
                    buffer.push(*b);
                },
            }

            escaped = false;
        }

        else {
            match b {
                b'\\' => {
                    escaped = true;
                },
                b'"' => {
                    ended_at = i + 1;
                    break;
                },
                _ => {
                    buffer.push(*b);
                },
            }
        }
    }

    match String::from_utf8(buffer) {
        Ok(s) => Some((s, ended_at)),
        Err(_) => None,
    }
}
