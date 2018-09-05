use chrono::NaiveDate;
use diesel::prelude::*;
use dotenv::dotenv;
use regex::Regex;
use std::env;

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn to_date(date_string: Option<String>) -> Option<NaiveDate> {
    date_string.and_then(|x| NaiveDate::parse_from_str(x.as_ref(), "%Y-%m-%d").ok())
}

pub fn to_string(date: Option<NaiveDate>) -> String {
    date.map_or_else(|| "".to_string(), |dt| dt.format("%Y-%m-%d").to_string())
}

pub fn extract_date(string: Option<String>) -> Option<NaiveDate> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(\d{4})\.(\d{2})\.(\d{2})").unwrap();
    }
    string.and_then(|str| {
        let caps = RE.captures(&str)?;
        let year = caps.get(1).unwrap().as_str();
        let month = caps.get(2).unwrap().as_str();
        let day = caps.get(3).unwrap().as_str();

        Some(NaiveDate::from_ymd(
            year.parse().expect("Number as year"),
            month.parse().expect("Number as month"),
            day.parse().expect("Number as day"),
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_extract_date_none() {
        assert_eq!(extract_date(None), None);
    }

    #[test]
    fn test_extract_date_string() {
        assert_eq!(
            extract_date(Some("XYZ. PD.  2016.10.20 4488620465".to_string())),
            Some(NaiveDate::from_ymd(2016, 10, 20))
        );
    }

}
