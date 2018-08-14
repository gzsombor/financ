use calamine::{open_workbook_auto, DataType, Range, Reader, Sheets, Xlsx};
use chrono::NaiveDate;
use diesel::prelude::*;

pub struct SheetDefinition {
    input_file: String,
    workbook: Sheets,
}

#[derive(Debug)]
pub struct ExternalTransaction {
    date: Option<NaiveDate>,
    booking_date: Option<NaiveDate>,
    amount: Option<f64>,
    category: Option<String>,
    description: Option<String>,
    other_account: Option<String>,
}

struct TransactionList {
    transactions: Vec<ExternalTransaction>,
    start_date: NaiveDate,
    end_date: NaiveDate,
}

/*
impl ExternalTransaction {
	pub fn new(date: NaiveDate,	amount: i64, description: String) -> Self {
		ExternalTransaction {
			date:date,
			amount:amount,
			description:description,
			other_account:None,
		}
	}
}
*/
impl SheetDefinition {
    pub fn new(input_file: String) -> Self {
        let workbook = open_workbook_auto(&input_file).expect("Cannot open file");
        SheetDefinition {
            input_file,
            workbook,
        }
    }

    pub fn load(&mut self, sheet_name: &str) -> Vec<ExternalTransaction> {
        if let Some(Ok(sheet)) = self.workbook.worksheet_range(&sheet_name) {
            println!("found sheet '{}'", &sheet_name);
            parse_sheet(&sheet)
        } else {
            Vec::new()
        }
    }
}

fn cell_to_date(cell: DataType) -> Option<NaiveDate> {
    if let DataType::String(str) = cell {
        NaiveDate::parse_from_str(str.as_ref(), "%Y.%m.%d.").ok()
    } else {
        None
    }
}

fn cell_to_string(cell: DataType) -> Option<String> {
    if let DataType::String(str) = cell {
        Some(str)
    } else {
        None
    }
}

fn cell_to_float(cell: DataType) -> Option<f64> {
    if let DataType::Float(flt) = cell {
        Some(flt)
    } else {
        None
    }
}

fn parse_sheet(range: &Range<DataType>) -> Vec<ExternalTransaction> {
    println!(
        "Range starts : {:?} ends at {:?}",
        range.start(),
        range.end()
    );
    range
        .rows()
        .filter(|row| row[0] != DataType::Empty)
        .map(|row| {
            // println!("row is {:?}", row);
            ExternalTransaction {
                date: cell_to_date(row[2].clone()),
                booking_date: cell_to_date(row[3].clone()),
                amount: cell_to_float(row[4].clone()),
                category: cell_to_string(row[1].clone()),
                description: cell_to_string(row[8].clone()),
                other_account: cell_to_string(row[6].clone()),
            }
        }).collect()
}

pub fn correlate(connection: &SqliteConnection, input_file: String, account: String) {
    let mut sd = SheetDefinition::new(input_file);

    let tx = sd.load("2015-06-");
    println!("Full transaction list: {:?}", tx);
}
