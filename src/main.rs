use std::collections::HashMap;

fn main() {
    let _ = dbg!(parse(
        &r#"{"key": ["val1"]}"#.to_string().chars().collect::<Vec<_>>(),
        &mut 0usize
    ));
}

#[derive(Debug, PartialEq)]
enum Primitive {
    Bool(bool),
    Null,
    String(String),
    Number(f64),
    Array(Vec<Primitive>),
    Object(HashMap<String, Primitive>),
}

fn parse(source: &[char], counter: &mut usize) -> anyhow::Result<Primitive> {
    if *counter >= source.len() {
        anyhow::bail!("unexpected end-of-input or empty string")
    }

    if source[*counter] == ' ' || source[*counter] == '\n' {
        parse_whitespaces_and_line_breaks(source, counter)?
    }

    match source[*counter] {
        '"' => {
            let result = parse_string(source, counter)?;
            Ok(Primitive::String(result))
        }
        '-' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
            let result = parse_number(source, counter)?;
            Ok(Primitive::Number(result))
        }
        '{' => {
            let result = parse_object(source, counter)?;
            Ok(Primitive::Object(result))
        }
        '[' => {
            let result = parse_array(source, counter)?;
            Ok(Primitive::Array(result))
        }
        't' | 'f' => {
            let result = parse_bool(source, counter)?;
            Ok(Primitive::Bool(result))
        }
        'n' => {
            parse_null(source, counter)?;
            Ok(Primitive::Null)
        }
        some_unknown_char => {
            anyhow::bail!(format!(
                "unknown token `{}` at {}",
                some_unknown_char, counter
            ))
        }
    }
}

fn parse_string(source: &[char], counter: &mut usize) -> anyhow::Result<String> {
    if source[*counter] != '"' {
        anyhow::bail!(format!(
            "unknown token `{}` at {}",
            source[*counter], *counter
        ))
    }
    let mut res = String::new();
    let mut is_escaped = false;
    *counter += 1;

    while *counter < source.len() && (source[*counter] != '"' || is_escaped) {
        if source[*counter] == '\\' && !is_escaped {
            is_escaped = true;
            *counter += 1;
            continue;
        }

        if is_escaped {
            match source[*counter] {
                '\\' | '"' | '/' => {
                    res.push(source[*counter]);
                }
                'b' => {
                    // backspace
                    res.push('\u{0008}');
                }
                'f' => {
                    // form feed
                    res.push('\u{000C}');
                }
                'n' => {
                    // line feed
                    res.push('\u{000A}');
                }
                'r' => {
                    // carriage return
                    res.push('\u{000D}');
                }
                't' => {
                    // tab
                    res.push('\u{0009}')
                }
                'u' => {
                    // unicode escape sequence
                    if *counter + 4 >= source.len() {
                        anyhow::bail!("unexpected end-of-input while parsing escape sequence")
                    }
                    let code_point_str = source[(*counter + 1)..=(*counter + 4)]
                        .iter()
                        .collect::<String>();
                    let code_point = u32::from_str_radix(&code_point_str, 16).map_err(|_| {
                        anyhow::format_err!(
                            "invalid escape sequence `\\u{code_point_str}` at {}",
                            *counter + 1
                        )
                    })?;

                    // SAFETY: JSON standard does not mention invalid sequence of Unicode code points
                    //         I am just passing it through here
                    let c = unsafe { char::from_u32_unchecked(code_point) };
                    res.push(c);
                    // will be += 1 ed afterwards
                    *counter += 4
                }
                _ => {
                    anyhow::bail!(format!(
                        "invalid escape sequence in string after backslash: `{}` at {}",
                        source[*counter], *counter
                    ))
                }
            }

            is_escaped = false;
            *counter += 1;
            continue;
        }

        res.push(source[*counter]);
        *counter += 1;
    }

    if *counter >= source.len() && source[*counter - 1] != '"' {
        anyhow::bail!("unexpected end-of-input while parsing string")
    }

    *counter += 1;
    Ok(res)
}

