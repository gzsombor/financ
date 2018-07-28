// use self::financ::*;
// #use self::financ::models::*;
use diesel::prelude::*;
use models::Account;

pub fn list_accounts(connection: &SqliteConnection, limit: i64) {
    use schema::accounts::dsl::*;

    let results = accounts
        .limit(limit)
        .load::<Account>(connection)
        .expect("Error loading accounts");

    println!("Displaying {} accounts", results.len());
    for account in results {
        println!("{} - {}", account.account_type, account.name);
    }
}