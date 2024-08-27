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

fn open_tag<Input>() -> impl Parser<Input, Output = (String, AttrMap)>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let open_tag_name = many1::<String, _, _>(letter());
    let open_tag_content = (
        open_tag_name,
        many::<String, _, _>(space().or(newline())),
        attributes(),
    )
        .map(|v: (String, _, AttrMap)| (v.0, v.2));
    between(char('<'), char('>'), open_tag_content)
}

fn close_tag<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let close_tag_name = many1::<String, _, _>(letter());
    let close_tag_content = (char('/'), close_tag_name).map(|v| v.1);
    between(char('<'), char('>'), close_tag_content)
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

    #[test]
    fn test_parse_open_tag() {
        assert_eq!(
            open_tag().easy_parse("<p>aaaa"),
            Ok((("p".to_string(), AttrMap::new()), "aaaa"))
        );
    }
    #[test]
    fn test_parse_open_tag_has_an_attribute() {
        let mut attributes = AttrMap::new();
        attributes.insert("id".to_string(), "test".to_string());
        assert_eq!(
            open_tag().easy_parse("<p id=\"test\">"),
            Ok((("p".to_string(), attributes), ""))
        )
    }
    #[test]
    fn test_parse_open_tag_has_attributes() {
        let result = open_tag().easy_parse("<p id=\"test\" class=\"sample\">");
        let mut attributes = AttrMap::new();
        attributes.insert("id".to_string(), "test".to_string());
        attributes.insert("class".to_string(), "sample".to_string());
        assert_eq!(result, Ok((("p".to_string(), attributes), "")));
    }

    #[test]
    fn test_parse_open_tag_invalid() {
        assert!(open_tag().easy_parse("<p id>").is_err());
    }

    #[test]
    fn test_parse_close_tag() {
        let result = close_tag().parse("</p>");
        assert_eq!(result, Ok(("p".to_string(), "")));
    }
}
