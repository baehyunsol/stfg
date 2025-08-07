use super::{
    Value,
    value_from_string,
    value_to_string,
};

#[test]
fn value_roundtrip() {
    for v1 in vec![
        Value::Null,
        Value::Integer(0),
        Value::Integer(1),
        Value::Integer(-1),
        Value::Integer(i64::MAX),
        Value::Integer(i64::MIN),
        Value::Real(1.0),
        Value::Real(1.5),
        Value::Real(0.0),
        Value::Real(-1.0),
        Value::Real(-1.5),
        Value::Real(123.456789),
        Value::Real(-123.456789),
        Value::Text(String::new()),
        Value::Text(String::from("Hello, world!")),
        Value::Text(String::from("\"This is a string.\"")),
        Value::Text(String::from("This is a string.\nAnd this is a newline.")),
        Value::Text(String::from("Bunch of escapes: \", \', \\, \n, \t, \r...")),
        Value::Blob(vec![]),
        Value::Blob(b"Hello, world!".to_vec()),
        Value::Blob(b"\"This is a string.\"".to_vec()),
        Value::Blob(b"This is a string.\nAnd this is a newline.".to_vec()),
        Value::Blob(b"Bunch of escapes: \", \', \\, \n, \t, \r...".to_vec()),
    ] {
        let s1 = value_to_string(&v1);
        let v2 = value_from_string(&s1).unwrap();
        let s2 = value_to_string(&v2);
        let v3 = value_from_string(&s2).unwrap();

        assert_eq!(s1, s2);
        assert_eq!(v1, v2);
        assert_eq!(v2, v3);
    }
}
