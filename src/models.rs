use chrono::NaiveDateTime;
use schema::{accounts, commodities, splits, transactions};
use std::fmt;

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

#[derive(Queryable, Debug)]
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

#[derive(Queryable, Debug)]
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
        println!("[{}]<{}> - {}", self.account_type, self.guid, self.name);
    }
}

impl Split {
    pub fn is_equal_amount(&self, amount: f64) -> bool {
        /*        println!(
            "is_equal_amount {:?} {:?} ?= {}",
            self.value_num, self.value_denom, amount
        );*/
        (amount * (self.quantity_denom as f64)) as i64 == self.quantity_num
    }
}

impl fmt::Display for Split {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{} - {}({}) {}({})",
            self.memo,
            self.action,
            self.value_num,
            self.value_denom,
            self.quantity_num,
            self.quantity_denom
        )
    }
}

impl Transaction {
    pub fn posting(&self) -> Option<NaiveDateTime> {
        parse_date(&self.post_date)
    }
    pub fn entering(&self) -> Option<NaiveDateTime> {
        parse_date(&self.enter_date)
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(parsed_date) = parse_date(&self.post_date) {
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

fn parse_date(value: &Option<String>) -> Option<NaiveDateTime> {
    value
        .clone()
        .and_then(|date_str| NaiveDateTime::parse_from_str(date_str.as_ref(), "%Y%m%d%H%M%S").ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_parse_none() {
        assert_eq!(parse_date(&None), None);
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse_date(&Some("20161020203213".to_string())),
            Some(NaiveDate::from_ymd(2016, 10, 20).and_hms(20, 32, 13))
        );
    }

}
