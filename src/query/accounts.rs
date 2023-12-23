use std::fmt;

use diesel::prelude::*;

use crate::{
    cli::{DefaultAccountParams, FeeAccountParams, FromAccountParams, TargetAccountParams},
    models::Account,
};

#[derive(Debug)]
pub struct AccountQuery {
    pub limit: i64,
    pub guid_filter: Option<String>,
    pub name_filter: Option<String>,
    pub parent_filter: Option<String>,
    pub type_filter: Option<String>,
}

pub(crate) trait ToAccountQuery {
    fn build(&self, limit: Option<i64>) -> AccountQuery;
}

impl ToAccountQuery for DefaultAccountParams {
    fn build(&self, limit: Option<i64>) -> AccountQuery {
        AccountQuery {
            limit: limit.unwrap_or(10),
            guid_filter: self.guid.clone(),
            name_filter: self.name.clone(),
            parent_filter: self.parent_guid.clone(),
            type_filter: self.account_type.clone(),
        }
    }
}

impl ToAccountQuery for TargetAccountParams {
    fn build(&self, limit: Option<i64>) -> AccountQuery {
        AccountQuery {
            limit: limit.unwrap_or(10),
            guid_filter: self.target_guid.clone(),
            name_filter: self.target_name.clone(),
            parent_filter: self.target_parent_guid.clone(),
            type_filter: self.target_account_type.clone(),
        }
    }
}

impl ToAccountQuery for FromAccountParams {
    fn build(&self, limit: Option<i64>) -> AccountQuery {
        AccountQuery {
            limit: limit.unwrap_or(10),
            guid_filter: self.from_guid.clone(),
            name_filter: self.from_name.clone(),
            parent_filter: self.from_parent_guid.clone(),
            type_filter: self.from_account_type.clone(),
        }
    }
}

impl ToAccountQuery for FeeAccountParams {
    fn build(&self, limit: Option<i64>) -> AccountQuery {
        AccountQuery {
            limit: limit.unwrap_or(10),
            guid_filter: self.fee_guid.clone(),
            name_filter: self.fee_name.clone(),
            parent_filter: self.fee_parent_guid.clone(),
            type_filter: self.fee_account_type.clone(),
        }
    }
}

impl AccountQuery {
    pub fn execute(&self, connection: &SqliteConnection) -> Vec<Account> {
        use crate::schema::accounts::dsl::*;

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
        let results = self.execute(connection);
        println!("Displaying {} accounts", results.len());
        for account in results {
            account.display();
        }
    }

    pub fn get_one(&self, connection: &SqliteConnection, show_warning: bool) -> Option<Account> {
        let mut account_list = self.execute(connection);
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

impl fmt::Display for AccountQuery {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "limit:{}", self.limit)?;
        if let Some(ref name_filter) = self.name_filter {
            write!(f, " name-filter:{}", name_filter)?;
        }
        if let Some(ref guid_filter) = self.guid_filter {
            write!(f, " guid-filter:{}", guid_filter)?;
        }
        if let Some(ref parent_filter) = self.parent_filter {
            write!(f, " parent-filter:{}", parent_filter)?;
        }
        if let Some(ref type_filter) = self.type_filter {
            write!(f, " type-filter:{}", type_filter)?;
        }
        Ok(())
    }
}
