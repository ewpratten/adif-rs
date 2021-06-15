use std::{collections::HashMap, fmt::Display};

use chrono::{Datelike, Timelike, Utc};

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
    Date(chrono::Date<Utc>),

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

    pub fn serialize(&self, key_name: String) -> Result<String, SerializeError> {
        // Convert enum value into a usable string
        let value: Result<String, SerializeError> = match self {
            AdifType::Str(val) => {
                // String cannot contain linebreaks, and must be ASCII
                if val.contains('\n') {
                    return Err(SerializeError {
                        message: "String cannot contain linebreaks".to_string(),
                        offender: val.clone(),
                    });
                }
                if !val.is_ascii() {
                    return Err(SerializeError {
                        message: "String must be ASCII".to_string(),
                        offender: val.clone(),
                    });
                }

                Ok(val.clone())
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
        let value = value?;

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
pub type AdifRecord = HashMap<String, AdifType>;

#[cfg(test)]
mod types_tests {
    use chrono::{Date, NaiveDate, NaiveTime};

    use super::*;

    #[test]
    pub fn test_ser_string() {
        assert_eq!(
            AdifType::Str("Hello, world!".to_string())
                .serialize("test".to_string())
                .unwrap(),
            "<TEST:13>Hello, world!".to_string()
        );
    }

    #[test]
    pub fn test_ser_bool() {
        assert_eq!(
            AdifType::Boolean(true)
                .serialize("test".to_string())
                .unwrap(),
            "<TEST:1:B>Y".to_string()
        );
        assert_eq!(
            AdifType::Boolean(false)
                .serialize("test".to_string())
                .unwrap(),
            "<TEST:1:B>N".to_string()
        );
    }

    #[test]
    pub fn test_ser_num() {
        assert_eq!(
            AdifType::Number(3.5).serialize("test".to_string()).unwrap(),
            "<TEST:3:N>3.5".to_string()
        );
        assert_eq!(
            AdifType::Number(-3.5)
                .serialize("test".to_string())
                .unwrap(),
            "<TEST:4:N>-3.5".to_string()
        );
        assert_eq!(
            AdifType::Number(-12.0)
                .serialize("test".to_string())
                .unwrap(),
            "<TEST:3:N>-12".to_string()
        );
    }

    #[test]
    pub fn test_ser_date() {
        assert_eq!(
            AdifType::Date(Date::from_utc(NaiveDate::from_ymd(2020, 2, 24), Utc))
                .serialize("test".to_string())
                .unwrap(),
            "<TEST:8:D>20200224".to_string()
        );
        assert!(
            AdifType::Date(Date::from_utc(NaiveDate::from_ymd(1910, 2, 2), Utc))
                .serialize("test".to_string()).is_err()
        );
    }

    #[test]
    pub fn test_ser_time() {
        assert_eq!(
            AdifType::Time(NaiveTime::from_hms(23, 2, 5))
                .serialize("test".to_string())
                .unwrap(),
            "<TEST:6:T>230205".to_string()
        );
    }
}
