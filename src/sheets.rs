use calamine::Data;
use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::{prelude::FromPrimitive, Decimal};

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
        Data::String(str) => NaiveDate::parse_from_str(str, format).ok(),
        Data::DateTime(date_time) => {
            println!("Unexpected format: date time : {date_time}");
            panic!("Wrong type!");
        }
        Data::DateTimeIso(date_time) => NaiveDate::parse_from_str(date_time, "%Y-%m-%d").ok(),
        _ => None,
    }
}

pub fn cell_to_datetime(cell: &Data) -> Option<NaiveDateTime> {
    if let Data::String(str) = cell {
        NaiveDateTime::parse_from_str(str, "%Y.%m.%d. %H:%M:%S").ok()
    } else {
        None
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
