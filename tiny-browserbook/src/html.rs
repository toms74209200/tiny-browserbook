use crate::dom::AttrMap;
use combine::between;
use combine::error::ParseError;
use combine::many;
use combine::parser::char::char;
use combine::parser::char::letter;
use combine::parser::char::newline;
use combine::parser::char::space;
use combine::satisfy;
use combine::sep_by;
use combine::{many1, Parser, Stream};

fn attribute<Input>() -> impl Parser<Input, Output = (String, String)>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        many1::<String, _, _>(letter()),
        many::<String, _, _>(space().or(newline())),
        char('='),
        many::<String, _, _>(space().or(newline())),
        between(
            char('"'),
            char('"'),
            many1::<String, _, _>(satisfy(|c: char| c != '"')),
        ),
    )
        .map(|v| (v.0, v.4))
}

fn attributes<Input>() -> impl Parser<Input, Output = AttrMap>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    sep_by::<Vec<(String, String)>, _, _, _>(
        attribute(),
        many::<String, _, _>(space().or(newline())),
    )
    .map(|attrs: Vec<(String, String)>| attrs.into_iter().collect::<AttrMap>())
}

#[cfg(test)]
mod tests {
    use combine::EasyParser;

    use super::*;

    #[test]
    fn test_parse_attribut() {
        assert_eq!(
            attribute().parse("test=\"foobar\""),
            Ok((("test".to_string(), "foobar".to_string()), ""))
        );
    }

    #[test]
    fn test_parse_attribut_has_space() {
        assert_eq!(
            attribute().parse("test = \"foobar\""),
            Ok((("test".to_string(), "foobar".to_string()), ""))
        );
    }

    #[test]
    fn test_parse_attributes() {
        let mut expected_map = AttrMap::new();
        expected_map.insert("test".to_string(), "foobar".to_string());
        expected_map.insert("abc".to_string(), "def".to_string());
        assert_eq!(
            attributes().easy_parse("test=\"foobar\" abc=\"def\""),
            Ok((expected_map, ""))
        )
    }

    #[test]
    fn test_parse_non_attributes() {
        assert_eq!(attributes().easy_parse(""), Ok((AttrMap::new(), "")))
    }
}