fn parse_number(source: &[char], counter: &mut usize) -> anyhow::Result<f64> {
    let strlen = source.len();

    match source[*counter] {
        '-' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
            let mut integer_part = 0f64;
            let plusminus = if source[*counter] == '-' {
                *counter += 1;
                -1f64
            } else {
                1f64
            };

            // assuming first char is not '0' (due to match arm condition)
            while *counter < strlen && '0' <= source[*counter] && source[*counter] <= '9' {
                integer_part *= 10f64;
                integer_part += (source[*counter] as u32 - 0x30) as f64;
                *counter += 1;
            }

            let mut fractional_part = 0f64;
            if source[*counter] == '.' {
                let counter_base = *counter;
                *counter += 1;

                // there must be 1+ digit(s) after decimal-point
                if source[*counter] <= '0' || '9' <= source[*counter] {
                    anyhow::bail!(format!(
                        "unknown token `{}` at {}",
                        source[*counter], *counter
                    ))
                }

                while *counter < strlen && '0' <= source[*counter] && source[*counter] <= '9' {
                    fractional_part += (source[*counter] as u32 - 0x30) as f64
                        * 0.1f64.powi((*counter - counter_base).try_into().unwrap()); // FIXME: better handling of counter overflow
                    *counter += 1;
                }
            }

            let mut exp = 0i32;
            if source[*counter] == 'e' || source[*counter] == 'E' {
                *counter += 1;
                let plusminus = if source[*counter] == '-' {
                    *counter += 1;
                    -1i32
                } else {
                    1i32
                };

                // there must be 1+ digit(s) after e/E
                if source[*counter] <= '0' || '9' <= source[*counter] {
                    anyhow::bail!(format!(
                        "unknown token `{}` at {}",
                        source[*counter], *counter
                    ))
                }

                while *counter < strlen && '0' <= source[*counter] && source[*counter] <= '9' {
                    exp *= 10;
                    exp += source[*counter] as i32 - 0x30;
                    *counter += 1;
                }

                exp *= plusminus;
            }

            let result = (integer_part + fractional_part) * 10f64.powi(exp) * plusminus;
            Ok(result)
        }
        _ => anyhow::bail!(format!(
            "unknown token `{}` at {}",
            source[*counter], *counter
        )),
    }
}

fn parse_object(
    source: &[char],
    counter: &mut usize,
) -> anyhow::Result<HashMap<String, Primitive>> {
    #[derive(Debug, PartialEq)]
    enum InternalState {
        Fresh,
        KeyDetermined(String),
    }

    if source[*counter] != '{' {
        anyhow::bail!(format!(
            "unknown token `{}` at {}",
            source[*counter], *counter
        ))
    }

    let mut res: HashMap<String, Primitive> = HashMap::new();
    *counter += 1;
    let strlen = source.len();

    let mut state = InternalState::Fresh;
    while *counter < strlen && source[*counter] != '}' {
        match state {
            InternalState::Fresh => {
                parse_whitespaces_and_line_breaks(source, counter)?;

                let key = parse_string(source, counter)?;
                state = InternalState::KeyDetermined(key);
            }
            InternalState::KeyDetermined(key) => {
                parse_whitespaces_and_line_breaks(source, counter)?;
                if source[*counter] != ':' {
                    anyhow::bail!(format!(
                        "unknown token `{}` at {}",
                        source[*counter], *counter
                    ))
                }
                *counter += 1;

                parse_whitespaces_and_line_breaks(source, counter)?;
                let child = parse(source, counter)?;

                res.insert(key, child);
                state = InternalState::Fresh;

                parse_whitespaces_and_line_breaks(source, counter)?;
                if *source.get(*counter).ok_or(anyhow::format_err!(
                    "unexpected end-of-input while parsing object"
                ))? != ','
                {
                    break;
                } else {
                    *counter += 1;
                }
            }
        }
    }

    if *counter >= strlen && source[*counter - 1] != '}' {
        anyhow::bail!("unexpected end-of-input while parsing object")
    }

    *counter += 1;
    Ok(res)
}

