use std::{collections::HashMap, fmt::Display};

use chrono::{Datelike, Timelike, Utc};

#[derive(Debug)]
pub struct SerializeError<'a> {
    pub message: &'a str,
    pub offender: String,
}

impl<'a> Display for SerializeError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}. Offending value: {}", self.message, self.offender)
    }
}

/// Supported datatypes for representing ADIF data
#[derive(Debug, Clone, PartialEq)]
pub enum AdifType<'a> {
    /// Basic string type
    Str(&'a str),

    /// Basic boolean type
    Boolean(bool),

    /// Basic number type
    Number(f64),

    /// 8 Digits representing a UTC date in `YYYYMMDD` format, where
    ///  - YYYY is a 4-Digit year specifier, where 1930 <= YYYY
    ///  - MM is a 2-Digit month specifier, where 1 <= MM <= 12
    ///  - DD is a 2-Digit day specifier, where 1 <= DD <= DaysInMonth(MM)
    Date(chrono::Date<Utc>),

    /// 6 Digits representing a UTC time in HHMMSS format
    /// or 4 Digits representing a time in HHMM format, where:
    ///  - HH is a 2-Digit hour specifier, where 0 <= HH <= 23
    ///  - MM is a 2-Digit minute specifier, where 0 <= MM <= 59
    ///  - SS is a 2-Digit second specifier, where 0 <= SS <= 59
    Time(chrono::NaiveTime),
}

impl<'a> AdifType<'a> {
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
                        message: "String cannot contain linebreaks",
                        offender: val.to_string(),
                    });
                }
                if !val.is_ascii() {
                    return Err(SerializeError {
                        message: "String must be ASCII",
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
                        message: "Date must be >= 1930",
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

/// A single ADIF record, consisting of many values
pub struct AdifRecord<'a>(HashMap<String, AdifType<'a>>);

impl<'a> AdifRecord<'a> {
    /// Serialize into a full record string
    pub fn serialize(&self) -> Result<String, SerializeError> {
        let mut output = self
            .0
            .iter()
            .map(|(key, value)| value.serialize(&key))
            .collect::<Result<Vec<String>, SerializeError>>()?
            .join("");
        output.push_str("<eor>");
        Ok(output)
    }
}

impl<'a> From<HashMap<String, AdifType<'a>>> for AdifRecord<'a> {
    fn from(map: HashMap<String, AdifType<'a>>) -> Self {
        Self { 0: map }
    }
}

impl<'a> From<HashMap<&'a str, AdifType<'a>>> for AdifRecord<'a> {
    fn from(map: HashMap<&'a str, AdifType<'a>>) -> Self {
        Self {
            0: map
                .iter()
                .map(|(key, value)| (key.to_string(), value.clone()))
                .collect(),
        }
    }
}

#[cfg(test)]
mod types_tests {
    use chrono::{Date, NaiveDate, NaiveTime};

    use super::*;

    #[test]
    pub fn test_ser_string() {
        assert_eq!(
            AdifType::Str("Hello, world!").serialize("test").unwrap(),
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
            AdifType::Date(Date::from_utc(NaiveDate::from_ymd(2020, 2, 24), Utc))
                .serialize("test")
                .unwrap(),
            "<TEST:8:D>20200224"
        );
        assert!(
            AdifType::Date(Date::from_utc(NaiveDate::from_ymd(1910, 2, 2), Utc))
                .serialize("test")
                .is_err()
        );
    }

    #[test]
    pub fn test_ser_time() {
        assert_eq!(
            AdifType::Time(NaiveTime::from_hms(23, 2, 5))
                .serialize("test")
                .unwrap(),
            "<TEST:6:T>230205"
        );
    }
}

#[cfg(test)]
mod record_tests {
    use maplit::hashmap;

    use super::*;

    #[test]
    pub fn test_ser_record() {
        let test_record: AdifRecord = hashmap! {
            "test string" => AdifType::Str("Heyo rusty friends!"),
            "a number" => AdifType::Number(15.5)
        }
        .into();

        assert_eq!(
            test_record.serialize().unwrap(),
            "<TEST_STRING:19>Heyo rusty friends!<A_NUMBER:4:N>15.5<eor>"
        );
    }
}
