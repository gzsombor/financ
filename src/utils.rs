use anyhow::{Context, Result};
use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use dotenv::dotenv;
use regex::Regex;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::env;
use std::sync::LazyLock;

/// Establishes a connection to the `GnuCash` database.
///
/// # Errors
///
/// Returns an error if the `DATABASE_URL` environment variable is not set or
/// if the connection cannot be established.
pub fn establish_connection() -> Result<SqliteConnection> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    SqliteConnection::establish(&database_url)
        .with_context(|| format!("Error connecting to {database_url}"))
}

#[must_use]
pub fn to_date(date_string: Option<String>) -> Option<NaiveDate> {
    date_string.and_then(|x| NaiveDate::parse_from_str(x.as_ref(), "%Y-%m-%d").ok())
}

#[must_use]
pub fn to_string(date: Option<NaiveDate>) -> String {
    date.map_or_else(String::new, |dt| dt.format("%Y-%m-%d").to_string())
}

#[must_use]
/// Extracts a date in `yyyy.mm.dd.` format from a string.
pub fn extract_date(string: &Option<String>) -> Option<NaiveDate> {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(\d{4})\.(\d{2})\.(\d{2})").unwrap());
    string.as_ref().and_then(|str| {
        let caps = RE.captures(str)?;
        let year = caps.get(1)?.as_str();
        let month = caps.get(2)?.as_str();
        let day = caps.get(3)?.as_str();

        NaiveDate::from_ymd_opt(year.parse().ok()?, month.parse().ok()?, day.parse().ok()?)
    })
}

#[must_use]
pub fn parse_sqlite_date(value: &Option<String>) -> Option<NaiveDateTime> {
    value
        .as_ref()
        .and_then(|date_str| parse_date_2_format(date_str))
}

fn parse_date_2_format(value: &str) -> Option<NaiveDateTime> {
    if let Ok(ndt) = NaiveDateTime::parse_from_str(value, "%Y%m%d%H%M%S") {
        Some(ndt)
    } else {
        NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").ok()
    }
}

#[must_use]
pub fn format_sqlite_date(ndt: &NaiveDateTime) -> String {
    //    ndt.format("%Y%m%d%H%M%S").to_string()
    ndt.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[must_use]
pub fn format_guid(guid: &str) -> String {
    let mut lower = guid.to_lowercase();
    lower.retain(|c| c != '-');
    lower
}

#[must_use]
pub fn get_value_or_empty(opt: &Option<String>) -> &str {
    match opt {
        Some(x) => x,
        None => "",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DenominatedValue {
    pub value: i64,
    pub denom: i64,
}

impl DenominatedValue {
    #[must_use]
    pub fn new(value: i64, denom: i64) -> Self {
        Self { value, denom }
    }

    /// Converts a decimal to a fixed-point value with the given denominator.
    ///
    /// Returns `None` if the rounded value does not fit into an `i64`.
    #[must_use]
    pub fn denominate_decimal(value: Decimal, denom: i32) -> Option<Self> {
        let value = (value * Decimal::from(denom)).round();
        Some(Self {
            value: value.to_i64()?,
            denom: i64::from(denom),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn to_date(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(year, month, day)
            .and_then(|f| f.and_hms_opt(hour, min, sec))
            .expect("Valid date time")
    }

    #[test]
    fn test_parse_none() {
        assert_eq!(parse_sqlite_date(&None), None);
    }

    #[test]
    fn test_parse_sqlite_string() {
        assert_eq!(
            parse_sqlite_date(&Some("20161020203213".to_string())),
            Some(to_date(2016, 10, 20, 20, 32, 13))
        );
    }

    #[test]
    fn test_format_sqlite_date() {
        assert_eq!(
            format_sqlite_date(&to_date(2016, 10, 20, 20, 32, 13)),
            "2016-10-20 20:32:13"
        );
    }

    #[test]
    fn test_format_and_parse_sqlite_date() {
        let nd = to_date(2016, 10, 20, 20, 32, 13);
        let as_str = format_sqlite_date(&nd);
        assert_eq!(parse_sqlite_date(&Some(as_str)), Some(nd));
    }

    #[test]
    fn test_extract_date_none() {
        assert_eq!(extract_date(&None), None);
    }

    #[test]
    fn test_extract_date_string() {
        assert_eq!(
            extract_date(&Some("XYZ. PD.  2016.10.20 4488620465".to_string())),
            NaiveDate::from_ymd_opt(2016, 10, 20)
        );
    }

    #[test]
    fn test_guid_formatting() {
        assert_eq!(
            format_guid("A4C28BAB-39CE-800D-AE5E-A072872B2D62"),
            "a4c28bab39ce800dae5ea072872b2d62"
        );
        assert_eq!(
            format_guid("3FC86613-6A98-584F-BB40-EB0715B75429"),
            "3fc866136a98584fbb40eb0715b75429"
        );
    }
}
