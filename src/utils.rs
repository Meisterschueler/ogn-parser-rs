pub fn split_value_unit(s: &str) -> Option<(&str, &str)> {
    let length = s.len();
    s.chars()
        .enumerate()
        .scan(
            (false, false, false),
            |(has_digits, is_signed, has_decimal), (idx, elem)| {
                if idx == 0 && ['+', '-'].contains(&elem) {
                    *is_signed = true;
                    Some((idx, *has_digits))
                } else if elem == '.' && !(*has_decimal) {
                    *has_decimal = true;
                    Some((idx, *has_digits))
                } else if elem.is_ascii_digit() {
                    *has_digits = true;
                    Some((idx, *has_digits))
                } else {
                    None
                }
            },
        )
        .last()
        .and_then(|(split_position, has_digits)| {
            if has_digits && split_position != length - 1 {
                Some((&s[..(split_position + 1)], &s[(split_position + 1)..]))
            } else {
                None
            }
        })
}

pub fn extract_values(part: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current_value = String::new();

    for c in part.chars() {
        if c == '+' || c == '-' {
            if !current_value.is_empty() {
                result.push(current_value.clone());
            }
            current_value = String::new();
            current_value.push(c);
        } else if char::is_numeric(c) || c == '.' {
            current_value.push(c);
        } else if !current_value.is_empty() {
            result.push(current_value.clone());
            current_value = String::new();
        }
    }

    if !current_value.is_empty() {
        result.push(current_value.clone());
    }
    result
}

#[test]
fn test_extract_values() {
    assert_eq!(
        extract_values("-1.2+3.4-5.6dB7km"),
        vec!["-1.2", "+3.4", "-5.6", "7"]
    );
}

#[test]
fn test_split_value_unit() {
    assert_eq!(split_value_unit("1dB"), Some(("1", "dB")));
    assert_eq!(split_value_unit("-3kHz"), Some(("-3", "kHz")));
    assert_eq!(split_value_unit("+3.141rpm"), Some(("+3.141", "rpm")));
    assert_eq!(split_value_unit("+.1A"), Some(("+.1", "A")));
    assert_eq!(split_value_unit("-12.V"), Some(("-12.", "V")));
    assert_eq!(split_value_unit("+kVA"), None);
    assert_eq!(split_value_unit("25"), None);
}
