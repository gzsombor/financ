use std::fmt;
use std::fs::File;
use std::{cell::RefCell, io::BufReader};

use anyhow::Result;
use calamine::{open_workbook_auto, DataType, Range, Reader, Sheets};
use chrono::NaiveDate;
use console::{style, Term};

use crate::models::{Split, Transaction};

#[derive(Debug, Clone)]
pub struct ExternalTransaction {
    pub date: Option<NaiveDate>,
    pub booking_date: Option<NaiveDate>,
    pub amount: Option<f64>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub other_account: Option<String>,
    pub other_account_name: Option<String>,
    pub textual_date: Option<NaiveDate>,
    pub transaction_fee: Option<f64>,
}

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
            write!(f, " {}", amount)?;
        }
        if let Some(transaction_fee) = self.transaction_fee {
            write!(f, " (fee: {})", transaction_fee)?;
        }
        if let Some(category) = &self.category {
            write!(f, " [{}]", category)?;
        }
        if let Some(description) = &self.description {
            write!(f, " - {}", description)?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone)]
pub enum Matching {
    ByBooking,
    BySpending,
}

impl ExternalTransaction {
    // TODO: make it configurable
    pub fn get_matching_date(&self, matching: Matching) -> Option<NaiveDate> {
        match matching {
            Matching::ByBooking => self.date,
            Matching::BySpending => self.textual_date.or(self.date),
        }
    }

    pub fn get_description_or_category(&self) -> Option<String> {
        self.description.clone().or_else(|| self.category.clone())
    }
    /*
        pub fn get_description(&self) -> Option<String> {
            self.description.clone()
        }
    */
    pub fn get_amount(&self) -> Option<f64> {
        self.amount
    }

    pub fn get_other_account_desc(&self) -> String {
        match (&self.other_account, &self.other_account_name) {
            (Some(acc), Some(name)) => {
                let mut res = acc.clone();
                res.push_str(" - ");
                res.push_str(name);
                res
            }
            (None, Some(name)) => name.clone(),
            (Some(acc), None) => acc.clone(),
            (_, _) => "".to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct ExternalTransactionList(
    pub Vec<ExternalTransaction>,
    pub Option<NaiveDate>,
    pub Option<NaiveDate>,
);

pub struct SheetDefinition {
    //    input_file: String,
    workbook: Sheets<BufReader<File>>,
}

pub trait SheetFormat {
    fn parse_sheet(&self, range: &Range<DataType>) -> Vec<ExternalTransaction>;
}

impl SheetDefinition {
    pub fn new(input_file: &str) -> Result<Self> {
        let workbook = open_workbook_auto(input_file)?; //.expect("Cannot open file");
        Ok(SheetDefinition {
            // input_file,
            workbook,
        })
    }

    pub fn load(
        &mut self,
        maybe_sheet_name: Option<String>,
        matching: Matching,
        format: &Box<dyn SheetFormat>,
        term: &Term,
    ) -> Result<ExternalTransactionList> {
        let sheet_name = match maybe_sheet_name {
            Some(name) => name,
            None => {
                let sheet_names = self.workbook.sheet_names();
                sheet_names.first().unwrap().to_owned()
            }
        };
        if let Ok(sheet) = self.workbook.worksheet_range(&sheet_name) {
            term.write_line(&format!("found sheet '{}'", style(sheet_name).blue()))?;
            let trans = format.parse_sheet(&sheet);
            let (min, max) = SheetDefinition::find_min_max(&trans, matching);
            Ok(ExternalTransactionList(trans, min, max))
        } else {
            term.write_line(&format!(
                "Sheet '{}' not found, no transactions will be imported!",
                style(&sheet_name).red()
            ))?;
            Err(anyhow!(
                "Sheet '{}' not found, no transactions will be imported!",
                sheet_name
            ))
        }
    }

    fn find_min_max(
        transactions: &[ExternalTransaction],
        matching: Matching,
    ) -> (Option<NaiveDate>, Option<NaiveDate>) {
        transactions
            .iter()
            .fold((None, None), |(min, max), current| {
                let maybe_current_date = current.get_matching_date(matching);
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

pub struct TransactionPairing {
    transaction: Transaction,
    split: Split,
    external: RefCell<Option<ExternalTransaction>>,
}

impl TransactionPairing {
    pub fn new(pair: (Split, Transaction)) -> Self {
        TransactionPairing {
            transaction: pair.1,
            split: pair.0,
            external: RefCell::new(None),
        }
    }
    pub fn is_equal_amount(&self, amount: f64) -> bool {
        self.split.is_equal_amount(amount)
    }

    pub fn is_not_matched(&self) -> bool {
        self.external.borrow().is_none()
    }

    pub fn pair_with(&self, external_trans: &ExternalTransaction) {
        let mut inner = self.external.borrow_mut();
        *inner = Some(external_trans.to_owned());
    }
}

impl fmt::Display for TransactionPairing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.transaction, self.split)
    }
}
