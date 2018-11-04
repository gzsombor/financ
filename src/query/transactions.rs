use chrono::naive::NaiveDate;
use clap::ArgMatches;
use diesel::prelude::*;

use models::{Account, Split, Transaction};
use utils::to_date;

pub struct TransactionQuery {
    pub limit: i64,
    pub txid_filter: Option<String>,
    pub account_filter: Option<String>,
    pub description_filter: Option<String>,
    pub memo_filter: Option<String>,
    pub before_filter: Option<NaiveDate>,
    pub after_filter: Option<NaiveDate>,
}

impl TransactionQuery {
    pub fn with_account_id(self, account_id: String) -> Self {
        TransactionQuery {
            limit: self.limit,
            txid_filter: self.txid_filter,
            account_filter: Some(account_id),
            description_filter: self.description_filter,
            memo_filter: self.memo_filter,
            before_filter: self.before_filter,
            after_filter: self.after_filter,
        }
    }

    pub fn execute(&self, connection: &SqliteConnection) -> Vec<(Split, Transaction)> {
        use schema::splits::dsl::*;
        use schema::transactions::dsl::*;

        let join = splits.inner_join(transactions);

        let mut query = join.into_boxed();
        if let Some(ref txid_txt) = self.txid_filter {
            query = query.filter(tx_guid.like(format!("%{}%", txid_txt)));
        }
        if let Some(ref account_txt) = self.account_filter {
            query = query.filter(account_guid.like(format!("%{}%", account_txt)));
        }
        if let Some(ref name_txt) = self.memo_filter {
            query = query.filter(memo.like(format!("%{}%", name_txt)));
        }
        if let Some(ref description_txt) = self.description_filter {
            query = query.filter(description.like(format!("%{}%", description_txt)));
        }
        if let Some(after_date) = self.after_filter {
            let after_as_txt = after_date.and_hms(0, 0, 0).format("%Y%m%d%H%M%S");
            query = query.filter(post_date.ge(after_as_txt.to_string()));
        }
        if let Some(before_date) = self.before_filter {
            let before_as_txt = before_date.and_hms(23, 59, 59).format("%Y%m%d%H%M%S");
            query = query.filter(post_date.le(before_as_txt.to_string()));
        }

        query
            .limit(self.limit)
            .load::<(Split, Transaction)>(connection)
            .expect("Error loading splits")
    }

    pub fn execute_and_process(
        &self,
        connection: &SqliteConnection,
        target_account: &Option<Account>,
    ) {
        let results = self.execute(&connection);
        match target_account {
            None => self.display(results),
            Some(account) => self.move_splits(results, account),
        }
    }
    fn display(&self, transactions: Vec<(Split, Transaction)>) {
        println!("Displaying {} splits", transactions.len());
        for (split, tx) in transactions {
            println!(
                "[{}]<{}> - {} - {}",
                split.account_guid, split.tx_guid, tx, split
            );
        }
    }
    fn move_splits(&self, transactions: Vec<(Split, Transaction)>, target_account: &Account) {
        println!("Moving {} splits to {}", transactions.len(), target_account);
        for (split, tx) in transactions {
            println!(
                "[{}]<{}> - {} - {}",
                split.account_guid, split.tx_guid, tx, split
            );
        }
    }
}

impl<'a> From<&'a ArgMatches<'a>> for TransactionQuery {
    fn from(entries_cmd: &ArgMatches) -> Self {
        let limit = value_t!(entries_cmd, "limit", i64).unwrap_or(10);
        let txid_filter = value_t!(entries_cmd, "txid", String).ok();
        let description_filter = value_t!(entries_cmd, "description", String).ok();
        let memo_filter = value_t!(entries_cmd, "memo", String).ok();
        let before_filter = to_date(value_t!(entries_cmd, "before", String).ok());
        let after_filter = to_date(value_t!(entries_cmd, "after", String).ok());
        TransactionQuery {
            limit,
            txid_filter,
            account_filter: None,
            description_filter,
            memo_filter,
            before_filter,
            after_filter,
        }
    }
}
