use nom::{
    branch::alt,
    bytes::complete::{take_while1},
    character::complete::{char},
    combinator::{map},
    multi::separated_list1,
    sequence::preceded,
    IResult,
};
use crate::{ArrayIndex, PathElement};

fn parse_escaped_underscore(input: &str) -> IResult<&str, PathElement> {
    map(
        preceded(char('\\'), char('_')),
        |_| PathElement::Key("_".to_string())
    )(input)
}

fn parse_element(input: &str) -> IResult<&str, PathElement> {
    alt((
        parse_escaped_underscore,
        map(
            take_while1(|c: char| c.is_alphanumeric() || c == '_'),
            |s: &str| {
                if s == "_" {
                    PathElement::ArrayIndex(ArrayIndex::All)
                } else {
                    PathElement::Key(s.to_string())
                }
            }
        ),
    ))(input)
}

fn parse_path(input: &str) -> IResult<&str, Vec<PathElement>> {
    separated_list1(char('.'), parse_element)(input)
}

pub(crate) fn parse_element_path(input: &str) -> Result<Vec<PathElement>, String> {
    match parse_path(input) {
        Ok((remaining, result)) => {
            if remaining.is_empty() {
                Ok(result)
            } else {
                Err(format!("Unparsed input remaining: {}", remaining))
            }
        }
        Err(e) => Err(format!("Parsing error: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_parsing() {
        assert_eq!(
            parse_element_path("a.b.c").unwrap(),
            vec![
                PathElement::Key("a".to_string()),
                PathElement::Key("b".to_string()),
                PathElement::Key("c".to_string())
            ]
        );

        assert_eq!(
            parse_element_path("_.a.c").unwrap(),
            vec![
                PathElement::ArrayIndex(ArrayIndex::All),
                PathElement::Key("a".to_string()),
                PathElement::Key("c".to_string())
            ]
        );

        assert_eq!(
            parse_element_path(r"\_.field_1._1").unwrap(),
            vec![
                PathElement::Key("_".to_string()),
                PathElement::Key("field_1".to_string()),
                PathElement::Key("_1".to_string())
            ]
        );

        assert_eq!(
            parse_element_path(r"\_.\_._1_").unwrap(),
            vec![
                PathElement::Key("_".to_string()),
                PathElement::Key("_".to_string()),
                PathElement::Key("_1_".to_string())
            ]
        );

        assert_eq!(
            parse_element_path(r"__").unwrap(),
            vec![
                PathElement::Key("__".to_string()),
            ]
        );
    }
}
