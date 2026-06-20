use std::collections::HashMap;

fn main() {
    let _ = dbg!(parse(&"{}".to_string().chars().collect(), &mut 0usize));
}

#[derive(Debug, PartialEq)]
enum Primitive {
    String(String),
    Number(f64),
    Array(Vec<Primitive>),
    Object(HashMap<String, Primitive>),
}

fn parse(source: &Vec<char>, counter: &mut usize) -> anyhow::Result<Primitive> {
    let strlen = source.len();

    while *counter <= strlen {
        match source[*counter] {
            '"' => {
                let result = parse_string(source, counter)?;
                return Ok(Primitive::String(result));
            }
            '{' => {
                let result = parse_object(source, counter)?;
                return Ok(Primitive::Object(result));
            }
            some_unknown_char => {
                anyhow::bail!(format!(
                    "unknown token `{}` at {}",
                    some_unknown_char, counter
                ))
            }
        }
    }

    Ok(Primitive::Object(HashMap::new()))
}

fn parse_string(source: &Vec<char>, counter: &mut usize) -> anyhow::Result<String> {
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
    return Ok(res);
}

fn parse_object(
    source: &Vec<char>,
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

fn parse_whitespaces_and_line_breaks(
    source: &Vec<char>,
    counter: &mut usize,
) -> anyhow::Result<()> {
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
        let result = parse(&String::from("{}").chars().collect(), &mut 0usize).unwrap();

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
            parse(&fixture.chars().collect(), &mut 0usize).unwrap()
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
            parse(&fixture.chars().collect(), &mut 0usize).unwrap()
        )
    }
}
