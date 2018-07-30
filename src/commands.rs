use chrono::prelude::*;
use diesel::prelude::*;
use models::{Account, Split, Transaction};

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

pub fn list_entries(
    connection: &SqliteConnection,
    limit: i64,
    txid_filter: Option<String>,
    guid_filter: Option<String>,
    name_filter: Option<String>,
    before_filter: Option<NaiveDate>,
    after_filter: Option<NaiveDate>,
) {
    use schema::splits::dsl::*;
    use schema::transactions::dsl::*;

    let join = splits.inner_join(transactions);

    let mut query = join.into_boxed();
    if let Some(txid_txt) = txid_filter {
        query = query.filter(tx_guid.like(format!("%{}%", txid_txt)));
    }
    if let Some(guid_txt) = guid_filter {
        query = query.filter(account_guid.like(format!("%{}%", guid_txt)));
    }
    if let Some(name_txt) = name_filter {
        query = query.filter(memo.like(format!("%{}%", name_txt)));
    }
    if let Some(after_date) = after_filter {
        let after_as_txt = after_date.and_hms(0, 0, 0).format("%Y%m%d%H%M%S");
        query = query.filter(post_date.ge(after_as_txt.to_string()));
    }
    if let Some(before_date) = before_filter {
        let before_as_txt = before_date.and_hms(23, 59, 59).format("%Y%m%d%H%M%S");
        query = query.filter(post_date.le(before_as_txt.to_string()));
    }

    let results = query
        .limit(limit)
        .load::<(Split, Transaction)>(connection)
        .expect("Error loading splits");

    println!("Displaying {} splits", results.len());
    for (split, tx) in results {
        println!(
            "[{}]<{}> - @{} - '{}' - {}({}) {}({})",
            split.account_guid,
            split.tx_guid,
            tx.post_date.unwrap_or_else(|| "".to_string()),
            tx.description.unwrap_or_else(|| "".to_string()),
            split.value_num,
            split.value_denom,
            split.quantity_num,
            split.quantity_denom
        );
    }
}
