use chrono::{Date, NaiveDate, NaiveTime, Utc};
use indexmap::IndexMap;
use regex::Regex;

use crate::data::{AdifFile, AdifHeader, AdifRecord, AdifType};

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
            value: cap[4].trim_end().to_string(),
        })
        .collect()
}

fn create_token_map(tokens: Vec<Token>) -> IndexMap<String, AdifType> {
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
                        lexical::parse(token.value.to_string())
                            .expect("Found a number value that cannot be parsed"),
                    ),
                    'D' => AdifType::Date(Date::from_utc(
                        NaiveDate::parse_from_str(token.value.as_str(), "%Y%m%d").unwrap(),
                        Utc,
                    )),
                    'T' => AdifType::Time(
                        NaiveTime::parse_from_str(token.value.as_str(), "%H%M%S").unwrap(),
                    ),
                    _ => AdifType::Str(token.value),
                },
                None => AdifType::Str(token.value),
            },
        );
    }
    map
}

fn parse_tokens_to_record(tokens: Vec<Token>) -> AdifRecord {
    create_token_map(tokens).into()
}

fn parse_tokens_to_header(tokens: Vec<Token>) -> AdifHeader {
    create_token_map(tokens).into()
}

/// Parse the contents of an ADIF (`.adi`) file into a struct representation
pub fn parse_adif(data: &str) -> AdifFile {
    // Clean up EOH and EOR tokens
    let data = data.replace("<eoh>", "<EOH>").replace("<eor>", "<EOR>");
    let data = data.split("<EOH>");
    let data = data.collect::<Vec<&str>>();

    // Split file into a header and body
    let header_raw = data.first().unwrap_or(&"");
    let body_raw = data.last().unwrap_or(&"");

    // Parse the header
    let header_tokens = parse_line_to_tokens(&header_raw);
    let header = parse_tokens_to_header(header_tokens);

    // Create the file
    let file = AdifFile {
        header,
        body: body_raw
            .split("<EOR>")
            .collect::<Vec<&str>>()
            .iter()
            .map(|record_line| {
                // Parse the record
                let record_tokens = parse_line_to_tokens(&record_line);
                parse_tokens_to_record(record_tokens)
            })
            .collect(),
    };

    // Return
    file
}

#[cfg(test)]
mod tokenization_tests {
    use super::*;

    #[test]
    pub fn test_line_to_tokens() {
        let result = parse_line_to_tokens(
            "<CALL:4>VA3ZZA <BAND:3>40m <MODE:2>CW <NAME:12>Evan Pratten <eor>",
        );

        assert_eq!(result.len(), 4);
        assert_eq!(result[0].key, "CALL");
        assert_eq!(result[0].value, "VA3ZZA");
        assert_eq!(result[3].key, "NAME");
        assert_eq!(result[3].value, "Evan Pratten");
    }

    #[test]
    pub fn test_tokens_to_record() {
        let tokens = parse_line_to_tokens("<CALL:4>VA3ZZA<A_NUMBER:3:N>401<BOOL:1:B>N<eor>");
        let record = parse_tokens_to_record(tokens);

        assert_eq!(
            record.get("CALL"),
            Some(&AdifType::Str("VA3ZZA".to_string()))
        );
        assert_eq!(record.get("A_NUMBER"), Some(&AdifType::Number(401.0)));
        assert_eq!(record.get("BOOL"), Some(&AdifType::Boolean(false)));
    }
}
