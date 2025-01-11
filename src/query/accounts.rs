use std::fmt;

use diesel::prelude::*;

use crate::{
    cli::{DefaultAccountParams, FeeAccountParams, FromAccountParams, TargetAccountParams},
    models::Account, schema::commodities,
};

#[derive(Debug)]
pub struct AccountQuery {
    pub limit: i64,
    pub guid_filter: Option<String>,
    pub name_filter: Option<String>,
    pub parent_filter: Option<String>,
    pub type_filter: Option<String>,
    pub parent_name_filter: Option<String>,
    pub commodity_id_filter: Option<String>,
    pub commodity_name_filter: Option<String>,
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
            parent_name_filter: self.parent_name.clone(),
            commodity_id_filter: self.commodity_id.clone(),
            commodity_name_filter: self.commodity_name.clone(),
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
            parent_name_filter: self.target_parent_name.clone(),
            commodity_id_filter: self.commodity_id.clone(),
            commodity_name_filter: self.commodity_name.clone(),
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
            parent_name_filter: self.from_parent_name.clone(),
            commodity_id_filter: self.commodity_id.clone(),
            commodity_name_filter: self.commodity_name.clone(),
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
            parent_name_filter: self.fee_parent_name.clone(),
            commodity_id_filter: self.commodity_id.clone(),
            commodity_name_filter: self.commodity_name.clone(),
        }
    }
}

impl AccountQuery {
    pub fn execute(&self, connection: &mut SqliteConnection) -> Vec<Account> {
        use crate::schema::accounts;

        let mut query = accounts::table.into_boxed();
        if let Some(ref guid_txt) = self.guid_filter {
            query = query.filter(accounts::guid.like(format!("%{}%", guid_txt)));
        }
        if let Some(ref name_txt) = self.name_filter {
            query = query.filter(accounts::name.like(format!("%{}%", name_txt)));
        }
        if let Some(ref parent_txt) = self.parent_filter {
            query = query.filter(accounts::parent_guid.like(format!("%{}%", parent_txt)));
        }
        if let Some(ref type_txt) = self.type_filter {
            query = query.filter(accounts::account_type.like(format!("%{}%", type_txt)));
        }
        if let Some(ref parent_name_txt) = self.parent_name_filter {
            let subquery = accounts::table
                .filter(accounts::name.like(format!("%{}%", parent_name_txt)))
                .select(accounts::guid.nullable())
                .into_boxed();
            query = query.filter(accounts::parent_guid.eq_any(subquery));
        }
        if let Some(ref commodity_id) = self.commodity_id_filter {
            query = query.filter(accounts::commodity_guid.like(format!("%{}%", commodity_id)));
        }
        if let Some(ref commodity_name) = self.commodity_name_filter {
            let subquery = commodities::table
                .filter(commodities::fullname.like(format!("%{}%", commodity_name)))
                .select(commodities::guid.nullable())
                .into_boxed();
            query = query.filter(accounts::commodity_guid.eq_any(subquery));
        }

        query
            .limit(self.limit)
            .load::<Account>(connection)
            .expect("Error loading accounts")
    }

    pub fn execute_and_display(&self, connection: &mut SqliteConnection) {
        let results = self.execute(connection);
        println!("Displaying {} accounts", results.len());
        for account in results {
            account.display();
        }
    }

    pub fn get_one(
        &self,
        connection: &mut SqliteConnection,
        show_warning: bool,
    ) -> Option<Account> {
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
