use calamine::DataType;
use chrono::NaiveDate;

pub fn cell_to_date(cell: &DataType) -> Option<NaiveDate> {
    if let DataType::String(str) = cell {
        NaiveDate::parse_from_str(str, "%Y.%m.%d.").ok()
    } else {
        None
    }
}

pub fn cell_to_iso_date(cell: &DataType) -> Option<NaiveDate> {
    if let DataType::String(str) = cell {
        NaiveDate::parse_from_str(str, "%Y-%m-%d").ok()
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
