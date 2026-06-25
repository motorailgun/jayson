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
    *counter += 1;

    while *counter <= source.len() && source[*counter] != '"' {
        res.push(source[*counter]);
        *counter += 1;
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
    while *counter <= strlen && source[*counter] != '}' {
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
                if source[*counter] != ',' {
                    break;
                } else {
                    *counter += 1;
                }
            }
        }
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
            while *counter <= strlen && source[*counter] != ']' {
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
            *counter += 4;
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
    while *counter <= source.len() && source[*counter] == ' ' || source[*counter] == '\n' {
        *counter += 1;
        continue;
    }

    if source.len() < *counter {
        anyhow::bail!("unknown token end with whitespaces")
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
}
