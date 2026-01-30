use std::fmt;
use std::fs::File;
use std::{cell::RefCell, io::BufReader};

use anyhow::Result;
use calamine::{Data, Range, Reader, Sheets, open_workbook_auto};
use chrono::NaiveDate;
use console::{Term, style};
use rhai::CustomType;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::models::{Split, Transaction};

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize, CustomType)]
pub struct ExternalTransaction {
    pub date: Option<NaiveDate>,
    pub booking_date: Option<NaiveDate>,
    pub amount: Option<Decimal>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub other_account: Option<String>,
    pub other_account_name: Option<String>,
    pub textual_date: Option<NaiveDate>,
    pub transaction_fee: Option<Decimal>,
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

#[derive(Debug, Clone, CustomType)]
pub struct ExternalTransactionBuilder {
    date: Option<NaiveDate>,
    booking_date: Option<NaiveDate>,
    amount: Option<Decimal>,
    category: Option<String>,
    description: Option<String>,
    other_account: Option<String>,
    other_account_name: Option<String>,
    textual_date: Option<NaiveDate>,
    transaction_fee: Option<Decimal>,
}

impl ExternalTransactionBuilder {
    pub fn new() -> Self {
        ExternalTransactionBuilder {
            date: None,
            booking_date: None,
            amount: None,
            category: None,
            description: None,
            other_account: None,
            other_account_name: None,
            textual_date: None,
            transaction_fee: None,
        }
    }

    pub fn with_date(mut self, date: Option<NaiveDate>) -> Self {
        self.date = date;
        self
    }

    pub fn with_booking_date(mut self, booking_date: Option<NaiveDate>) -> Self {
        self.booking_date = booking_date;
        self
    }

    pub fn with_amount(mut self, amount: Option<Decimal>) -> Self {
        self.amount = amount;
        self
    }

    pub fn with_category(mut self, category: Option<String>) -> Self {
        self.category = category;
        self
    }

    pub fn with_description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    pub fn with_other_account(mut self, other_account: Option<String>) -> Self {
        self.other_account = other_account;
        self
    }

    pub fn with_other_account_name(mut self, other_account_name: Option<String>) -> Self {
        self.other_account_name = other_account_name;
        self
    }

    pub fn with_textual_date(mut self, textual_date: Option<NaiveDate>) -> Self {
        self.textual_date = textual_date;
        self
    }

    pub fn with_transaction_fee(mut self, transaction_fee: Option<Decimal>) -> Self {
        self.transaction_fee = transaction_fee;
        self
    }

    pub fn create(self) -> ExternalTransaction {
        ExternalTransaction {
            date: self.date,
            booking_date: self.booking_date,
            amount: self.amount,
            category: self.category,
            description: self.description,
            other_account: self.other_account,
            other_account_name: self.other_account_name,
            textual_date: self.textual_date,
            transaction_fee: self.transaction_fee,
        }
    }
}

impl Default for ExternalTransactionBuilder {
    fn default() -> Self {
        Self::new()
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
    pub fn get_amount(&self) -> Option<Decimal> {
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

pub trait SheetParser {
    fn parse_sheet(&self, range: &Range<Data>) -> Result<Vec<ExternalTransaction>>;
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
        format: &dyn SheetParser,
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
            term.write_line(&format!("found sheet '{}'", style(&sheet_name).blue()))?;
            let trans = format.parse_sheet(&sheet)?;
            term.write_line(&format!(
                "found {} transaction on sheet {}",
                style(trans.len()).cyan(),
                style(&sheet_name).blue()
            ))?;
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
        let dates = transactions
            .iter()
            .flat_map(|current| current.get_matching_date(matching));
        let min = dates.clone().min();
        let max = dates.max();

        (min, max)
    }
}

#[derive(Debug)]
pub struct TransactionPairing {
    transaction: Transaction,
    split: Split,
    external: RefCell<Option<ExternalTransaction>>,
    amount: Decimal,
}

impl TransactionPairing {
    pub fn new(pair: (Split, Transaction)) -> Self {
        let amount = pair.0.get_quantity_as_decimal();
        TransactionPairing {
            transaction: pair.1,
            split: pair.0,
            external: RefCell::new(None),
            amount,
        }
    }
    pub fn is_equal_amount(&self, amount: Decimal) -> bool {
        amount.normalize() == self.amount
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