fn parse_array(source: &[char], counter: &mut usize) -> anyhow::Result<Vec<Primitive>> {
    let strlen = source.len();
    let result = match source[*counter] {
        '[' => {
            *counter += 1;
            let mut inner = Vec::new();
            while *counter < strlen && source[*counter] != ']' {
                parse_whitespaces_and_line_breaks(source, counter)?;
                inner.push(parse(source, counter)?);
                parse_whitespaces_and_line_breaks(source, counter)?;

                if source[*counter] == ',' {
                    *counter += 1;
                    continue;
                }

                if source[*counter] != ']' {
                    anyhow::bail!(format!(
                        "unknown token `{}` at {}",
                        source[*counter], *counter
                    ))
                }
            }

            inner
        }
        _ => anyhow::bail!(format!(
            "unknown token `{}` at {}",
            source[*counter], *counter
        )),
    };

    *counter += 1;
    Ok(result)
}

fn parse_bool(source: &[char], counter: &mut usize) -> anyhow::Result<bool> {
    let result = match source[*counter] {
        't' => {
            if source.iter().skip(*counter).take(4).collect::<String>() == "true" {
                *counter += 4;
                true
            } else {
                anyhow::bail!(format!(
                    "unknown token `{}` at {}",
                    source[*counter], *counter
                ))
            }
        }
        'f' if source.iter().skip(*counter).take(5).collect::<String>() == "false" => {
            *counter += 5;
            false
        }
        _ => {
            anyhow::bail!(format!(
                "unknown token `{}` at {}",
                source[*counter], *counter
            ))
        }
    };

    Ok(result)
}

fn parse_null(source: &[char], counter: &mut usize) -> anyhow::Result<()> {
    match source[*counter] {
        'n' if source.iter().skip(*counter).take(4).collect::<String>() == "null" => {
            *counter += 4;
            true
        }
        _ => {
            anyhow::bail!(format!(
                "unknown token `{}` at {}",
                source[*counter], *counter
            ))
        }
    };

    Ok(())
}

