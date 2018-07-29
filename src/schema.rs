table! {
    accounts (guid) {
        guid -> Text,
        name -> Text,
        account_type -> Text,
        commodity_guid -> Nullable<Text>,
        commodity_scu -> Integer,
        non_std_scu -> Integer,
        parent_guid -> Nullable<Text>,
        code -> Nullable<Text>,
        description -> Nullable<Text>,
        hidden -> Nullable<Integer>,
        placeholder -> Nullable<Integer>,
    }
}
table! {
    books (guid) {
        guid -> Text,
        root_account_guid -> Text,
        root_template_guid -> Text,
    }
}

table! {
    commodities (guid) {
        guid -> Text,
        namespace -> Text,
        mnemonic -> Text,
        fullname -> Nullable<Text>,
        cusip -> Nullable<Text>,
        fraction -> Integer,
        quote_flag -> Integer,
        quote_source -> Nullable<Text>,
        quote_tz -> Nullable<Text>,
    }
}
table! {
    entries (guid) {
        guid -> Text,
        date -> Text,
        date_entered -> Nullable<Text>,
        description -> Nullable<Text>,
        action -> Nullable<Text>,
        notes -> Nullable<Text>,
        quantity_num -> Nullable<BigInt>,
        quantity_denom -> Nullable<BigInt>,
        i_acct -> Nullable<Text>,
        i_price_num -> Nullable<BigInt>,
        i_price_denom -> Nullable<BigInt>,
        i_discount_num -> Nullable<BigInt>,
        i_discount_denom -> Nullable<BigInt>,
        invoice -> Nullable<Text>,
        i_disc_type -> Nullable<Text>,
        i_disc_how -> Nullable<Text>,
        i_taxable -> Nullable<Integer>,
        i_taxincluded -> Nullable<Integer>,
        i_taxtable -> Nullable<Text>,
        b_acct -> Nullable<Text>,
        b_price_num -> Nullable<BigInt>,
        b_price_denom -> Nullable<BigInt>,
        bill -> Nullable<Text>,
        b_taxable -> Nullable<Integer>,
        b_taxincluded -> Nullable<Integer>,
        b_taxtable -> Nullable<Text>,
        b_paytype -> Nullable<Integer>,
        billable -> Nullable<Integer>,
        billto_type -> Nullable<Integer>,
        billto_guid -> Nullable<Text>,
        order_guid -> Nullable<Text>,
    }
}
table! {
    prices (guid) {
        guid -> Text,
        commodity_guid -> Text,
        currency_guid -> Text,
        date -> Text,
        source -> Nullable<Text>,
        #[sql_name = "type"]
        type_ -> Nullable<Text>,
        value_num -> BigInt,
        value_denom -> BigInt,
    }
}
table! {
    splits (guid) {
        guid -> Text,
        tx_guid -> Text,
        account_guid -> Text,
        memo -> Text,
        action -> Text,
        reconcile_state -> Text,
        reconcile_date -> Nullable<Text>,
        value_num -> BigInt,
        value_denom -> BigInt,
        quantity_num -> BigInt,
        quantity_denom -> BigInt,
        lot_guid -> Nullable<Text>,
    }
}

table! {
    transactions (guid) {
        guid -> Text,
        currency_guid -> Text,
        num -> Text,
        post_date -> Nullable<Text>,
        enter_date -> Nullable<Text>,
        description -> Nullable<Text>,
    }
}
allow_tables_to_appear_in_same_query!(
    accounts,
    books,
    commodities,
    entries,
    prices,
    splits,
    transactions,
);
