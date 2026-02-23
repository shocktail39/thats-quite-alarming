use std::collections::HashMap;

// based on the charts on json.org

#[derive(Debug, PartialEq)]
pub enum IntOrFloat {
    Int(i64),
    Float(f64)
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
    NotClosed
}

const fn check_inbounds(string: &[u8], start_index: usize) -> Result<(), Error> {
    if start_index < string.len() {
        Ok(())
    } else {
        Err(Error::Syntax)
    }
}

const fn skip_whitespace(string: &[u8], start_index: usize) -> usize {
    let mut i = start_index;
    while i < string.len() {
        match string[i] {
            b' '|b'\t'|b'\r'|b'\n' => {/* do nothing, continue loop*/},
            _ => {return i;}
        }
        i += 1;
    }
    string.len()
}

const fn parse_null(string: &[u8], start_index: usize) -> Result<(Value, usize), Error> {
    if let Err(e) = check_inbounds(string, start_index + 3) {
        Err(e)
    } else {
        match (string[start_index+1], string[start_index+2], string[start_index+3]) {
            (b'u', b'l', b'l') => Ok((Value::Null, start_index + 4)),
            _ => Err(Error::Syntax)
        }
    }
}

const fn parse_true(string: &[u8], start_index: usize) -> Result<(Value, usize), Error> {
    if let Err(e) = check_inbounds(string, start_index + 3) {
        Err(e)
    } else {
        match (string[start_index+1], string[start_index+2], string[start_index+3]) {
            (b'r', b'u', b'e') => Ok((Value::Boolean(true), start_index + 4)),
            _ => Err(Error::Syntax)
        }
    }
}

const fn parse_false(string: &[u8], start_index: usize) -> Result<(Value, usize), Error> {
    if let Err(e) = check_inbounds(string, start_index + 4) {
        Err(e)
    } else {
        match (string[start_index+1], string[start_index+2], string[start_index+3], string[start_index+4]) {
            (b'a', b'l', b's', b'e') => Ok((Value::Boolean(false), start_index + 5)),
            _ => Err(Error::Syntax)
        }
    }
}

fn parse_number(string: &[u8], start_index: usize) -> Result<(Value, usize), Error> {
    let mut current_index = start_index;

    let (signi, signf) = if string[current_index] == b'-' {
        current_index += 1;
        if let Err(e) = check_inbounds(string, current_index) {
            return Err(e);
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
                current_index < string.len()
                && string[current_index] >= b'0'
                && string[current_index] <= b'9'
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

    if current_index >= string.len() || (
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
        if let Err(e) = check_inbounds(string, current_index) {
            return Err(e);
        }
        if string[current_index] < b'0' || string[current_index] > b'9' {
            return Err(Error::Syntax);
        }
        let mut current_decimal_place = 0.1f64;
        let mut number = 0f64;
        while
            current_index < string.len()
            && string[current_index] >= b'0'
            && string[current_index] <= b'9'
        {
            number += (string[current_index] - b'0') as f64 * current_decimal_place;
            current_decimal_place /= 10f64;
            current_index += 1;
        }
        Some(number)
    } else {
        None
    };

    if current_index < string.len() && (
        string[current_index] == b'e' || string[current_index] == b'E'
    ) {
        current_index += 1;
        if let Err(e) = check_inbounds(string, current_index) {
            return Err(e);
        }
        let exponent_sign = match string[current_index] {
            b'-' => {
                current_index += 1;
                if let Err(e) = check_inbounds(string, current_index) {
                    return Err(e);
                }
                -1i32
            },
            b'+' => {
                current_index += 1;
                if let Err(e) = check_inbounds(string, current_index) {
                    return Err(e);
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
                current_index < string.len()
                && string[current_index] >= b'0'
                && string[current_index] <= b'9'
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

pub fn parse_value(string: &[u8], start_index: usize) -> Result<(Value, usize), Error> {
    let value_start_index = skip_whitespace(string, start_index);
    check_inbounds(string, value_start_index)?;

    let (value, start_of_trailing_whitespace) = match string[value_start_index] {
        b'n' => {parse_null(string, value_start_index)},
        b't' => {parse_true(string, value_start_index)},
        b'f' => {parse_false(string, value_start_index)},
        b'[' => {todo!()},
        b'{' => {todo!()},
        b'"' => {todo!()},
        b'-'|b'0'..=b'9' => {parse_number(string, value_start_index)},
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

    #[test]
    fn test_number() {
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
}
