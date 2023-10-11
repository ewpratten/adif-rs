use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use chrono::{Datelike, Timelike};
use indexmap::IndexMap;

#[derive(Debug)]
pub struct SerializeError {
    pub message: String,
    pub offender: String,
}

impl Display for SerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}. Offending value: {}", self.message, self.offender)
    }
}

/// Supported datatypes for representing ADIF data
#[derive(Debug, Clone, PartialEq)]
pub enum AdifType {
    /// Basic string type
    Str(String),

    /// Basic boolean type
    Boolean(bool),

    /// Basic number type
    Number(f64),

    /// 8 Digits representing a UTC date in `YYYYMMDD` format, where
    ///  - YYYY is a 4-Digit year specifier, where 1930 <= YYYY
    ///  - MM is a 2-Digit month specifier, where 1 <= MM <= 12
    ///  - DD is a 2-Digit day specifier, where 1 <= DD <= DaysInMonth(MM)
    Date(chrono::NaiveDate),

    /// 6 Digits representing a UTC time in HHMMSS format
    /// or 4 Digits representing a time in HHMM format, where:
    ///  - HH is a 2-Digit hour specifier, where 0 <= HH <= 23
    ///  - MM is a 2-Digit minute specifier, where 0 <= MM <= 59
    ///  - SS is a 2-Digit second specifier, where 0 <= SS <= 59
    Time(chrono::NaiveTime),
}

impl AdifType {
    /// Get the single-char indicator used to specify a type
    pub fn get_data_type_indicator(&self) -> Option<char> {
        match self {
            AdifType::Str(_) => None,
            AdifType::Boolean(_) => Some('B'),
            AdifType::Number(_) => Some('N'),
            AdifType::Date(_) => Some('D'),
            AdifType::Time(_) => Some('T'),
        }
    }

    /// Serialize into a single value
    pub fn serialize(&self, key_name: &str) -> Result<String, SerializeError> {
        // Convert enum value into a usable string
        let value: Result<String, SerializeError> = match self {
            AdifType::Str(val) => {
                // String cannot contain linebreaks, and must be ASCII
                if val.contains('\n') {
                    return Err(SerializeError {
                        message: "String cannot contain linebreaks".to_string(),
                        offender: val.to_string(),
                    });
                }
                if !val.is_ascii() {
                    return Err(SerializeError {
                        message: "String must be ASCII".to_string(),
                        offender: val.to_string(),
                    });
                }

                Ok(val.to_string())
            }
            AdifType::Boolean(val) => Ok(match val {
                true => "Y",
                false => "N",
            }
            .to_string()),
            AdifType::Number(val) => Ok(val.to_string()),
            AdifType::Date(val) => {
                // Date must be after 1929
                if val.year() < 1930 {
                    return Err(SerializeError {
                        message: "Date must be >= 1930".to_string(),
                        offender: val.to_string(),
                    });
                }

                Ok(format!("{}{:02}{:02}", val.year(), val.month(), val.day()))
            }
            AdifType::Time(val) => Ok(format!(
                "{:02}{:02}{:02}",
                val.hour(),
                val.minute(),
                val.second()
            )),
        };
        let value: &str = &(value?);

        // Format the result
        Ok(format!(
            "<{}:{}{}>{}",
            key_name.to_uppercase().replace(' ', "_"),
            value.len(),
            match self.get_data_type_indicator() {
                Some(id) => format!(":{}", id),
                None => String::new(),
            },
            value
        ))
    }
}

impl Display for AdifType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// A single ADIF record, consisting of many values
#[derive(Debug, Clone, PartialEq)]
pub struct AdifRecord(IndexMap<String, AdifType>);

impl AdifRecord {
    /// Serialize into a full record string
    pub fn serialize(&self) -> Result<String, SerializeError> {
        let mut output = self
            .0
            .iter()
            .map(|(key, value)| value.serialize(key))
            .collect::<Result<Vec<String>, SerializeError>>()?
            .join("");
        output.push_str("<eor>");
        Ok(output)
    }
}

impl From<IndexMap<String, AdifType>> for AdifRecord {
    fn from(map: IndexMap<String, AdifType>) -> Self {
        Self(map)
    }
}

impl<'a> From<IndexMap<&'a str, AdifType>> for AdifRecord {
    fn from(map: IndexMap<&'a str, AdifType>) -> Self {
        Self(
            map.iter()
                .map(|(key, value)| (key.to_string(), value.clone()))
                .collect(),
        )
    }
}

impl Display for AdifRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.serialize())
    }
}

