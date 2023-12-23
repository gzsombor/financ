use anyhow::Result;
use chrono::naive::NaiveDate;
use console::{style, Term};
use diesel::prelude::*;

use crate::cli::TransactionsArgs;
use crate::models::{Account, Split, Transaction};
use crate::utils::{format_sqlite_date, to_date};

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
        use crate::schema::splits::dsl::*;
        use crate::schema::transactions::dsl::*;

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
            let after_as_txt = format_sqlite_date(&after_date.and_hms_opt(0, 0, 0).expect("Correct date"));
            query = query.filter(post_date.ge(after_as_txt));
        }
        if let Some(before_date) = self.before_filter {
            let before_as_txt = format_sqlite_date(&before_date.and_hms_opt(23, 59, 59).expect("Correct date"));
            query = query.filter(post_date.le(before_as_txt));
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
        term: &Term,
    ) -> Result<usize> {
        let results = self.execute(connection);
        match target_account {
            None => self.display(results),
            Some(account) => self.move_splits(connection, results, account, term),
        }
    }

    fn display(&self, transactions: Vec<(Split, Transaction)>) -> Result<usize> {
        let len = transactions.len();
        println!("Displaying {} splits", len);
        for (split, tx) in transactions {
            println!(
                "[{}]<{}> - {} - {}",
                split.account_guid, split.tx_guid, tx, split
            );
        }
        Ok(len)
    }

    fn move_splits(
        &self,
        connection: &SqliteConnection,
        transactions: Vec<(Split, Transaction)>,
        target_account: &Account,
        term: &Term,
    ) -> Result<usize> {
        use crate::schema::splits::dsl::{account_guid, splits};
        let len = transactions.len();
        term.write_line(&format!(
            "Moving {} splits to {}",
            style(len).cyan(),
            style(target_account).blue()
        ))?;
        for (split, tx) in transactions {
            println!(
                "[{}]<{}> - {} - {}",
                split.account_guid, split.tx_guid, tx, split
            );
            let res = diesel::update(splits.find(split.guid))
                .set(account_guid.eq(&target_account.guid))
                .execute(connection);
            assert_eq!(1, res.unwrap());
        }
        Ok(len)
    }
}

impl From<TransactionsArgs> for TransactionQuery {
    fn from(args: TransactionsArgs) -> Self {
        TransactionQuery {
            limit: args.limit.unwrap_or(10),
            txid_filter: args.txid,
            account_filter: None,
            description_filter: args.description,
            memo_filter: args.memo,
            before_filter: to_date(args.before),
            after_filter: to_date(args.after),
        }
    }
}
