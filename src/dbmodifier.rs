use diesel::prelude::*;
use guid_create::GUID;

use models::{Account, Commodities};
use schema::{splits, transactions};

#[derive(Insertable, Debug)]
#[table_name = "splits"]
pub struct NewSplit<'a> {
    pub guid: &'a str,
    pub tx_guid: &'a str,
    pub account_guid: &'a str,
    pub memo: &'a str,
    pub action: &'a str,
    pub reconcile_state: &'a str,
    pub reconcile_date: &'a str,

    pub value_num: i64,
    pub value_denom: i64,
    pub quantity_num: i64,
    pub quantity_denom: i64,
    pub lot_guid: &'a str,
}

#[derive(Insertable)]
#[table_name = "transactions"]
pub struct NewTransaction<'a> {
    pub guid: &'a str,
    pub currency_guid: &'a str,
    pub num: &'a str,
    pub post_date: &'a str,
    pub enter_date: &'a str,
    pub description: &'a str,
}

impl<'a> NewSplit<'a> {
    fn simple(
        guid: &'a str,
        tx_guid: &'a str,
        account_guid: &'a str,
        memo: &'a str,
        value_num: i64,
        value_denom: i64,
        quantity_num: i64,
        quantity_denom: i64,
    ) -> Self {
        NewSplit {
            guid,
            tx_guid,
            account_guid,
            memo,
            action: "",
            reconcile_state: "n",
            reconcile_date: "",
            value_num,
            value_denom,
            quantity_num,
            quantity_denom,
            lot_guid: "",
        }
    }

    pub fn create(
        split_guid: &'a str,
        tx_guid: &'a str,
        account: &'a Account,
        memo: &'a str,
        currency: &Commodities,
        amount: f64,
    ) -> Self {
        let fraction = f64::from(currency.fraction);
        let value_num = ((fraction * amount).round()) as i64;
        let value_denom = i64::from(currency.fraction);
        let account_qty = (f64::from(account.commodity_scu) * amount) as i64;
        NewSplit::simple(
            split_guid,
            tx_guid,
            &account.guid,
            memo,
            value_num,
            value_denom,
            account_qty,
            i64::from(account.commodity_scu),
        )
    }

    pub fn insert(
        connection: &SqliteConnection,
        tx_guid: &'a str,
        account: &'a Account,
        memo: &'a str,
        currency: &Commodities,
        amount: f64,
    ) -> usize {
        use schema::splits;

        let split_guid = GUID::rand().to_string();
        let split = NewSplit::create(&split_guid, tx_guid, account, memo, currency, amount);

        diesel::insert_into(splits::table)
            .values(&split)
            .execute(connection)
            .expect("Error saving new split")
    }
}
