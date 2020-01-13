use std::collections::BTreeMap;
use std::ops::Bound::Included;

use anyhow::Result;
use chrono::{Duration, Local, NaiveDate};
use console::{style, Key, Term};
use diesel::prelude::*;
use guid_create::GUID;

use crate::dbmodifier::{NewSplit, NewTransaction};
use crate::external_models::{
    ExternalTransaction, ExternalTransactionList, Matching, SheetDefinition, SheetFormat,
    TransactionPairing,
};
use crate::models::{Account, Split, Transaction};
use crate::query::accounts::AccountQuery;
use crate::query::currencies::CommoditiesQuery;
use crate::query::transactions::TransactionQuery;
use crate::utils::{format_guid, get_value_or_empty, to_string};

pub struct CorrelationCommand {
    pub input_file: String,
    pub sheet_name: Option<String>,
    pub matching: Matching,
    pub verbose: bool,
    pub list_extra_transactions: bool,
    pub account_query: AccountQuery,
    pub counterparty_account_query: AccountQuery,
    pub fee_account_query: AccountQuery,
}

struct TransactionCorrelator {
    external_transactions: ExternalTransactionList,
    account: String,
    matching: Matching,
    transaction_map: BTreeMap<NaiveDate, Vec<TransactionPairing>>,
    verbose: bool,
}

impl TransactionCorrelator {
    pub fn new(
        input_file: &str,
        sheet_name: &Option<String>,
        account: String,
        matching: Matching,
        verbose: bool,
        format: &Box<dyn SheetFormat>,
        term: &Term,
    ) -> Result<Self> {
        let mut sheet_definition = SheetDefinition::new(input_file)?;

        let external_transactions = sheet_definition.load(&sheet_name, matching, format, &term)?;
        Ok(TransactionCorrelator {
            external_transactions,
            account,
            matching,
            transaction_map: BTreeMap::new(),
            verbose,
        })
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
        if self.verbose {
            println!("Number of transactions in the database: {}", db_rows.len());
        }
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
        if self.verbose {
            println!("Found {} separate date", self.transaction_map.len());
        }
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
        if self.verbose {
            println!("Starting with {} transactions", &working_set.len());
        }
        working_set = self.match_transactions_with_delta_day(0, &working_set);
        if self.verbose {
            println!(
                "After matching with 0, {} transaction remained as unmatched",
                &working_set.len()
            );
        }
        let mut delta_day = 0;
        while !&working_set.is_empty() && delta_day < 10 {
            delta_day += 1;
            working_set = self.match_transactions_with_delta_day(delta_day, &working_set);
            working_set = self.match_transactions_with_delta_day(-delta_day, &working_set);
            if self.verbose {
                println!(
                    "After matching with {}, {} transaction remained as unmatched",
                    &delta_day,
                    &working_set.len()
                );
            }
        }
        working_set
    }

    // return the unmatched transactions
    pub fn match_transactions_with_delta_day(
        &self,
        delta_day: i64,
        transactions: &[ExternalTransaction],
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
        if let Some(ext_date) = external_transaction.get_matching_date(self.matching) {
            let actual_date = match delta_day {
                0 => ext_date,
                _ => ext_date
                    .checked_add_signed(Duration::days(delta_day))
                    .unwrap(),
            };
            if let Some(ext_amount) = external_transaction.get_amount() {
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
        None
    }
}

#[derive(Debug)]
enum Answer {
    Yes,
    No,
    Abort,
    All,
}

struct AddTransactions<'a> {
    connection: &'a SqliteConnection,
    unmatched_transactions: &'a [ExternalTransaction],
    only_account: &'a Account,
    counter_account: &'a Account,
    fee_account: &'a Option<Account>,
    term: &'a Term,
}

impl CorrelationCommand {
    pub fn execute(
        &self,
        connection: &SqliteConnection,
        term: &Term,
        format: &Box<dyn SheetFormat>,
    ) -> Result<usize> {
        if let Some(only_account) = self.account_query.get_one(&connection, true) {
            let mut correlator = TransactionCorrelator::new(
                &self.input_file.clone(),
                &self.sheet_name,
                only_account.guid.clone(),
                self.matching,
                self.verbose,
                format,
                &term,
            )?;
            correlator.build_mapping(connection);

            term.write_line(&format!(
                "Between {} and {}",
                style(to_string(correlator.get_min_date())).cyan(),
                style(to_string(correlator.get_max_date())).cyan()
            ))?;

            let unmatched_transactions = correlator.match_transactions();
            term.write_line(&format!(
                "Missing {} record from the internal database:",
                style(&unmatched_transactions.len()).red()
            ))?;

            if self.verbose {
                for tr in &unmatched_transactions {
                    println!(" - {}", &tr);
                }
            }

            let db_transactions = correlator.get_unmatched();
            term.write_line(&format!(
                "Missing {} record from the external source:",
                style(&db_transactions.len()).red()
            ))?;

            if self.list_extra_transactions {
                for tr in &db_transactions {
                    println!(" - {}", &tr);
                }
            }

            if !unmatched_transactions.is_empty() {
                let fee_account = self.fee_account_query.get_one(&connection, false);
                if let Some(counter_account) =
                    self.counterparty_account_query.get_one(&connection, true)
                {
                    let add_transactions = AddTransactions {
                        connection: &connection,
                        unmatched_transactions: &unmatched_transactions,
                        only_account: &only_account,
                        counter_account: &counter_account,
                        fee_account: &fee_account,
                        term: &term,
                    };
                    add_transactions.try_to_fix()?;
                } else {
                    term.write_line(&format!(
                        "Unable to fix, as {} is not specified exactly!",
                        style("counter account").red()
                    ))?;
                    return Err(anyhow!(
                        "Unable to fix, as account is not specified exactly!"
                    ));
                }
            } else {
                term.write_line(&format!(
                    "No unmatched transactions, everything is {}",
                    style("ok.").green()
                ))?;
            }
            Ok(unmatched_transactions.len())
        } else {
            Err(anyhow!("Account is not specified exactly!"))
        }
    }
}

impl<'a> AddTransactions<'a> {
    fn try_to_fix(&self) -> Result<()> {
        if self.only_account.commodity_guid != self.counter_account.commodity_guid {
            self.term.write_line(&format!(
                "The two account has different commodities, unable to transfer between: {} - {}",
                style(self.only_account).red(),
                style(self.counter_account).red()
            ))?;
            return Err(anyhow!(
                "Unable to fix, the two account has different commodities: {} - {}!",
                self.only_account,
                self.counter_account
            ));
        }
        self.term.write_line(&format!(
            "Creating transactions between {} and {}",
            self.counter_account, self.only_account
        ))?;
        for idx in 0..self.unmatched_transactions.len() {
            let transaction = &self.unmatched_transactions[idx];
            self.check_fee_configured(&transaction)?;
            self.term.write_line(&format!(
                "Adding {} [{}es/{}o/{}bort/a{}l]",
                style(&transaction).cyan(),
                style("Y").red(),
                style("N").red(),
                style("A").red(),
                style("L").red()
            ))?;
            let answer = Answer::get(&self.term)?;
            match answer {
                Answer::Yes => self.add_transaction(&transaction)?,
                Answer::No => self
                    .term
                    .write_line(&format!("Skipping {}", style(&transaction).magenta()))?,
                Answer::Abort => return Ok(()),
                Answer::All => {
                    for current in idx..self.unmatched_transactions.len() {
                        let transaction = &self.unmatched_transactions[idx];
                        self.check_fee_configured(&transaction)?;
                        self.add_transaction(&self.unmatched_transactions[current])?;
                    }
                    return Ok(());
                }
            };
        }
        Ok(())
    }

