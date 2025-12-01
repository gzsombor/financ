use calamine::Data;
use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::{Decimal, prelude::FromPrimitive};

// Format yyyy.mm.dd.
pub fn cell_to_date(cell: &Data) -> Option<NaiveDate> {
    cell_to_date_raw(cell, "%Y.%m.%d.")
}

// Format yyyy-mm-dd
pub fn cell_to_iso_date(cell: &Data) -> Option<NaiveDate> {
    cell_to_date_raw(cell, "%Y-%m-%d")
}

// Format dd.mm.yyyy
pub fn cell_to_german_date(cell: &Data) -> Option<NaiveDate> {
    cell_to_date_raw(cell, "%d.%m.%Y")
}

// Format dd-mm-yyyy
pub fn cell_to_english_date(cell: &Data) -> Option<NaiveDate> {
    cell_to_date_raw(cell, "%d-%m-%Y")
}

pub fn cell_to_date_raw(cell: &Data, format: &str) -> Option<NaiveDate> {
    match cell {
        Data::String(s) => {
            let owned_str = s.trim().to_owned();
            NaiveDate::parse_from_str(&owned_str, format).ok()
        }
        Data::DateTime(date_time) => {
            if date_time.is_datetime() {
                date_time.as_datetime().map(|date_time| date_time.date())
            } else {
                None
            }
        }
        Data::DateTimeIso(date_time) => NaiveDate::parse_from_str(date_time, "%Y-%m-%d").ok(),
        _ => None,
    }
}

