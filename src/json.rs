use std::collections::HashMap;

// based on the charts on json.org

#[derive(Debug, PartialEq)]
pub enum IntOrFloat {
    Int(i64),
    Float(f64)
}

impl IntOrFloat {
    pub const fn into_int(self) -> i64 {
        match self {
            Self::Int(i) => i,
            Self::Float(f) => f as i64
        }
    }

    pub const fn into_float(self) -> f64 {
        match self {
            Self::Float(f) => f,
            Self::Int(i) => i as f64
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    Number(IntOrFloat),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>)
}

#[derive(Debug, PartialEq)]
pub enum Error {
    Syntax,
    NotClosed,
    BadUtf8
}

const fn check_inbounds(string: &[u8], start_index: usize) -> bool {
    start_index < string.len()
}

const fn skip_whitespace(string: &[u8], start_index: usize) -> usize {
    let mut current_index = start_index;
    while check_inbounds(string, current_index) {
        match string[current_index] {
            b' '|b'\t'|b'\r'|b'\n' => {
                current_index += 1;
            },
            _ => {
                return current_index;
            }
        }
    }
    current_index
}

const fn parse_null(string: &[u8], start_index: usize) -> Result<(Value, usize), Error> {
    if check_inbounds(string, start_index + 3) {
        match (string[start_index+1], string[start_index+2], string[start_index+3]) {
            (b'u', b'l', b'l') => Ok((Value::Null, start_index + 4)),
            _ => Err(Error::Syntax)
        }
    } else {
        Err(Error::Syntax)
    }
}

const fn parse_true(string: &[u8], start_index: usize) -> Result<(Value, usize), Error> {
    if check_inbounds(string, start_index + 3) {
        match (string[start_index+1], string[start_index+2], string[start_index+3]) {
            (b'r', b'u', b'e') => Ok((Value::Boolean(true), start_index + 4)),
            _ => Err(Error::Syntax)
        }
    } else {
        Err(Error::Syntax)
    }
}

const fn parse_false(string: &[u8], start_index: usize) -> Result<(Value, usize), Error> {
    if check_inbounds(string, start_index + 4) {
        match (string[start_index+1], string[start_index+2], string[start_index+3], string[start_index+4]) {
            (b'a', b'l', b's', b'e') => Ok((Value::Boolean(false), start_index + 5)),
            _ => Err(Error::Syntax)
        }
    } else {
        Err(Error::Syntax)
    }
}

fn parse_number(string: &[u8], start_index: usize) -> Result<(Value, usize), Error> {
    const fn is_number(value: u8) -> bool {
        value >= b'0' && value <= b'9'
    }

    let mut current_index = start_index;

    let (signi, signf) = if string[current_index] == b'-' {
        current_index += 1;
        if !check_inbounds(string, current_index) {
            return Err(Error::Syntax);
        }
        (-1i64, -1f64)
    } else {
        (1i64, 1f64)
    };

    let left_of_decimal = match string[current_index] {
        b'0' => {
            current_index += 1;
            0
        },
        b'1'..=b'9' => {
            let mut number = (string[current_index] - b'0') as i64;
            current_index += 1;
            while
                check_inbounds(string, current_index)
                && is_number(string[current_index])
            {
                number *= 10;
                number += (string[current_index] - b'0') as i64;
                current_index += 1;
            }
            number
        },
        _ => {
            return Err(Error::Syntax);
        }
    };

    if !check_inbounds(string, current_index) || (
        string[current_index] != b'.'
        && string[current_index] != b'e'
        && string[current_index] != b'E'
    ) {
        return Ok((
            Value::Number(IntOrFloat::Int(signi * left_of_decimal)),
            current_index
        ));
    }

    let right_of_decimal = if string[current_index] == b'.' {
        current_index += 1;
        if !check_inbounds(string, current_index) || !is_number(string[current_index]) {
            return Err(Error::Syntax);
        }
        let mut current_decimal_place = 0.1f64;
        let mut number = 0f64;
        while
            check_inbounds(string, current_index)
            && is_number(string[current_index])
        {
            number += (string[current_index] - b'0') as f64 * current_decimal_place;
            current_decimal_place /= 10f64;
            current_index += 1;
        }
        Some(number)
    } else {
        None
    };

    if check_inbounds(string, current_index) && (
        string[current_index] == b'e' || string[current_index] == b'E'
    ) {
        current_index += 1;
        if !check_inbounds(string, current_index) {
            return Err(Error::Syntax);
        }
        let exponent_sign = match string[current_index] {
            b'-' => {
                current_index += 1;
                if !check_inbounds(string, current_index) {
                    return Err(Error::Syntax);
                }
                -1i32
            },
            b'+' => {
                current_index += 1;
                if !check_inbounds(string, current_index) {
                    return Err(Error::Syntax);
                }
                1i32
            },
            b'0'..=b'9' => {
                1i32
            },
            _ => {
                return Err(Error::Syntax);
            }
        };

        let exponent_amount = {
            let mut number = 0i32;
            while
                check_inbounds(string, current_index)
                && is_number(string[current_index])
            {
                number *= 10;
                number += (string[current_index] - b'0') as i32;
                current_index += 1;
            }
            number
        };

        let fraction = match right_of_decimal {
            Some(number) => number,
            None => 0f64
        };
        let mantissa = signf * (
            (left_of_decimal as f64) + fraction
        );
        let exponent = exponent_sign * exponent_amount;
        let result = IntOrFloat::Float(
            // this powi is the one thing stopping this fn from being const.
            // if powi is ever made const in the future,
            // make this fn a const fn.
            mantissa * 10f64.powi(exponent)
        );
        Ok((Value::Number(result), current_index))
    } else {
        let result = match right_of_decimal {
            Some(fraction) => IntOrFloat::Float(
                signf * (
                    (left_of_decimal as f64) + fraction
                )
            ),
            None => IntOrFloat::Int(
                signi * left_of_decimal
            )
        };
        Ok((Value::Number(result), current_index))
    }
}

fn parse_string(string: &[u8], start_index: usize) -> Result<(Value, usize), Error> {
    let mut current_index = start_index + 1;

    let mut result = String::new();
    loop {
        if !check_inbounds(string, current_index) {
            return Err(Error::NotClosed);
        }

        match string[current_index] {
            b'"' => {
                return Ok((Value::String(result), current_index + 1));
            },
            b'\\' => {
                if !check_inbounds(string, current_index + 1) {
                    return Err(Error::Syntax);
                }
                result.push(match string[current_index + 1] {
                    b'"'|b'\\'|b'/' => {
                        let ch = string[current_index + 1] as char;
                        current_index += 2;
                        ch
                    },
                    b'b' => { // backspace
                        current_index += 2;
                        8 as char
                    },
                    b'f' => { // formfeed
                        current_index += 2;
                        12 as char
                    },
                    b'n' => { // newline
                        current_index += 2;
                        '\n'
                    },
                    b'r' => { // carriage return
                        current_index += 2;
                        '\r'
                    },
                    b't' => { // tab
                        current_index += 2;
                        '\t'
                    },
                    b'u' => { // unicode
                        if !check_inbounds(string, current_index + 5) {
                            return Err(Error::Syntax);
                        }

                        const fn hex_to_nibble(value: u8) -> Option<u32> {
                            match value {
                                b'0'..=b'9' => Some((value - b'0') as u32),
                                b'a'..=b'f' => Some((value - b'a' + 10) as u32),
                                b'A'..=b'F' => Some((value - b'A' + 10) as u32),
                                _ => None
                            }
                        }

                        let codepoint = match (
                            hex_to_nibble(string[current_index + 2]),
                            hex_to_nibble(string[current_index + 3]),
                            hex_to_nibble(string[current_index + 4]),
                            hex_to_nibble(string[current_index + 5])
                        ) {
                            (Some(a), Some(b), Some(c), Some(d)) => {
                                (a << 12) | (b << 8) | (c << 4) | d
                            },
                            _ => {return Err(Error::Syntax);}
                        };
                        if let Some(ch) = char::from_u32(codepoint) {
                            current_index += 6;
                            ch
                        } else {
                            return Err(Error::Syntax);
                        }
                    },
                    _ => {
                        return Err(Error::Syntax);
                    }
                });
            },
            utf8 => {
                const fn invalid_second_byte(value: u8) -> bool {
                    value < 0b10_000000 || value > 0b10_111111
                }

                // handle utf-8
                let unicode = match utf8 {
                    0..=0b01111111 => {
                        current_index += 1;
                        utf8 as u32
                    },
                    0b110_00000..=0b110_11111 => {
                        if !check_inbounds(string, current_index + 1) {
                            return Err(Error::Syntax);
                        }
                        let second = string[current_index + 1];
                        if invalid_second_byte(second) {
                            return Err(Error::BadUtf8);
                        }
                        current_index += 2;
                        let first = ((utf8 & 0b00011111) as u32) << 6;
                        let second = (second & 0b00111111) as u32;
                        first | second
                    },
                    0b1110_0000..=0b1110_1111 => {
                        if !check_inbounds(string, current_index + 2) {
                            return Err(Error::Syntax);
                        }
                        let second = string[current_index + 1];
                        if invalid_second_byte(second) {
                            return Err(Error::BadUtf8);
                        }
                        let third = string[current_index + 2];
                        if invalid_second_byte(third) {
                            return Err(Error::BadUtf8);
                        }
                        current_index += 3;
                        let first = ((utf8 & 0b00001111) as u32) << 12;
                        let second = ((second & 0b00111111) as u32) << 6;
                        let third = (third & 0b00111111) as u32;
                        first | second | third
                    },
                    0b11110_000..=0b11110_111 => {
                        if !check_inbounds(string, current_index + 3) {
                            return Err(Error::Syntax);
                        }
                        let second = string[current_index + 1];
                        if invalid_second_byte(second) {
                            return Err(Error::BadUtf8);
                        }
                        let third = string[current_index + 2];
                        if invalid_second_byte(third) {
                            return Err(Error::BadUtf8);
                        }
                        let fourth = string[current_index + 3];
                        if invalid_second_byte(fourth) {
                            return Err(Error::BadUtf8);
                        }
                        current_index += 4;
                        let first = ((utf8 & 0b00000111) as u32) << 18;
                        let second = ((second & 0b00111111) as u32) << 12;
                        let third = ((third & 0b00111111) as u32) << 6;
                        let fourth = (fourth & 0b00111111) as u32;
                        first | second | third | fourth
                    },
                    _ => {
                      return Err(Error::BadUtf8);
                    }
                };

                if let Some(ch) = char::from_u32(unicode) {
                    result.push(ch);
                } else {
                    return Err(Error::BadUtf8);
                }
            }
        }
    }
}

fn parse_array(string: &[u8], start_index: usize) -> Result<(Value, usize), Error> {
    let mut current_index = start_index + 1;
    if !check_inbounds(string, current_index) {
        return Err(Error::NotClosed);
    }

    current_index = skip_whitespace(string, current_index);

    if !check_inbounds(string, current_index) {
        return Err(Error::NotClosed);
    }
    if string[current_index] == b']' {
        return Ok((Value::Array(vec![]), current_index + 1));
    }

    let mut result = vec![];
    match parse_value(string, current_index) {
        Ok((value, end_index)) => {
            result.push(value);
            current_index = end_index;
        },
        e => {
            return e;
        }
    }

    loop {
        if !check_inbounds(string, current_index) {
            return Err(Error::NotClosed);
        }
        match string[current_index] {
            b']' => {
                return Ok((Value::Array(result), current_index + 1));
            },
            b',' => {
                match parse_value(string, current_index + 1) {
                    Ok((value, end_index)) => {
                        result.push(value);
                        current_index = end_index;
                    },
                    e => {
                        return e;
                    }
                }
            },
            _ => {
                return Err(Error::Syntax);
            }
        }
    }
}

fn parse_object(string: &[u8], start_index: usize) -> Result<(Value, usize), Error> {
    let mut current_index = skip_whitespace(string, start_index + 1);
    if !check_inbounds(string, current_index) {
        return Err(Error::NotClosed);
    }

    if string[current_index] == b'}' {
        return Ok((Value::Object(HashMap::default()), current_index + 1));
    }

    let mut result = HashMap::new();
    loop {
        let key = match parse_string(string, current_index) {
            Ok((Value::String(k), index)) => {
                current_index = skip_whitespace(string, index);
                k
            },
            e => {
                return e;
            }
        };
        if !check_inbounds(string, current_index + 1) || string[current_index] != b':' {
            return Err(Error::Syntax);
        }
        current_index += 1;
        let value = match parse_value(string, current_index) {
            Ok((v, index)) => {
                current_index = index;
                v
            },
            e => {
                return e;
            }
        };
        if !check_inbounds(string, current_index) {
            return Err(Error::NotClosed);
        }
        result.insert(key, value);
        match string[current_index] {
            b'}' => {
                return Ok((Value::Object(result), current_index + 1));
            },
            b',' => {
                current_index = skip_whitespace(string, current_index + 1);
                if !check_inbounds(string, current_index) {
                    return Err(Error::NotClosed);
                }
            },
            _ => {
                return Err(Error::Syntax);
            }
        }
    }
}

pub fn parse_value(string: &[u8], start_index: usize) -> Result<(Value, usize), Error> {
    let value_start_index = skip_whitespace(string, start_index);
    if !check_inbounds(string, value_start_index) {
        return Err(Error::Syntax);
    }

    let result = match string[value_start_index] {
        b'n' => {parse_null(string, value_start_index)},
        b't' => {parse_true(string, value_start_index)},
        b'f' => {parse_false(string, value_start_index)},
        b'[' => {parse_array(string, value_start_index)},
        b'{' => {parse_object(string, value_start_index)},
        b'"' => {parse_string(string, value_start_index)},
        b'-'|b'0'..=b'9' => {parse_number(string, value_start_index)},
        _ => {Err(Error::Syntax)}
    };

    let (value, start_of_trailing_whitespace) = match result {
        Ok(res) => res,
        e => {
            return e;
        }
    };

    let end_index = skip_whitespace(string, start_of_trailing_whitespace);
    Ok((value, end_index))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn true_value() {
        let just_true = parse_value(b"true", 0);
        let true_with_whitespace = parse_value(b"\n\r\t true  ", 0);
        assert_eq!(just_true, Ok((Value::Boolean(true), 4)));
        assert_eq!(true_with_whitespace, Ok((Value::Boolean(true), 10)));
    }

    #[test]
    fn false_value() {
        let just_false = parse_value(b"false", 0);
        let false_with_whitespace = parse_value(b"\n\r\t false  ", 0);
        assert_eq!(just_false, Ok((Value::Boolean(false), 5)));
        assert_eq!(false_with_whitespace, Ok((Value::Boolean(false), 11)));
    }

    #[test]
    fn null() {
        let just_null = parse_value(b"null", 0);
        let null_with_whitespace = parse_value(b"\n\r\t null  ", 0);
        assert_eq!(just_null, Ok((Value::Null, 4)));
        assert_eq!(null_with_whitespace, Ok((Value::Null, 10)));
    }

    #[test]
    fn number() {
        let zero = parse_value(b"0", 0);
        assert_eq!(zero, Ok((Value::Number(IntOrFloat::Int(0)), 1)));

        let zero_ws = parse_value(b" 0 ", 0);
        assert_eq!(zero_ws, Ok((Value::Number(IntOrFloat::Int(0)), 3)));

        let two = parse_value(b"2", 0);
        assert_eq!(two, Ok((Value::Number(IntOrFloat::Int(2)), 1)));

        let neg_two = parse_value(b"-2", 0);
        assert_eq!(neg_two, Ok((Value::Number(IntOrFloat::Int(-2)), 2)));

        let six_seven = parse_value(b"67", 0);
        assert_eq!(six_seven, Ok((Value::Number(IntOrFloat::Int(67)), 2)));

        let neg_six_seven = parse_value(b"-67", 0);
        assert_eq!(neg_six_seven, Ok((Value::Number(IntOrFloat::Int(-67)), 3)));

        let two_e_three = parse_value(b"2e3", 0);
        assert_eq!(two_e_three, Ok((Value::Number(IntOrFloat::Float(2e3)), 3)));

        let two_e_neg_three = parse_value(b"2e-3", 0);
        assert_eq!(two_e_neg_three, Ok((Value::Number(IntOrFloat::Float(2e-3)), 4)));

        let neg_two_e_three = parse_value(b"-2e3", 0);
        assert_eq!(neg_two_e_three, Ok((Value::Number(IntOrFloat::Float(-2e3)), 4)));

        let neg_two_e_neg_three = parse_value(b"-2e-3", 0);
        assert_eq!(neg_two_e_neg_three, Ok((Value::Number(IntOrFloat::Float(-2e-3)), 5)));

        let three_point_five = parse_value(b"3.5", 0);
        assert_eq!(three_point_five, Ok((Value::Number(IntOrFloat::Float(3.5)), 3)));

        let three_point_four_five = parse_value(b"3.45", 0);
        assert_eq!(three_point_four_five, Ok((Value::Number(IntOrFloat::Float(3.45)), 4)));

        let two_three_point_four = parse_value(b"23.4", 0);
        assert_eq!(two_three_point_four, Ok((Value::Number(IntOrFloat::Float(23.4)), 4)));

        let two_three_point_four_five = parse_value(b"23.45", 0);
        assert_eq!(two_three_point_four_five, Ok((Value::Number(IntOrFloat::Float(23.45)), 5)));

        let neg_two_three_point_four_five = parse_value(b"-23.45", 0);
        assert_eq!(neg_two_three_point_four_five, Ok((Value::Number(IntOrFloat::Float(-23.45)), 6)));

        let neg_two_three_point_four_five_e_six = parse_value(b"-23.45e6", 0);
        assert_eq!(neg_two_three_point_four_five_e_six, Ok((Value::Number(IntOrFloat::Float(-23.45e6)), 8)));

        let neg_two_three_point_four_five_e_neg_six_seven = parse_value(b"-23.45E-67", 0);
        assert_eq!(neg_two_three_point_four_five_e_neg_six_seven, Ok((Value::Number(IntOrFloat::Float(-23.45e-67)), 10)));
    }

    #[test]
    fn string() {
        let empty = parse_value(br#""""#, 0);
        assert_eq!(empty, Ok((Value::String("".into()), 2)));

        let quote_slash = parse_value(br#""\"\\""#, 0);
        assert_eq!(quote_slash, Ok((Value::String(r#""\"#.into()), 6)));

        let backspace = parse_value(br#""\b""#, 0);
        assert_eq!(backspace, Ok((Value::String((8 as char).into()), 4)));

        let aaaa = parse_value(br#""A\u0041A\u0041""#, 0);
        assert_eq!(aaaa, Ok((Value::String("AAAA".into()), 16)));

        let ok_hand = parse_value(&[b'"', 0xf0, 0x9f, 0x91, 0x8c, b'"'], 0);
        assert_eq!(ok_hand, Ok((Value::String("👌".into()), 6)));
    }

    #[test]
    fn array() {
        let empty = parse_value(b"[]", 0);
        assert_eq!(empty, Ok((Value::Array(vec![]), 2)));

        let empty_ws = parse_value(b"[ ]", 0);
        assert_eq!(empty_ws, Ok((Value::Array(vec![]), 3)));

        let one_two = parse_value(b"[1,2]", 0);
        assert_eq!(one_two, Ok((Value::Array(vec![Value::Number(IntOrFloat::Int(1)),Value::Number(IntOrFloat::Int(2))]), 5)));

        let one_two_spaced = parse_value(b" [ 1 , 2 ] ", 0);
        assert_eq!(one_two_spaced, Ok((Value::Array(vec![Value::Number(IntOrFloat::Int(1)),Value::Number(IntOrFloat::Int(2))]), 11)));

        let hello_world = parse_value(br#"["hello", "world", false, true, null]"#, 0);
        assert_eq!(hello_world, Ok((Value::Array(vec![Value::String("hello".into()), Value::String("world".into()), Value::Boolean(false), Value::Boolean(true), Value::Null]), 37)));
    }

    #[test]
    fn object() {
        let empty = parse_value(b"{}", 0);
        assert_eq!(empty, Ok((Value::Object(HashMap::new()), 2)));

        let empty_ws = parse_value(b"{ }", 0);
        assert_eq!(empty_ws, Ok((Value::Object(HashMap::new()), 3)));

        let hello_world = parse_value(br#"{"hello":"world"}"#, 0);
        assert_eq!(hello_world, Ok((Value::Object(HashMap::from([("hello".into(), Value::String("world".into()))])), 17)));

        let six_seven = parse_value(br#"{"six":6,"seven":7.0}"#, 0);
        assert_eq!(six_seven, Ok((Value::Object(HashMap::from([("six".into(), Value::Number(IntOrFloat::Int(6))), ("seven".into(), Value::Number(IntOrFloat::Float(7f64)))])), 21)));

        let six_seven_ws = parse_value(br#" { "six" : 6 , "seven":7.0 } "#, 0);
        assert_eq!(six_seven_ws, Ok((Value::Object(HashMap::from([("six".into(), Value::Number(IntOrFloat::Int(6))), ("seven".into(), Value::Number(IntOrFloat::Float(7f64)))])), 29)));
    }

    #[test]
    fn nested() {
        let nested = parse_value(br#"{"first":[2,"three",4.0],"five":6,"seven":{"eight":[9,{"ten":11,"twelve":13}]}}"#, 0);
        assert_eq!(nested, Ok((
            Value::Object(HashMap::from([
                ("first".into(), Value::Array(vec![
                    Value::Number(IntOrFloat::Int(2)),
                    Value::String("three".into()),
                    Value::Number(IntOrFloat::Float(4f64))
                ])),
                ("five".into(), Value::Number(IntOrFloat::Int(6))),
                ("seven".into(), Value::Object(
                    HashMap::from([
                        (
                            "eight".into(),
                            Value::Array(vec![
                                Value::Number(IntOrFloat::Int(9)),
                                Value::Object(HashMap::from([
                                    ("ten".into(), Value::Number(IntOrFloat::Int(11))),
                                    ("twelve".into(), Value::Number(IntOrFloat::Int(13)))
                                ]))
                            ])
                        )
                    ])
                ))
            ]))
        , 79)));
    }
}