impl Deref for AdifRecord {
    type Target = IndexMap<String, AdifType>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AdifRecord {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// An ADIF file header, consisting of many values
#[derive(Debug, Clone, PartialEq)]
pub struct AdifHeader(IndexMap<String, AdifType>);

impl AdifHeader {
    /// Serialize into a full header string
    pub fn serialize(&self) -> Result<String, SerializeError> {
        let mut output = String::new();
        output.push_str(&format!(
            "Generated {} (UTC)\n\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
        ));

        let header_tags = self
            .0
            .iter()
            .map(|(key, value)| value.serialize(key))
            .collect::<Result<Vec<String>, SerializeError>>()?
            .join("\n");
        output.push_str(&header_tags);

        output.push('\n');
        output.push_str("<EOH>");

        Ok(output)
    }
}

impl From<IndexMap<String, AdifType>> for AdifHeader {
    fn from(map: IndexMap<String, AdifType>) -> Self {
        Self(map)
    }
}

impl<'a> From<IndexMap<&'a str, AdifType>> for AdifHeader {
    fn from(map: IndexMap<&'a str, AdifType>) -> Self {
        Self(
            map.iter()
                .map(|(key, value)| (key.to_string(), value.clone()))
                .collect(),
        )
    }
}

impl Display for AdifHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.serialize())
    }
}

impl Deref for AdifHeader {
    type Target = IndexMap<String, AdifType>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AdifHeader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Defines an entire file of ADIF data
#[derive(Debug, Clone, PartialEq)]
pub struct AdifFile {
    pub header: AdifHeader,
    pub body: Vec<AdifRecord>,
}

impl AdifFile {
    /// Serialize into text data to be written to a file
    pub fn serialize(&self) -> Result<String, SerializeError> {
        Ok(format!(
            "{}\n{}",
            self.header.serialize()?,
            self.body
                .iter()
                .map(|record| record.serialize())
                .collect::<Result<Vec<String>, SerializeError>>()?
                .join("\n")
        ))
    }
}

#[cfg(test)]
mod types_tests {
    use chrono::{NaiveDate, NaiveTime};

    use super::*;

    #[test]
    pub fn test_ser_string() {
        assert_eq!(
            AdifType::Str("Hello, world!".to_string())
                .serialize("test")
                .unwrap(),
            "<TEST:13>Hello, world!"
        );
    }

    #[test]
    pub fn test_ser_bool() {
        assert_eq!(
            AdifType::Boolean(true).serialize("test").unwrap(),
            "<TEST:1:B>Y"
        );
        assert_eq!(
            AdifType::Boolean(false).serialize("test").unwrap(),
            "<TEST:1:B>N"
        );
    }

    #[test]
    pub fn test_ser_num() {
        assert_eq!(
            AdifType::Number(3.5).serialize("test").unwrap(),
            "<TEST:3:N>3.5"
        );
        assert_eq!(
            AdifType::Number(-3.5).serialize("test").unwrap(),
            "<TEST:4:N>-3.5"
        );
        assert_eq!(
            AdifType::Number(-12.0).serialize("test").unwrap(),
            "<TEST:3:N>-12"
        );
    }

    #[test]
    pub fn test_ser_date() {
        assert_eq!(
            AdifType::Date(NaiveDate::from_ymd_opt(2020, 2, 24).unwrap())
                .serialize("test")
                .unwrap(),
            "<TEST:8:D>20200224"
        );
        assert!(AdifType::Date(NaiveDate::from_ymd_opt(1910, 2, 2).unwrap())
            .serialize("test")
            .is_err());
    }

    #[test]
    pub fn test_ser_time() {
        assert_eq!(
            AdifType::Time(NaiveTime::from_hms_opt(23, 2, 5).unwrap())
                .serialize("test")
                .unwrap(),
            "<TEST:6:T>230205"
        );
    }
}

#[cfg(test)]
mod record_tests {

    use indexmap::indexmap;

    use super::*;

    #[test]
    pub fn test_ser_record() {
        let test_record: AdifRecord = indexmap! {
            "a number" => AdifType::Number(15.5),
            "test string" => AdifType::Str("Heyo rusty friends!".to_string()),
        }
        .into();

        assert_eq!(
            test_record.serialize().unwrap(),
            "<A_NUMBER:4:N>15.5<TEST_STRING:19>Heyo rusty friends!<eor>"
        );
    }

    #[test]
    pub fn test_ser_header() {
        let test_header: AdifHeader = indexmap! {
            "a number" => AdifType::Number(15.5),
            "test string" => AdifType::Str("Heyo rusty friends!".to_string()),
        }
        .into();

        let serialized_lines = test_header.serialize().unwrap();
        let mut serialized_lines = serialized_lines.lines();

        // Skip the "generated" line
        serialized_lines.next();
        serialized_lines.next();

        // Test the header lines
        assert_eq!(serialized_lines.next(), Some("<A_NUMBER:4:N>15.5"));
        assert_eq!(
            serialized_lines.next(),
            Some("<TEST_STRING:19>Heyo rusty friends!")
        );
        assert_eq!(serialized_lines.next(), Some("<EOH>"));
    }
}