pub fn cell_to_datetime(cell: &Data) -> Option<NaiveDateTime> {
    match cell {
        Data::String(str) => NaiveDateTime::parse_from_str(str, "%Y.%m.%d. %H:%M:%S").ok(),
        Data::DateTime(date_time) => {
            if date_time.is_datetime() {
                date_time.as_datetime()
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn cell_to_string(cell: &Data) -> Option<String> {
    if let Data::String(str) = cell {
        if str.is_empty() {
            None
        } else {
            Some(str.clone())
        }
    } else {
        None
    }
}

pub fn cell_to_decimal(cell: &Data) -> Option<Decimal> {
    match cell {
        Data::Float(flt) => Decimal::from_f64(*flt),
        Data::Int(numb) => Decimal::from_i64(*numb),
        Data::String(string) => Decimal::from_scientific(string).ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use calamine::{Data, ExcelDateTime, ExcelDateTimeType};
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

    #[test]
    fn test_cell_to_date() {
        let cell = Data::String("2023.01.15.".to_string());
        let date = cell_to_date(&cell);
        assert_eq!(date, Some(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap()));
    }

    #[test]
    fn test_cell_to_iso_date() {
        let cell = Data::String("2023-01-15".to_string());
        let date = cell_to_iso_date(&cell);
        assert_eq!(date, Some(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap()));
    }

    #[test]
    fn test_cell_to_german_date() {
        let cell = Data::String("15.01.2023".to_string());
        let date = cell_to_german_date(&cell);
        assert_eq!(date, Some(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap()));
    }

    #[test]
    fn test_cell_to_english_date() {
        let cell = Data::String("15-01-2023".to_string());
        let date = cell_to_english_date(&cell);
        assert_eq!(date, Some(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap()));
    }

    #[test]
    fn test_cell_to_datetime() {
        let cell = Data::String("2023.01.15. 12:30:45".to_string());
        let datetime = cell_to_datetime(&cell);
        assert_eq!(
            datetime,
            Some(
                NaiveDateTime::parse_from_str("2023.01.15. 12:30:45", "%Y.%m.%d. %H:%M:%S")
                    .unwrap()
            )
        );
    }

    #[test]
    fn test_cell_to_string() {
        let cell = Data::String("Hello, world!".to_string());
        let string = cell_to_string(&cell);
        assert_eq!(string, Some("Hello, world!".to_string()));
    }

    #[test]
    fn test_cell_to_decimal() {
        let cell = Data::Float(123.45);
        let decimal = cell_to_decimal(&cell);
        assert_eq!(decimal, Some(Decimal::from_f64(123.45).unwrap()));
    }

    #[test]
    fn test_naive_date_parse() {
        let date_str = "2023-01-15";
        let format = "%Y-%m-%d";
        let parsed_date = NaiveDate::parse_from_str(date_str, format);
        assert!(
            parsed_date.is_ok(),
            "Failed to parse date: {:?}",
            parsed_date.err()
        );
        assert_eq!(
            parsed_date.unwrap(),
            NaiveDate::from_ymd_opt(2023, 1, 15).unwrap()
        );
    }

    #[test]
    fn test_cell_to_datetime_from_float() {
        let cell = Data::Float(44060.0);
        let parsed_datetime = cell_to_datetime(&cell);
        assert_eq!(parsed_datetime, None);
    }

    #[test]
    fn test_cell_to_datetime_from_datetime() {
        // Test with DateTime input
        let cell = Data::DateTime(ExcelDateTime::new(
            44060.0,
            ExcelDateTimeType::DateTime,
            false,
        ));
        let parsed_datetime = cell_to_datetime(&cell);
        assert_eq!(
            parsed_datetime,
            Some(NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2020, 8, 17).unwrap(),
                NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            ))
        );
    }

    #[test]
    fn test_cell_to_date_raw_edge_cases() {
        // Test with empty string
        let empty_cell = Data::String("".to_string());
        let empty_date = cell_to_date_raw(&empty_cell, "%Y-%m-%d");
        assert_eq!(empty_date, None);

        // Test with malformed date string
        let malformed_cell = Data::String("2023/01/15".to_string());
        let malformed_date = cell_to_date_raw(&malformed_cell, "%Y-%m-%d");
        assert_eq!(malformed_date, None);

        // Test with non-date string
        let non_date_cell = Data::String("not a date".to_string());
        let non_date = cell_to_date_raw(&non_date_cell, "%Y-%m-%d");
        assert_eq!(non_date, None);
    }

    #[test]
    fn test_cell_to_datetime_edge_cases() {
        // Test with empty string
        let empty_cell = Data::String("".to_string());
        let empty_datetime = cell_to_datetime(&empty_cell);
        assert_eq!(empty_datetime, None);

        // Test with malformed datetime string
        let malformed_cell = Data::String("2023/01/15 12:30:45".to_string());
        let malformed_datetime = cell_to_datetime(&malformed_cell);
        assert_eq!(malformed_datetime, None);

        // Test with non-datetime string
        let non_datetime_cell = Data::String("not a datetime".to_string());
        let non_datetime = cell_to_datetime(&non_datetime_cell);
        assert_eq!(non_datetime, None);
    }

    #[test]
    fn test_cell_to_date_from_datetime_iso() {
        let cell = Data::DateTimeIso("2023-01-15".to_string());
        let date = cell_to_date(&cell);
        assert_eq!(date, Some(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap()));
    }

    #[test]
    fn test_cell_to_iso_date_from_datetime_iso() {
        let cell = Data::DateTimeIso("2023-01-15".to_string());
        let date = cell_to_iso_date(&cell);
        assert_eq!(date, Some(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap()));
    }

    #[test]
    fn test_cell_to_german_date_from_datetime_iso() {
        let cell = Data::DateTimeIso("2023-01-15".to_string());
        let date = cell_to_german_date(&cell);
        assert_eq!(date, Some(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap()));
    }

    #[test]
    fn test_cell_to_english_date_from_datetime_iso() {
        let cell = Data::DateTimeIso("2023-01-15".to_string());
        let date = cell_to_english_date(&cell);
        assert_eq!(date, Some(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap()));
    }

    #[test]
    fn test_cell_to_date_from_datetime() {
        let cell = Data::DateTime(ExcelDateTime::new(
            44060.0,
            ExcelDateTimeType::DateTime,
            false,
        ));
        let date = cell_to_date(&cell);
        assert_eq!(date, Some(NaiveDate::from_ymd_opt(2020, 8, 17).unwrap()));
    }

    #[test]
    fn test_cell_to_iso_date_from_datetime() {
        let cell = Data::DateTime(ExcelDateTime::new(
            44060.0,
            ExcelDateTimeType::DateTime,
            false,
        ));
        let date = cell_to_iso_date(&cell);
        assert_eq!(date, Some(NaiveDate::from_ymd_opt(2020, 8, 17).unwrap()));
    }

    #[test]
    fn test_cell_to_german_date_from_datetime() {
        let cell = Data::DateTime(ExcelDateTime::new(
            44060.0,
            ExcelDateTimeType::DateTime,
            false,
        ));
        let date = cell_to_german_date(&cell);
        assert_eq!(date, Some(NaiveDate::from_ymd_opt(2020, 8, 17).unwrap()));
    }

    #[test]
    fn test_cell_to_english_date_from_datetime() {
        let cell = Data::DateTime(ExcelDateTime::new(
            44060.0,
            ExcelDateTimeType::DateTime,
            false,
        ));
        let date = cell_to_english_date(&cell);
        assert_eq!(date, Some(NaiveDate::from_ymd_opt(2020, 8, 17).unwrap()));
    }
}
