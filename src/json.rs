use std::collections::HashMap;

// based on the charts on json.org

#[derive(Debug, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>)
}

#[derive(Debug, PartialEq)]
pub enum Error {
    Syntax,
    NotClosed
}

fn skip_whitespace(string: &[u8], start_index: usize) -> usize {
    for i in start_index..string.len() {
        match string[i] {
            b' '|b'\t'|b'\r'|b'\n' => {/* do nothing, continue for loop*/},
            _ => {return i;}
        }
    }
    string.len()
}

fn parse_null(string: &[u8], start_index: usize) -> Result<(Value, usize), Error> {
    if start_index + 3 >= string.len() {
        Err(Error::Syntax)
    } else {
        match (string[start_index+1], string[start_index+2], string[start_index+3]) {
            (b'u', b'l', b'l') => Ok((Value::Null, start_index + 4)),
            _ => Err(Error::Syntax)
        }
    }
}

fn parse_true(string: &[u8], start_index: usize) -> Result<(Value, usize), Error> {
    if start_index + 3 >= string.len() {
        Err(Error::Syntax)
    } else {
        match (string[start_index+1], string[start_index+2], string[start_index+3]) {
            (b'r', b'u', b'e') => Ok((Value::Boolean(true), start_index + 4)),
            _ => Err(Error::Syntax)
        }
    }
}

fn parse_false(string: &[u8], start_index: usize) -> Result<(Value, usize), Error> {
    if start_index + 4 >= string.len() {
        Err(Error::Syntax)
    } else {
        match (string[start_index+1], string[start_index+2], string[start_index+3], string[start_index+4]) {
            (b'a', b'l', b's', b'e') => Ok((Value::Boolean(false), start_index + 5)),
            _ => Err(Error::Syntax)
        }
    }
}

pub fn parse_value(string: &[u8], start_index: usize) -> Result<(Value, usize), Error> {
    let value_start_index = skip_whitespace(string, start_index);
    if value_start_index >= string.len() {
        return Err(Error::Syntax);
    }

    let (value, start_of_trailing_whitespace) = match string[value_start_index] {
        b'n' => {parse_null(string, value_start_index)},
        b't' => {parse_true(string, value_start_index)},
        b'f' => {parse_false(string, value_start_index)},
        b'[' => {todo!()},
        b'{' => {todo!()},
        b'"' => {todo!()},
        b'-'|b'0'..=b'9' => {todo!()},
        _ => {Err(Error::Syntax)}
    }?;
    let end_index = skip_whitespace(string, start_of_trailing_whitespace);
    Ok((value, end_index))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_true() {
        let just_true = parse_value(b"true", 0);
        let true_with_whitespace = parse_value(b"\n\r\t true  ", 0);
        assert_eq!(just_true, Ok((Value::Boolean(true), 4)));
        assert_eq!(true_with_whitespace, Ok((Value::Boolean(true), 10)));
    }

    #[test]
    fn test_false() {
        let just_false = parse_value(b"false", 0);
        let false_with_whitespace = parse_value(b"\n\r\t false  ", 0);
        assert_eq!(just_false, Ok((Value::Boolean(false), 5)));
        assert_eq!(false_with_whitespace, Ok((Value::Boolean(false), 11)));
    }

    #[test]
    fn test_null() {
        let just_null = parse_value(b"null", 0);
        let null_with_whitespace = parse_value(b"\n\r\t null  ", 0);
        assert_eq!(just_null, Ok((Value::Null, 4)));
        assert_eq!(null_with_whitespace, Ok((Value::Null, 10)));
    }
}
