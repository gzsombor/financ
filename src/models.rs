use std::fmt;

use chrono::NaiveDateTime;

use crate::schema::{accounts, splits, transactions};
use crate::utils::{get_value_or_empty, parse_sqlite_date};

joinable!(splits -> transactions (tx_guid));
joinable!(splits -> accounts (account_guid));

#[derive(Queryable)]
pub struct Account {
    pub guid: String,
    pub name: String,
    pub account_type: String,
    pub commodity_guid: Option<String>,
    pub commodity_scu: i32,
    pub non_std_scu: i32,
    pub parent_guid: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub hidden: Option<i32>,
    pub placeholder: Option<i32>,
}

#[derive(Queryable, Selectable, Debug, PartialEq)]
#[diesel(table_name = splits)]
#[diesel(belongs_to(Transaction))]
pub struct Split {
    pub guid: String,
    pub tx_guid: String,
    pub account_guid: String,
    pub memo: String,
    pub action: String,
    pub reconcile_state: String,
    pub reconcile_date: Option<String>,

    pub value_num: i64,
    pub value_denom: i64,
    pub quantity_num: i64,
    pub quantity_denom: i64,
    pub lot_guid: Option<String>,
}

#[derive(Queryable, Selectable, Debug, PartialEq)]
#[diesel(table_name = transactions)]
pub struct Transaction {
    pub guid: String,
    pub currency_guid: String,
    pub num: String,
    pub post_date: Option<String>,
    pub enter_date: Option<String>,
    pub description: Option<String>,
}

#[derive(Queryable, Debug)]
pub struct Commodities {
    pub guid: String,
    pub namespace: String,
    pub mnemonic: String,
    pub fullname: Option<String>,
    pub cusip: Option<String>,
    pub fraction: i32,
    pub quote_flag: i32,
    pub quote_source: Option<String>,
    pub quote_tz: Option<String>,
}

impl Account {
    pub fn display(&self) {
        println!(
            "[{}]<id= {}>(parent= {},commodity= {}) - {} {}",
            self.account_type,
            self.guid,
            get_value_or_empty(&self.parent_guid),
            get_value_or_empty(&self.commodity_guid),
            self.name,
            get_value_or_empty(&self.description)
        );
    }
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({})", self.name, self.guid)
    }
}

impl Split {
    pub fn is_equal_amount(&self, amount: f64) -> bool {
        (amount * (self.quantity_denom as f64)) as i64 == self.quantity_num
    }

    pub fn get_value(&self) -> f64 {
        (self.value_num as f64) / (self.value_denom as f64)
    }

    pub fn get_quantity(&self) -> f64 {
        (self.quantity_num as f64) / (self.quantity_denom as f64)
    }
}

impl fmt::Display for Split {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{} - {} {}",
            self.memo,
            self.action,
            self.get_value(),
            self.get_quantity(),
        )
    }
}

impl Transaction {
    pub fn posting(&self) -> Option<NaiveDateTime> {
        parse_sqlite_date(&self.post_date)
    }
    pub fn entering(&self) -> Option<NaiveDateTime> {
        parse_sqlite_date(&self.enter_date)
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(parsed_date) = parse_sqlite_date(&self.post_date) {
            f.write_str(&parsed_date.format("%Y-%m-%d").to_string())?;
        } else {
            f.write_str("--------")?;
        }
        if let Some(ref desc) = self.description {
            write!(f, " - {}", desc)?;
        }
        Ok(())
    }
}

impl Commodities {
    pub fn display(&self) {
        println!(
            "[{}]<{}> - {} (fraction:{})",
            self.namespace, self.guid, self.mnemonic, self.fraction
        );
    }
}
