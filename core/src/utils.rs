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

pub fn split_letter_number_pairs(s: &str) -> Vec<(char, i32)> {
    let mut result = Vec::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_ascii_alphabetic() {
            let mut number_str = String::new();
            if let Some(&next) = chars.peek() && next == '-' {
                number_str.push(chars.next().unwrap());
            }
            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit() {
                    number_str.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            let number = number_str.parse::<i32>().unwrap_or(0);
            result.push((c, number));
        }
    }

    result
}

pub fn separate_comment(s: &str) -> (Vec<&str>, &str) {
    if let Some(colon_pos) = s.find(':') {
        // find the position of the word before the colon
        let word_start = s[..colon_pos]
            .rfind(|c: char| c.is_whitespace())
            .map(|pos| pos + 1)
            .unwrap_or(0);

        let words: Vec<&str> = s[..word_start].split_whitespace().collect();
        let comment = s[word_start..].trim();
        (words, comment)
    } else {
        let words: Vec<&str> = s.split_whitespace().collect();
        (words, "")
    }
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

#[test]
fn test_split_letter_number_pairs() {
    let input = "a523b8876B98173X3Z00F-432";
    let parsed = split_letter_number_pairs(input);
    assert_eq!(parsed.len(), 6);
    assert_eq!(parsed[0], ('a', 523));
    assert_eq!(parsed[1], ('b', 8876));
    assert_eq!(parsed[2], ('B', 98173));
    assert_eq!(parsed[3], ('X', 3));
    assert_eq!(parsed[4], ('Z', 0));
    assert_eq!(parsed[5], ('F', -432));
}

#[test]
fn test_separate_comment() {
    assert_eq!(
        separate_comment("No colon here"),
        (vec!["No", "colon", "here"], "")
    );
    assert_eq!(
        separate_comment("RTX4050 fps120 game: condor soaring simulator"),
        (vec!["RTX4050", "fps120"], "game: condor soaring simulator")
    );
    assert_eq!(
        separate_comment("here: we: have: multiple: colons"),
        (vec![], "here: we: have: multiple: colons")
    );
}
