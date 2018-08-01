use chrono::NaiveDate;
use diesel::prelude::*;
use dotenv::dotenv;
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
