use crate::{ArrayIndex, PathElement};

pub fn parse_element_path(s: &str) -> Result<Vec<PathElement>, String> {
    if s.is_empty() {
        return Err("Empty path is not allowed".to_string());
    }

    let mut result = Vec::new();
    let mut chars = s.chars().peekable();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut in_brackets = false;

    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                if in_quotes {
                    if current.is_empty() {
                        return Err("Empty quoted string is not allowed".to_string());
                    }
                    result.push(PathElement::Key(current.clone()));
                    current.clear();
                    in_quotes = false;
                } else {
                    if !current.is_empty() {
                        return Err("Unexpected quote".to_string());
                    }
                    in_quotes = true;
                }
            }
            '.' => {
                if in_quotes {
                    current.push(c);
                } else {
                    if !current.is_empty() {
                        result.push(PathElement::Key(current.clone()));
                        current.clear();
                    } else if result.is_empty() {
                        return Err("Path cannot start with a dot".to_string());
                    }
                }
            }
            '[' => {
                if in_quotes {
                    current.push(c);
                } else {
                    if !current.is_empty() {
                        result.push(PathElement::Key(current.clone()));
                        current.clear();
                    }
                    in_brackets = true;
                }
            }
            ']' => {
                if in_quotes {
                    current.push(c);
                } else if in_brackets {
                    if current == "_" {
                        result.push(PathElement::ArrayIndex(ArrayIndex::All));
                    } else {
                        match current.parse::<usize>() {
                            Ok(index) => result.push(PathElement::ArrayIndex(ArrayIndex::Index(index))),
                            Err(_) => return Err(format!("Invalid array index: {}", current)),
                        }
                    }
                    current.clear();
                    in_brackets = false;
                } else {
                    return Err("Unexpected closing bracket".to_string());
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if in_quotes {
        return Err("Unclosed quote".to_string());
    }

    if in_brackets {
        return Err("Unclosed bracket".to_string());
    }

    if !current.is_empty() {
        result.push(PathElement::Key(current));
    }

    if result.is_empty() {
        return Err("Empty path is not allowed".to_string());
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::{ArrayIndex, PathElement};
    use super::*;

    #[test]
    fn test_parse_element_path() {
        assert_eq!(
            parse_element_path("a.b.c").unwrap(),
            vec![
                PathElement::Key("a".to_string()),
                PathElement::Key("b".to_string()),
                PathElement::Key("c".to_string())
            ]
        );

        assert_eq!(
            parse_element_path("[_].a.c").unwrap(),
            vec![
                PathElement::ArrayIndex(ArrayIndex::All),
                PathElement::Key("a".to_string()),
                PathElement::Key("c".to_string())
            ]
        );

        assert_eq!(
            parse_element_path("'[_]'.a").unwrap(),
            vec![
                PathElement::Key("[_]".to_string()),
                PathElement::Key("a".to_string())
            ]
        );

        assert_eq!(
            parse_element_path("a.[1]").unwrap(),
            vec![
                PathElement::Key("a".to_string()),
                PathElement::ArrayIndex(ArrayIndex::Index(1))
            ]
        );

        assert_eq!(
            parse_element_path("a.'.'.b").unwrap(),
            vec![
                PathElement::Key("a".to_string()),
                PathElement::Key(".".to_string()),
                PathElement::Key("b".to_string())
            ]
        );

        assert!(parse_element_path("").is_err());
        assert!(parse_element_path("''").is_err());
        assert!(parse_element_path("a.'").is_err());
        assert!(parse_element_path("a.[").is_err());
        assert!(parse_element_path("a.[x]").is_err());
    }
}
