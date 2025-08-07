use crate::error::Error;
use crate::util::parse_string_literal;
use crate::value::{
    Value,
    value_from_string,
    value_to_string,
};
use ragit_fs::{
    WriteMode,
    read_string,
    write_string,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

#[derive(Clone)]
pub struct Record {
    // Read the comments of `RecordId`.
    pub id: RecordId,

    // The order matters a lot!!
    pub fields: Vec<(String, Value)>,
}

/// A table is splitted into multiple files based on `RecordId`. In order to do that,
/// 1. it has to be a hash value so that the records are evenly distributed.
/// 2. when a field of a record is updated, its id MUST NOT change (so that `git diff` can easily spot the difference).
/// 3. it has to be deterministic and only depends on values, not internal data structure like rowid (so that git can easily track the history).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RecordId(pub(crate) u64);

impl RecordId {
    pub fn hash(data: &[Value]) -> RecordId {
        let mut hasher = DefaultHasher::new();

        for value in data.iter() {
            match value {
                Value::Null => {
                    hasher.write(b"0");
                },
                Value::Integer(n) => {
                    hasher.write(b"1");
                    hasher.write(&n.to_le_bytes());
                },
                Value::Real(n) => {
                    hasher.write(b"2");
                    hasher.write(&n.to_le_bytes());
                },
                Value::Text(s) => {
                    hasher.write(b"3");
                    hasher.write(s.as_bytes());
                },
                Value::Blob(v) => {
                    hasher.write(b"4");
                    hasher.write(&v);
                },
            }
        }

        RecordId(hasher.finish())
    }

    /// stfg splits a table into 512 files.
    pub fn prefix(&self) -> u64 {
        self.0 >> 55
    }
}

pub(crate) fn read_records(path: &str) -> Result<Vec<Record>, Error> {
    let r = read_string(path)?;
    let mut result = vec![];

    let mut id = None;
    let mut fields = vec![];

    for line in r.lines() {
        let bytes = line.as_bytes();

        match bytes.get(0) {
            Some(b'"') => {
                fields.push(parse_line(bytes)?);
            },
            Some(b'0'..=b'9' | b'a'..=b'f') => match u64::from_str_radix(line, 16) {
                Ok(n) => match id {
                    Some(_) => {
                        return Err(Error::CorruptedDataFile(String::from("id appears twice")));
                    },
                    None => {
                        id = Some(RecordId(n));
                    },
                },
                Err(_) => {
                    return Err(Error::CorruptedDataFile(format!("failed to parse id: {line}")));
                },
            },
            Some(b) => {
                if let Some(id) = id {
                    return Err(Error::CorruptedDataFile(format!(
                        "expected a field name, got {} (at id {:016x})",
                        String::from_utf8_lossy(&[*b]),
                        id.0,
                    )));
                }

                else {
                    return Err(Error::CorruptedDataFile(format!("expected an id, got {}", String::from_utf8_lossy(&[*b]))))
                }
            },
            None => match id {
                Some(id_) => {
                    result.push(Record {
                        id: id_,
                        fields,
                    });

                    id = None;
                    fields = vec![];
                },
                None => {
                    return Err(Error::CorruptedDataFile(String::from("a record without an id")));
                },
            },
        }
    }

    if let Some(id) = id {
        result.push(Record {
            id,
            fields,
        });
    }

    Ok(result)
}

pub(crate) fn write_records(path: &str, records: &[Record]) -> Result<(), Error> {
    let mut lines = vec![];

    for record in records.iter() {
        lines.push(format!("{:016x}", record.id.0));

        for (field, value) in record.fields.iter() {
            // `field` can have an arbitrary character, so we have to use `Debug` format instead of `Display`.
            lines.push(format!("{field:?}={}", value_to_string(value)));
        }

        lines.push(String::new());
    }

    write_string(
        path,
        &lines.join("\n"),
        WriteMode::CreateOrTruncate,
    )?;
    Ok(())
}

fn parse_line(s: &[u8]) -> Result<(String, Value), Error> {
    let (field_name, mut cursor) = match parse_string_literal(s) {
        Some((s, i)) => (s, i + 1),
        None => {
            return Err(Error::CorruptedDataFile(format!("failed to parse data: {}", String::from_utf8_lossy(s))));
        },
    };

    match s.get(cursor) {
        Some(b'=') => {
            cursor += 1;
        },
        Some(b) => {
            return Err(Error::CorruptedDataFile(format!("expected '=', got {}", String::from_utf8_lossy(&[*b]))));
        },
        None => {
            return Err(Error::CorruptedDataFile(String::from("expected '=', got nothing")));
        },
    }

    let value_s = match String::from_utf8(s[cursor..].to_vec()) {
        Ok(s) => s,
        Err(_) => {
            return Err(Error::CorruptedDataFile(format!("corrupted value")));
        },
    };

    let value = match value_from_string(&value_s) {
        Some(v) => v,
        None => {
            return Err(Error::CorruptedDataFile(format!("failed to parse value: {value_s}")));
        },
    };

    Ok((field_name, value))
}
