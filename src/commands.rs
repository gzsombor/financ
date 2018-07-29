// use self::financ::*;
// #use self::financ::models::*;
use diesel::prelude::*;
use models::{Account, Split};

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
) {
    use schema::splits::dsl::*;

    let mut query = splits.into_boxed();
    if let Some(txid_txt) = txid_filter {
        query = query.filter(tx_guid.like(format!("%{}%", txid_txt)));
    }
    if let Some(guid_txt) = guid_filter {
        query = query.filter(account_guid.like(format!("%{}%", guid_txt)));
    }
    if let Some(name_txt) = name_filter {
        query = query.filter(memo.like(format!("%{}%", name_txt)));
    }

    let results = query
        .limit(limit)
        .load::<Split>(connection)
        .expect("Error loading splits");

    println!("Displaying {} splits", results.len());
    for split in results {
        println!(
            "[{}]<{}> - {}({}) {}({})",
            split.account_guid,
            split.tx_guid,
            split.value_num,
            split.value_denom,
            split.quantity_num,
            split.quantity_denom
        );
    }
}
