use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
use std::ops::Bound::Included;

use calamine::{open_workbook_auto, DataType, Range, Reader, Sheets};
use chrono::{Duration, NaiveDate};
use diesel::prelude::*;
use regex::Regex;

use models::{Split, Transaction};
use query::accounts::AccountQuery;
use query::transactions::TransactionQuery;
use utils::{extract_date, to_string};

pub struct SheetDefinition {
    input_file: String,
    workbook: Sheets,
}

#[derive(Debug, Clone)]
pub struct ExternalTransaction {
    date: Option<NaiveDate>,
    booking_date: Option<NaiveDate>,
    amount: Option<f64>,
    category: Option<String>,
    description: Option<String>,
    other_account: Option<String>,
    textual_date: Option<NaiveDate>,
}

#[derive(Debug)]
pub struct ExternalTransactionList(
    Vec<ExternalTransaction>,
    Option<NaiveDate>,
    Option<NaiveDate>,
);

impl fmt::Display for ExternalTransaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(date) = self.date {
            f.write_str(&date.format("%Y-%m-%d").to_string())?;
        } else {
            f.write_str("----------")?;
        }
        if let Some(other_date) = self.textual_date {
            f.write_str(&other_date.format(" %Y-%m-%d").to_string())?;
        } else {
            f.write_str(" ----------")?;
        }
        if let Some(amount) = self.amount {
            write!(f, " {}", amount);
        }
        if let Some(category) = &self.category {
            write!(f, " [{}]", category);
        }
        if let Some(description) = &self.description {
            write!(f, " - {}", description);
        }
        Ok(())
    }
}

impl ExternalTransaction {
    pub fn get_matching_date(&self) -> Option<NaiveDate> {
        self.textual_date.or(self.date)
    }
}

// [derive(Debug)]
struct TransactionPairing {
    transaction: Transaction,
    split: Split,
    external: RefCell<Option<ExternalTransaction>>,
}

pub struct TransactionCorrelator {
    sheet_definition: SheetDefinition,
    external_transactions: ExternalTransactionList,
    account: String,
    transaction_map: BTreeMap<NaiveDate, Vec<TransactionPairing>>,
}

impl TransactionPairing {
    pub fn new(pair: (Split, Transaction)) -> Self {
        TransactionPairing {
            transaction: pair.1,
            split: pair.0,
            external: RefCell::new(None),
        }
    }
    fn is_equal_amount(&self, amount: f64) -> bool {
        self.split.is_equal_amount(amount)
    }

    fn is_not_matched(&self) -> bool {
        self.external.borrow().is_none()
    }

    fn pair_with(&self, external_trans: &ExternalTransaction) {
        let mut inner = self.external.borrow_mut();
        *inner = Some(external_trans.to_owned());
    }
}

impl fmt::Display for TransactionPairing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.transaction, self.split)
    }
}

impl SheetDefinition {
    pub fn new(input_file: String) -> Self {
        let workbook = open_workbook_auto(&input_file).expect("Cannot open file");
        SheetDefinition {
            input_file,
            workbook,
        }
    }

    pub fn load(&mut self, sheet_name: &str) -> ExternalTransactionList {
        if let Some(Ok(sheet)) = self.workbook.worksheet_range(&sheet_name) {
            println!("found sheet '{}'", &sheet_name);
            let trans = SheetDefinition::parse_sheet(&sheet);
            let (min, max) = SheetDefinition::find_min_max(&trans);
            ExternalTransactionList(trans, min, max)
        } else {
            ExternalTransactionList(Vec::new(), None, None)
        }
    }

    fn cell_to_date(cell: &DataType) -> Option<NaiveDate> {
        if let DataType::String(str) = cell {
            NaiveDate::parse_from_str(str, "%Y.%m.%d.").ok()
        } else {
            None
        }
    }

    fn cell_to_string(cell: &DataType) -> Option<String> {
        if let DataType::String(str) = cell {
            Some(str.clone())
        } else {
            None
        }
    }

    fn cell_to_float(cell: &DataType) -> Option<f64> {
        if let DataType::Float(flt) = cell {
            Some(flt.clone())
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
                let descrip = SheetDefinition::cell_to_string(&row[8]);
                let parsed_date = extract_date(descrip.clone());
                // println!("{:?} - {:?} - {:?}", &row[0],&row[5], &row[8]);
                ExternalTransaction {
                    date: SheetDefinition::cell_to_date(&row[2]),
                    booking_date: SheetDefinition::cell_to_date(&row[3]),
                    amount: SheetDefinition::cell_to_float(&row[4]),
                    category: SheetDefinition::cell_to_string(&row[1]),
                    description: descrip,
                    other_account: SheetDefinition::cell_to_string(&row[6]),
                    textual_date: parsed_date,
                }
            }).collect()
    }

    fn find_min_max(
        transactions: &Vec<ExternalTransaction>,
    ) -> (Option<NaiveDate>, Option<NaiveDate>) {
        transactions
            .into_iter()
            .fold((None, None), |(min, max), current| {
                let maybe_current_date = current.get_matching_date();
                match maybe_current_date {
                    Some(current_date) => {
                        let new_min = match min {
                            None => Some(current_date),
                            Some(y) => Some(if current_date < y { current_date } else { y }),
                        };
                        let new_max = match max {
                            None => Some(current_date),
                            Some(y) => Some(if current_date > y { current_date } else { y }),
                        };
                        (new_min, new_max)
                    }
                    None => (min, max),
                }
            })
    }
}

