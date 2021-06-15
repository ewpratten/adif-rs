use chrono::{Date, NaiveDate, NaiveTime, Utc};
use indexmap::IndexMap;
use regex::Regex;

use crate::data::{AdifRecord, AdifType};

const TOKEN_RE: &str = r"(?:<([A-Za-z_]+):(\d+)(?::([A-Za-z]))?>([^<]*))";

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    pub key: String,
    pub len: usize,
    pub ty: Option<char>,
    pub value: String,
}

fn parse_line_to_tokens(line: &str) -> Vec<Token> {
    Regex::new(TOKEN_RE)
        .unwrap()
        .captures_iter(line)
        .map(|cap| Token {
            key: cap[1].to_string().to_uppercase(),
            len: cap[2].parse().expect("Length is not an integer"),
            ty: match cap.get(3) {
                Some(val) => Some(val.as_str().chars().next().unwrap().to_ascii_uppercase()),
                None => None,
            },
            value: cap[4].to_string(),
        })
        .collect()
}

fn parse_tokens_to_record<'a>(tokens: &'a Vec<Token>) -> AdifRecord<'a> {
    // Build a map
    let mut map = IndexMap::new();

    // Handle every token
    for token in tokens {
        map.insert(
            token.key.clone(),
            match token.ty {
                Some(ty) => match ty {
                    'B' => AdifType::Boolean(token.value.to_uppercase() == "Y"),
                    'N' => AdifType::Number(
                        ty.to_string()
                            .parse()
                            .expect("Found a number value that cannot be parsed"),
                    ),
                    'D' => AdifType::Date(Date::from_utc(
                        NaiveDate::parse_from_str(&token.value.clone(), "%Y%m%d").unwrap(),
                        Utc,
                    )),
                    'T' => AdifType::Time(
                        NaiveTime::parse_from_str(&token.value.clone(), "%H%M%S").unwrap(),
                    ),
                    _ => AdifType::Str(token.value.as_str()),
                },
                None => AdifType::Str(token.value.as_str()),
            },
        );
    }

    return map.into();
}

#[cfg(test)]
mod tokenization_tests {
    use super::*;

    #[test]
    pub fn test_line_to_tokens() {
        let result = parse_line_to_tokens("<CALL:4>VA3ZZA<BAND:3>40m<MODE:2>CW<eor>");

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].key, "CALL");
        assert_eq!(result[0].value, "VA3ZZA");
    }
}
