use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use guid_create::GUID;
use rust_decimal::Decimal;

use crate::models::{Account, Commodities};
use crate::schema::{splits, transactions};
use crate::utils::{DenominatedValue, format_guid, format_sqlite_date};

#[derive(Insertable, Debug)]
#[diesel(table_name = splits)]
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

#[derive(Insertable, Debug)]
#[diesel(table_name = transactions)]
pub struct NewTransaction<'a> {
    pub guid: &'a str,
    pub currency_guid: &'a str,
    pub num: &'a str,
    pub post_date: &'a str,
    pub enter_date: &'a str,
    pub description: &'a str,
}

impl<'a> NewSplit<'a> {
    fn new_with_defaults(
        guid: &'a str,
        tx_guid: &'a str,
        account_guid: &'a str,
        memo: &'a str,
        value: &DenominatedValue,
        quantity: &DenominatedValue,
    ) -> Self {
        NewSplit {
            guid,
            tx_guid,
            account_guid,
            memo,
            action: "",
            reconcile_state: "n",
            reconcile_date: "",
            value_num: value.value,
            value_denom: value.denom,
            quantity_num: quantity.value,
            quantity_denom: quantity.denom,
            lot_guid: "",
        }
    }

    fn create_split(
        split_guid: &'a str,
        tx_guid: &'a str,
        account: &'a Account,
        memo: &'a str,
        currency: &Commodities,
        amount: Decimal,
    ) -> Result<Self> {
        let value = DenominatedValue::denominate_decimal(amount, currency.fraction)
            .context("Failed to denominate value")?;
        let qty = DenominatedValue::denominate_decimal(amount, account.commodity_scu)
            .context("Failed to denominate quantity")?;
        Ok(NewSplit::new_with_defaults(
            split_guid,
            tx_guid,
            &account.guid,
            memo,
            &value,
            &qty,
        ))
    }

    pub fn insert(
        connection: &mut SqliteConnection,
        tx_guid: &'a str,
        account: &'a Account,
        memo: &'a str,
        currency: &Commodities,
        amount: Decimal,
    ) -> Result<String> {
        let split_guid = format_guid(&GUID::rand().to_string());
        {
            let split =
                NewSplit::create_split(&split_guid, tx_guid, account, memo, currency, amount)?;

            let inserted_rows = diesel::insert_into(splits::table)
                .values(&split)
                .execute(connection)
                .context("Error saving new split")?;
            if inserted_rows != 1 {
                anyhow::bail!("Inserting split inserted {inserted_rows} rows, expected 1");
            }
        }
        Ok(split_guid)
    }
}

impl<'a> NewTransaction<'a> {
    pub fn new(
        guid: &'a str,
        currency_guid: &'a str,
        post_date: &'a str,
        enter_date: &'a str,
        description: &'a str,
    ) -> Self {
        NewTransaction {
            guid,
            currency_guid,
            num: "",
            post_date,
            enter_date,
            description,
        }
    }

    pub fn insert(
        connection: &mut SqliteConnection,
        guid: &'a str,
        currency_guid: &'a str,
        post_date: Option<NaiveDateTime>,
        enter_date: NaiveDateTime,
        description: &'a str,
    ) -> Result<usize> {
        let formatted_date = post_date
            .map(|x| format_sqlite_date(&x))
            .unwrap_or_default();
        let formatted_enter_date = format_sqlite_date(&enter_date);
        let transaction = NewTransaction::new(
            guid,
            currency_guid,
            &formatted_date,
            &formatted_enter_date,
            description,
        );

        let inserted_rows = diesel::insert_into(transactions::table)
            .values(transaction)
            .execute(connection)
            .context("Error saving transaction")?;
        if inserted_rows != 1 {
            anyhow::bail!("Inserting transaction inserted {inserted_rows} rows, expected 1");
        }
        Ok(inserted_rows)
    }
}
