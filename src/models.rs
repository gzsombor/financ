// use super::schema::accounts;

// CREATE TABLE accounts (
//  guid text(32) PRIMARY KEY NOT NULL,
// name text(2048) NOT NULL,
// account_type text(2048) NOT NULL,
// commodity_guid text(32),
// commodity_scu integer NOT NULL,
// non_std_scu integer NOT NULL,
// parent_guid text(32),
// code text(2048),
// description text(2048),
// hidden integer,
// placeholder integer);

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

#[derive(Queryable)]
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
