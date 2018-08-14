use chrono::prelude::*;
use diesel::prelude::*;
use models::{Account, Split, Transaction};

pub struct TransactionQuery {
    pub limit: i64,
    pub txid_filter: Option<String>,
    pub account_filter: Option<String>,
    pub description_filter: Option<String>,
    pub memo_filter: Option<String>,
    pub before_filter: Option<NaiveDate>,
    pub after_filter: Option<NaiveDate>,
}

pub fn list_accounts(
    connection: &SqliteConnection,
    limit: i64,
    name_filter: Option<String>,
    parent_filter: Option<String>,
    type_filter: Option<String>,
) {
    use schema::accounts::dsl::*;

    let mut query = accounts.into_boxed();
    if let Some(name_txt) = name_filter {
        query = query.filter(name.like(format!("%{}%", name_txt)));
    }
    if let Some(parent_txt) = parent_filter {
        query = query.filter(parent_guid.like(format!("%{}%", parent_txt)));
    }
    if let Some(type_txt) = type_filter {
        query = query.filter(account_type.like(format!("%{}%", type_txt)));
    }

    let results = query
        .limit(limit)
        .load::<Account>(connection)
        .expect("Error loading accounts");

    println!("Displaying {} accounts", results.len());
    for account in results {
        println!(
            "[{}]<{}> - {}",
            account.account_type, account.guid, account.name
        );
    }
}

impl TransactionQuery {
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

    pub fn execute_and_display(&self, connection: &SqliteConnection) {
        let results = self.execute(&connection);
        println!("Displaying {} splits", results.len());
        for (split, tx) in results {
            println!(
                "[{}]<{}> - @{} - '{}' - {} - {}({}) {}({})",
                split.account_guid,
                split.tx_guid,
                tx.post_date.unwrap_or_else(|| "".to_string()),
                tx.description.unwrap_or_else(|| "".to_string()),
                split.memo,
                split.value_num,
                split.value_denom,
                split.quantity_num,
                split.quantity_denom
            );
        }
    }
}
