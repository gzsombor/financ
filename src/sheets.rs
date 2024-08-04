use calamine::DataType;
use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::{prelude::FromPrimitive, Decimal};

// Format yyyy.mm.dd.
pub fn cell_to_date(cell: &DataType) -> Option<NaiveDate> {
    cell_to_date_raw(cell, "%Y.%m.%d.")
}

// Format yyyy-mm-dd
pub fn cell_to_iso_date(cell: &DataType) -> Option<NaiveDate> {
    cell_to_date_raw(cell, "%Y-%m-%d")
}

// Format dd.mm.yyyy
pub fn cell_to_german_date(cell: &DataType) -> Option<NaiveDate> {
    cell_to_date_raw(cell, "%d.%m.%Y")
}

// Format dd-mm-yyyy
pub fn cell_to_english_date(cell: &DataType) -> Option<NaiveDate> {
    cell_to_date_raw(cell, "%d-%m-%Y")
}

pub fn cell_to_date_raw(cell: &DataType, format: &str) -> Option<NaiveDate> {
    match cell {
        DataType::String(str) => NaiveDate::parse_from_str(str, format).ok(),
        DataType::DateTime(date_time) => {
            println!("Unexpected format: date time : {date_time}");
            panic!("Wrong type!");
        }
        DataType::DateTimeIso(date_time) => NaiveDate::parse_from_str(date_time, "%Y-%m-%d").ok(),
        _ => None,
    }
}

pub fn cell_to_datetime(cell: &DataType) -> Option<NaiveDateTime> {
    if let DataType::String(str) = cell {
        NaiveDateTime::parse_from_str(str, "%Y.%m.%d. %H:%M:%S").ok()
    } else {
        None
    }
}

pub fn cell_to_string(cell: &DataType) -> Option<String> {
    if let DataType::String(str) = cell {
        if str.is_empty() {
            None
        } else {
            Some(str.clone())
        }
    } else {
        None
    }
}

pub fn cell_to_decimal(cell: &DataType) -> Option<Decimal> {
    match cell {
        DataType::Float(flt) => Decimal::from_f64(*flt),
        DataType::Int(numb) => Decimal::from_i64(*numb),
        DataType::String(string) => Decimal::from_scientific(string).ok(),
        _ => None,
    }
}