    fn check_fee_configured(&self, transaction: &ExternalTransaction) -> Result<()> {
        match (transaction.transaction_fee, self.fee_account) {
            (Some(fee), None) if fee != 0.0 =>
                Err(anyhow!("Transaction({}) has a fee({}), however no fee account specified!",
                    get_value_or_empty(&transaction.description), fee)),
            _ => Ok(())
        }
    }

    fn add_transaction(&self, transaction: &ExternalTransaction) -> Result<()> {
        self.term
            .write_line(&format!("adding {}", style(&transaction).red()))?;
        let commodity_guid = &self
            .only_account
            .commodity_guid
            .clone()
            .expect("Commodity guid is not null");
        let commodity = CommoditiesQuery::get_by_guid(&self.connection, &commodity_guid)
            .expect("Currency not found!");
        let tr_guid = format_guid(&GUID::rand().to_string());
        let spend_date = transaction
            .get_matching_date(Matching::BySpending)
            .map(|d| d.and_hms(12, 0, 0));
        let current_time = Local::now().naive_local();
        let description = transaction
            .get_description_or_category()
            .unwrap_or_else(|| "".to_owned());
        let amount = transaction.get_amount().expect("Amount is expected!");

        let fee_value = &transaction.transaction_fee.unwrap_or_default();

        NewTransaction::insert(
            &self.connection,
            &tr_guid,
            &commodity.guid,
            spend_date,
            current_time,
            &description,
        );
        let _split_id_from = NewSplit::insert(
            &self.connection,
            &tr_guid,
            &self.only_account,
            &description,
            &commodity,
            amount,
        );
        let _split_id_counter = NewSplit::insert(
            &self.connection,
            &tr_guid,
            &self.counter_account,
            &transaction.get_other_account_desc(),
            &commodity,
            -amount - fee_value,
        );
        if *fee_value != 0.0 {
            let _fee_id_counter = NewSplit::insert(
                &self.connection,
                &tr_guid,
                &self.fee_account.as_ref().expect("Fee account is expected!"),
                &description,
                &commodity,
                *fee_value
            );
        }
        /*        self.term.write_line(&format!(
            "trans id:{} \n\t{} - {} \n\t{} - {}",
            tr_guid,
            self.only_account.name,
            split_id_from,
            self.counter_account.name,
            split_id_counter
        ))?;*/
        Ok(())
    }
}

impl Answer {
    fn get(term: &Term) -> Result<Answer> {
        loop {
            let key = term.read_key()?;
            match key {
                Key::Char('y') => return Ok(Answer::Yes),
                Key::Char('Y') => return Ok(Answer::Yes),
                Key::Enter => return Ok(Answer::Yes),
                Key::Char('n') => return Ok(Answer::No),
                Key::Char('N') => return Ok(Answer::No),
                Key::Escape => return Ok(Answer::Abort),
                Key::Char('a') => return Ok(Answer::Abort),
                Key::Char('A') => return Ok(Answer::Abort),
                Key::Char('l') => return Ok(Answer::All),
                Key::Char('L') => return Ok(Answer::All),
                _ => {}
            }
        }
    }
}
