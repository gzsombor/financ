use clap::{App, Arg, ArgMatches};
use diesel::prelude::*;

use models::Account;

#[derive(Debug)]
pub struct AccountQuery {
    pub limit: i64,
    pub guid_filter: Option<String>,
    pub name_filter: Option<String>,
    pub parent_filter: Option<String>,
    pub type_filter: Option<String>,
}

pub struct AccountQueryCli {
    name: &'static str,
    name_long: &'static str,
    name_short: &'static str,
    parent_guid: &'static str,
    parent_guid_long: &'static str,
    parent_guid_short: &'static str,
    guid: &'static str,
    guid_long: &'static str,
    guid_short: &'static str,
    type_name: &'static str,
    type_name_long: &'static str,
    type_name_short: &'static str,
}

pub const DEFAULT_ACCOUNT_PARAMS: AccountQueryCli = AccountQueryCli {
    name: "name",
    name_long: "account-name",
    name_short: "n",
    parent_guid: "parent_guid",
    parent_guid_long: "account-parent",
    parent_guid_short: "p",
    guid: "guid",
    guid_long: "account-guid",
    guid_short: "g",
    type_name: "type",
    type_name_long: "account-type",
    type_name_short: "t",
};

pub const FROM_ACCOUNT_PARAMS: AccountQueryCli = AccountQueryCli {
    name: "from_name",
    name_long: "from-account-name",
    name_short: "N",
    parent_guid: "from_parent_guid",
    parent_guid_long: "from-account-parent",
    parent_guid_short: "P",
    guid: "from_guid",
    guid_long: "from-account-guid",
    guid_short: "G",
    type_name: "from_type",
    type_name_long: "from-account-type",
    type_name_short: "T",
};

pub const TARGET_ACCOUNT_PARAMS: AccountQueryCli = AccountQueryCli {
    name: "target_name",
    name_long: "target-account-name",
    name_short: "r",
    parent_guid: "target_parent_guid",
    parent_guid_long: "target-account-parent",
    parent_guid_short: "P",
    guid: "target_guid",
    guid_long: "target-account-guid",
    guid_short: "G",
    type_name: "target_type",
    type_name_long: "target-account-type",
    type_name_short: "T",
};

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
            account.display();
        }
    }

    pub fn get_one(&self, connection: &SqliteConnection, show_warning: bool) -> Option<Account> {
        let mut account_list = self.execute(&connection);
        if account_list.len() != 1 {
            if show_warning {
                println!(
                    "Account filter should pick only one account, found : {}",
                    &account_list.len()
                );
                for acc in account_list {
                    acc.display();
                }
            }
            return None;
        }
        account_list.pop()
    }
}

impl AccountQueryCli {
    pub fn add_arguments<'a, 'b>(&self, app: App<'a, 'b>) -> App<'a, 'b> {
        app.arg(
            Arg::with_name(self.name)
                .short(self.name_short)
                .long(self.name_long)
                .help("Limit to accounts which name contains the specified string")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(self.parent_guid)
                .short(self.parent_guid_short)
                .long(self.parent_guid_long)
                .help("Filter to the childs accounts of the given account")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(self.guid)
                .short(self.guid_short)
                .long(self.guid_long)
                .help("Filter by guid")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(self.type_name)
                .short(self.type_name_short)
                .long(self.type_name_long)
                .help("Limit to specified account types")
                .required(false)
                .takes_value(true),
        )
    }

    pub fn build(&self, ls_acc_cmd: &ArgMatches, limit_param: Option<&str>) -> AccountQuery {
        let limit = if let Some(limit) = limit_param {
            value_t!(ls_acc_cmd, limit, i64).unwrap_or(10)
        } else {
            10
        };
        let name_filter = value_t!(ls_acc_cmd, self.name, String).ok();
        let guid_filter = value_t!(ls_acc_cmd, self.guid, String).ok();
        let parent_filter = value_t!(ls_acc_cmd, self.parent_guid, String).ok();
        let type_filter = value_t!(ls_acc_cmd, self.type_name, String).ok();
        AccountQuery {
            limit,
            guid_filter,
            name_filter,
            parent_filter,
            type_filter,
        }
    }
}
