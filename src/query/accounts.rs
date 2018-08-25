use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use diesel::prelude::*;

use models::Account;

pub struct AccountQuery {
    pub limit: i64,
    pub guid_filter: Option<String>,
    pub name_filter: Option<String>,
    pub parent_filter: Option<String>,
    pub type_filter: Option<String>,
}

impl AccountQuery {
    pub fn execute(&self, connection: &SqliteConnection) -> Vec<Account> {
        use schema::accounts::dsl::*;

        let mut query = accounts.into_boxed();
        if let Some(ref guid_txt) = self.guid_filter {
            query = query.filter(guid.like(format!("%{}%", guid_txt)));
        }
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

    pub fn add_arguments<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        app.arg(
            Arg::with_name("name")
                .short("n")
                .long("name")
                .help("Limit to accounts which name contains the specified string")
                .required(false)
                .takes_value(true),
        ).arg(
            Arg::with_name("parent_guid")
                .short("p")
                .long("parent")
                .help("Filter to the childs accounts of the given account")
                .required(false)
                .takes_value(true),
        ).arg(
            Arg::with_name("guid")
                .short("g")
                .long("guid")
                .help("Filter by guid")
                .required(false)
                .takes_value(true),
        ).arg(
            Arg::with_name("type")
                .short("t")
                .long("type")
                .help("Limit to specified account types")
                .required(false)
                .takes_value(true),
        )
    }
}

impl<'a> From<&'a ArgMatches<'a>> for AccountQuery {
    fn from(ls_acc_cmd: &ArgMatches) -> Self {
        let limit = value_t!(ls_acc_cmd, "limit", i64).unwrap_or(10);
        let name_filter = value_t!(ls_acc_cmd, "name", String).ok();
        let guid_filter = value_t!(ls_acc_cmd, "guid", String).ok();
        let parent_filter = value_t!(ls_acc_cmd, "parent_guid", String).ok();
        let type_filter = value_t!(ls_acc_cmd, "type", String).ok();
        AccountQuery {
            limit,
            guid_filter,
            name_filter,
            parent_filter,
            type_filter,
        }
    }
}