impl TransactionCorrelator {
    pub fn new(input_file: String, sheet_name: String, account: String) -> Self {
        let mut sheet_definition = SheetDefinition::new(input_file);

        let external_transactions = sheet_definition.load(&sheet_name);
        TransactionCorrelator {
            sheet_definition,
            external_transactions,
            account,
            transaction_map: BTreeMap::new(),
        }
    }

    fn load_from_database(&self, connection: &SqliteConnection) -> Vec<(Split, Transaction)> {
        let db_query = TransactionQuery {
            limit: 10000,
            txid_filter: None,
            account_filter: Some(self.account.clone()),
            description_filter: None,
            memo_filter: None,
            before_filter: None,
            after_filter: None,
        };
        let db_rows = db_query.execute(&connection);
        println!("loaded {} transactions from the database", db_rows.len());
        db_rows
    }

    fn get_min_date(&self) -> Option<NaiveDate> {
        self.external_transactions.1.to_owned()
    }

    fn get_max_date(&self) -> Option<NaiveDate> {
        self.external_transactions.2.to_owned()
    }

    fn build_mapping(&mut self, connection: &SqliteConnection) {
        let db_transactions = self.load_from_database(&connection);

        for row in db_transactions {
            if let Some(posting_date) = row.1.posting().map(|date_time| date_time.date()) {
                let list = self
                    .transaction_map
                    .entry(posting_date)
                    .or_insert_with(Vec::new);
                list.push(TransactionPairing::new(row));
            }
        }
        println!("found {} separate date", self.transaction_map.len());
    }

    fn get_unmatched(&self) -> Vec<&TransactionPairing> {
        let min = self.get_min_date();
        let max = self.get_max_date();
        if let Some(max_value) = max {
            if let Some(min_value) = min {
                return self
                    .transaction_map
                    .range((Included(min_value), Included(max_value)))
                    .map(|(_, v)| v)
                    .flatten()
                    .filter(|pairing| pairing.is_not_matched())
                    .collect();
            }
        }
        self.transaction_map
            .values()
            .flatten()
            .filter(|pairing| pairing.is_not_matched())
            .collect()
    }

    pub fn match_transactions(&mut self) -> Vec<ExternalTransaction> {
        let mut working_set = self.external_transactions.0.clone();
        println!("Starting with {} transactions", &working_set.len());
        working_set = self.match_transactions_with_delta_day(0, &working_set);
        println!(
            "After matching with 0, {} transaction remained as unmatched",
            &working_set.len()
        );
        let mut delta_day = 0;
        while !&working_set.is_empty() && delta_day < 10 {
            delta_day = delta_day + 1;
            working_set = self.match_transactions_with_delta_day(delta_day, &working_set);
            working_set = self.match_transactions_with_delta_day(-delta_day, &working_set);
            println!(
                "After matching with {}, {} transaction remained as unmatched",
                &delta_day,
                &working_set.len()
            );
        }
        working_set
    }

    // return the unmatched transactions
    pub fn match_transactions_with_delta_day(
        &self,
        delta_day: i64,
        transactions: &Vec<ExternalTransaction>,
    ) -> Vec<ExternalTransaction> {
        let mut result = Vec::new();
        for external_transaction in transactions {
            if self
                .add_transaction(delta_day, &external_transaction)
                .is_none()
            {
                result.push(external_transaction.clone());
            }
        }
        result
    }

    fn add_transaction(
        &self,
        delta_day: i64,
        external_transaction: &ExternalTransaction,
    ) -> Option<&TransactionPairing> {
        if let Some(ext_date) = external_transaction.get_matching_date() {
            let actual_date = match delta_day {
                0 => ext_date,
                _ => ext_date
                    .checked_add_signed(Duration::days(delta_day))
                    .unwrap(),
            };
            if let Some(ext_amount) = external_transaction.amount {
                if let Some(list) = self.transaction_map.get(&actual_date) {
                    if let Some(tr_pairing) = list
                        .iter()
                        .find(|&x| x.is_equal_amount(ext_amount) && x.is_not_matched())
                    {
                        tr_pairing.pair_with(external_transaction);
                        return Some(&tr_pairing);
                    }
                }
            }
        }
        return None;
    }
}

pub fn correlate(
    connection: &SqliteConnection,
    input_file: String,
    sheet_name: String,
    account_query: AccountQuery,
) -> Option<usize> {
    if let Some(only_account) = account_query.get_one(&connection) {
        let mut correlator = TransactionCorrelator::new(input_file, sheet_name, only_account.guid);
        correlator.build_mapping(connection);
        println!(
            "Between {} and {}",
            to_string(correlator.get_min_date()),
            to_string(correlator.get_max_date())
        );
        let unmatched_transactions = correlator.match_transactions();
        for tr in &unmatched_transactions {
            println!(" - {}", &tr);
        }
        let db_transactions = correlator.get_unmatched();
        println!("Missing from external table:");
        for tr in &db_transactions {
            println!(" - {}", &tr);
        }
        Some(unmatched_transactions.len())
    } else {
        None
    }
}
