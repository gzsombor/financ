use calamine::DataType;
use chrono::NaiveDate;

pub fn cell_to_date(cell: &DataType) -> Option<NaiveDate> {
    cell_to_date_raw(cell, "%Y.%m.%d.")
}

pub fn cell_to_iso_date(cell: &DataType) -> Option<NaiveDate> {
    cell_to_date_raw(cell, "%Y-%m-%d")
}

pub fn cell_to_german_date(cell: &DataType) -> Option<NaiveDate> {
    cell_to_date_raw(cell, "%d.%m.%Y")
}

pub fn cell_to_date_raw(cell: &DataType, format: &str) -> Option<NaiveDate> {
    if let DataType::String(str) = cell {
        NaiveDate::parse_from_str(str, &format).ok()
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

pub fn cell_to_float(cell: &DataType) -> Option<f64> {
    if let DataType::Float(flt) = cell {
        Some(*flt)
    } else {
        None
    }
}
