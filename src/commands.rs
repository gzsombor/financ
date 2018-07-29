// use self::financ::*;
// #use self::financ::models::*;
use diesel::prelude::*;
use models::Account;

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