fn parse_whitespaces_and_line_breaks(source: &[char], counter: &mut usize) -> anyhow::Result<()> {
    while *counter < source.len() {
        if !(source[*counter] == ' ' || source[*counter] == '\n') {
            break;
        }
        *counter += 1;
        continue;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{Primitive, parse};

    #[test]
    fn parse_very_simple() {
        let result = parse(&String::from("{}").chars().collect::<Vec<_>>(), &mut 0usize).unwrap();

        assert_eq!(Primitive::Object(HashMap::new()), result)
    }

    #[test]
    fn parse_simple_object() {
        let fixture = r#"{"key": "value"}"#;
        let mut expected_hash = HashMap::new();
        expected_hash.insert("key".into(), Primitive::String("value".into()));
        let expected = Primitive::Object(expected_hash);

        assert_eq!(
            expected,
            parse(&fixture.chars().collect::<Vec<_>>(), &mut 0usize).unwrap()
        )
    }

    #[test]
    fn parse_simple_object_with_whitespaces() {
        let fixture = r#"

            {"key": "value"}"#;
        let mut expected_hash = HashMap::new();
        expected_hash.insert("key".into(), Primitive::String("value".into()));
        let expected = Primitive::Object(expected_hash);

        assert_eq!(
            expected,
            parse(&fixture.chars().collect::<Vec<_>>(), &mut 0usize).unwrap()
        )
    }

    #[test]
    fn parse_object_string() {
        let fixture = r#"{"key": "value", "key2": "value2"}"#;

        let mut expected_hash = HashMap::new();
        expected_hash.insert("key".into(), Primitive::String("value".into()));
        expected_hash.insert("key2".into(), Primitive::String("value2".into()));

        let expected = Primitive::Object(expected_hash);

        assert_eq!(
            expected,
            parse(&fixture.chars().collect::<Vec<_>>(), &mut 0usize).unwrap()
        )
    }

    #[test]
    fn parse_object_number() {
        let fixture = r#"{"key": 1, "key2": 10.1e-10}"#;

        let mut expected_hash = HashMap::new();
        expected_hash.insert("key".into(), Primitive::Number(1.into()));
        expected_hash.insert("key2".into(), Primitive::Number(10.1e-10.into()));

        let expected = Primitive::Object(expected_hash);

        assert_eq!(
            expected,
            parse(&fixture.chars().collect::<Vec<_>>(), &mut 0usize).unwrap()
        )
    }

    #[test]
    fn parse_object_bool() {
        let fixture = r#"{"key": true, "key2": false}"#;

        let mut expected_hash = HashMap::new();
        expected_hash.insert("key".into(), Primitive::Bool(true));
        expected_hash.insert("key2".into(), Primitive::Bool(false));

        let expected = Primitive::Object(expected_hash);

        assert_eq!(
            expected,
            parse(&fixture.chars().collect::<Vec<_>>(), &mut 0usize).unwrap()
        )
    }

    #[test]
    fn parse_object_null() {
        let fixture = r#"{"key": null}"#;

        let mut expected_hash = HashMap::new();
        expected_hash.insert("key".into(), Primitive::Null);

        let expected = Primitive::Object(expected_hash);

        assert_eq!(
            expected,
            parse(&fixture.chars().collect::<Vec<_>>(), &mut 0usize).unwrap()
        )
    }

    #[test]
    fn parse_array() {
        let fixture = r#"{"key": ["simple", "array"]}"#;

        let mut expected_hash = HashMap::new();
        expected_hash.insert(
            "key".into(),
            Primitive::Array(vec![
                Primitive::String("simple".into()),
                Primitive::String("array".into()),
            ]),
        );

        let expected = Primitive::Object(expected_hash);

        assert_eq!(
            expected,
            parse(&fixture.chars().collect::<Vec<_>>(), &mut 0usize).unwrap()
        )
    }

    #[test]
    fn parse_array_extra_padding() {
        let fixture = r#"{"key" :  [ "simple" , "array" ] }"#;

        let mut expected_hash = HashMap::new();
        expected_hash.insert(
            "key".into(),
            Primitive::Array(vec![
                Primitive::String("simple".into()),
                Primitive::String("array".into()),
            ]),
        );

        let expected = Primitive::Object(expected_hash);

        assert_eq!(
            expected,
            parse(&fixture.chars().collect::<Vec<_>>(), &mut 0usize).unwrap()
        )
    }

    #[test]
    fn parse_escaped_string() {
        // U+3042 = "あ"
        let fixture = r#"{"key": "v\nalue", "key \"\\ \u3042": "value2"}"#;

        let mut expected_hash = HashMap::new();
        expected_hash.insert("key".into(), Primitive::String("v\nalue".into()));
        expected_hash.insert("key \"\\ あ".into(), Primitive::String("value2".into()));

        let expected = Primitive::Object(expected_hash);

        assert_eq!(
            expected,
            parse(&fixture.chars().collect::<Vec<_>>(), &mut 0usize).unwrap()
        )
    }

    #[test]
    fn parse_unfinished_object() {
        let fixture = r#"{"#;

        assert_eq!(
            "unexpected end-of-input while parsing object",
            parse(&fixture.chars().collect::<Vec<_>>(), &mut 0usize)
                .unwrap_err()
                .to_string()
        )
    }

    #[test]
    fn parse_unfinished_object_complex() {
        let fixture = r#"{"key": "value", "key2": """#;

        assert_eq!(
            "unexpected end-of-input while parsing object",
            parse(&fixture.chars().collect::<Vec<_>>(), &mut 0usize)
                .unwrap_err()
                .to_string()
        )
    }

    #[test]
    fn parse_unfinished_string() {
        let fixture = r#"{"key": "value}"#;

        assert_eq!(
            "unexpected end-of-input while parsing string",
            parse(&fixture.chars().collect::<Vec<_>>(), &mut 0usize)
                .unwrap_err()
                .to_string()
        )
    }

    #[test]
    fn parse_unfinished_array() {
        let fixture = r#"{"key": [1, 2, 3}"#;

        assert_eq!(
            "unknown token `}` at 16",
            parse(&fixture.chars().collect::<Vec<_>>(), &mut 0usize)
                .unwrap_err()
                .to_string()
        )
    }
}
