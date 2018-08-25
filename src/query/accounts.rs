use clap::ArgMatches;
use diesel::prelude::*;

use models::Account;

pub struct AccountQuery {
    pub limit: i64,
    pub name_filter: Option<String>,
    pub parent_filter: Option<String>,
    pub type_filter: Option<String>,
}

impl AccountQuery {
    pub fn execute(&self, connection: &SqliteConnection) -> Vec<Account> {
        use schema::accounts::dsl::*;

        let mut query = accounts.into_boxed();
        if let Some(ref name_txt) = self.name_filter {
            query = query.filter(name.like(format!("%{}%", name_txt)));
        }
        if let Some(ref parent_txt) = self.parent_filter {
            query = query.filter(parent_guid.like(format!("%{}%", parent_txt)));
        }
        if let Some(ref type_txt) = self.type_filter {
            query = query.filter(account_type.like(format!("%{}%", type_txt)));
        }

        query
            .limit(self.limit)
            .load::<Account>(connection)
            .expect("Error loading accounts")
    }

    pub fn execute_and_display(&self, connection: &SqliteConnection) {
        let results = self.execute(&connection);
        println!("Displaying {} accounts", results.len());
        for account in results {
            println!(
                "[{}]<{}> - {}",
                account.account_type, account.guid, account.name
            );
        }
    }
}

impl<'a> From<&'a ArgMatches<'a>> for AccountQuery {
    fn from(ls_acc_cmd: &ArgMatches) -> Self {
        let limit = value_t!(ls_acc_cmd, "limit", i64).unwrap_or(10);
        let name_filter = value_t!(ls_acc_cmd, "name", String).ok();
        let parent_filter = value_t!(ls_acc_cmd, "parent_guid", String).ok();
        let type_filter = value_t!(ls_acc_cmd, "type", String).ok();
        AccountQuery {
            limit,
            name_filter,
            parent_filter,
            type_filter,
        }
    }
}
