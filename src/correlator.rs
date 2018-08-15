use std::collections::BTreeMap;

use commands::TransactionQuery;

use calamine::{open_workbook_auto, DataType, Range, Reader, Sheets};
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

/*struct TransactionList {
    transactions: Vec<ExternalTransaction>,
    start_date: NaiveDate,
    end_date: NaiveDate,
}
*/
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

pub fn correlate(connection: &SqliteConnection, input_file: String, sheet_name: String, account: String) {
    let mut sd = SheetDefinition::new(input_file);

    let tx = sd.load(&sheet_name);
    println!("Full transaction list: {:?}", tx);
    let db_query = TransactionQuery {
        limit: 10000,
        txid_filter: None,
        account_filter: Some(account),
        description_filter: None,
        memo_filter: None,
        before_filter: None,
        after_filter: None,
    };
    let db_rows = db_query.execute(&connection);
    println!("loaded {} transactions from the database", db_rows.len());

    let mut trans_map = BTreeMap::new();

    for row in db_rows {
        if let Some(posting_date) = row.1.posting().map(|date_time| date_time.date()) {
            let list = trans_map.entry(posting_date).or_insert_with(Vec::new);
            list.push(row);
        }
    }
    println!("found {} separate date", trans_map.len());
}
