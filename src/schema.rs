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
    billterms (guid) {
        guid -> Text,
        name -> Text,
        description -> Text,
        refcount -> Integer,
        invisible -> Integer,
        parent -> Nullable<Text>,
        #[sql_name = "type"]
        type_ -> Text,
        duedays -> Nullable<Integer>,
        discountdays -> Nullable<Integer>,
        discount_num -> Nullable<BigInt>,
        discount_denom -> Nullable<BigInt>,
        cutoff -> Nullable<Integer>,
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
    budget_amounts (id) {
        id -> Integer,
        budget_guid -> Text,
        account_guid -> Text,
        period_num -> Integer,
        amount_num -> BigInt,
        amount_denom -> BigInt,
    }
}

table! {
    budgets (guid) {
        guid -> Text,
        name -> Text,
        description -> Nullable<Text>,
        num_periods -> Integer,
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
    employees (guid) {
        guid -> Text,
        username -> Text,
        id -> Text,
        language -> Text,
        acl -> Text,
        active -> Integer,
        currency -> Text,
        ccard_guid -> Nullable<Text>,
        workday_num -> BigInt,
        workday_denom -> BigInt,
        rate_num -> BigInt,
        rate_denom -> BigInt,
        addr_name -> Nullable<Text>,
        addr_addr1 -> Nullable<Text>,
        addr_addr2 -> Nullable<Text>,
        addr_addr3 -> Nullable<Text>,
        addr_addr4 -> Nullable<Text>,
        addr_phone -> Nullable<Text>,
        addr_fax -> Nullable<Text>,
        addr_email -> Nullable<Text>,
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
    invoices (guid) {
        guid -> Text,
        id -> Text,
        date_opened -> Nullable<Text>,
        date_posted -> Nullable<Text>,
        notes -> Text,
        active -> Integer,
        currency -> Text,
        owner_type -> Nullable<Integer>,
        owner_guid -> Nullable<Text>,
        terms -> Nullable<Text>,
        billing_id -> Nullable<Text>,
        post_txn -> Nullable<Text>,
        post_lot -> Nullable<Text>,
        post_acc -> Nullable<Text>,
        billto_type -> Nullable<Integer>,
        billto_guid -> Nullable<Text>,
        charge_amt_num -> Nullable<BigInt>,
        charge_amt_denom -> Nullable<BigInt>,
    }
}

table! {
    jobs (guid) {
        guid -> Text,
        id -> Text,
        name -> Text,
        reference -> Text,
        active -> Integer,
        owner_type -> Nullable<Integer>,
        owner_guid -> Nullable<Text>,
    }
}

table! {
    lots (guid) {
        guid -> Text,
        account_guid -> Nullable<Text>,
        is_closed -> Integer,
    }
}

table! {
    orders (guid) {
        guid -> Text,
        id -> Text,
        notes -> Text,
        reference -> Text,
        active -> Integer,
        date_opened -> Text,
        date_closed -> Text,
        owner_type -> Integer,
        owner_guid -> Text,
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
    recurrences (id) {
        id -> Integer,
        obj_guid -> Text,
        recurrence_mult -> Integer,
        recurrence_period_type -> Text,
        recurrence_period_start -> Text,
        recurrence_weekend_adjust -> Text,
    }
}

table! {
    schedxactions (guid) {
        guid -> Text,
        name -> Nullable<Text>,
        enabled -> Integer,
        start_date -> Nullable<Text>,
        end_date -> Nullable<Text>,
        last_occur -> Nullable<Text>,
        num_occur -> Integer,
        rem_occur -> Integer,
        auto_create -> Integer,
        auto_notify -> Integer,
        adv_creation -> Integer,
        adv_notify -> Integer,
        instance_count -> Integer,
        template_act_guid -> Text,
    }
}

table! {
    slots (id) {
        id -> Integer,
        obj_guid -> Text,
        name -> Text,
        slot_type -> Integer,
        int64_val -> Nullable<BigInt>,
        string_val -> Nullable<Text>,
        double_val -> Nullable<Float>,
        timespec_val -> Nullable<Text>,
        guid_val -> Nullable<Text>,
        numeric_val_num -> Nullable<BigInt>,
        numeric_val_denom -> Nullable<BigInt>,
        gdate_val -> Nullable<Text>,
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
    taxtable_entries (id) {
        id -> Integer,
        taxtable -> Text,
        account -> Text,
        amount_num -> BigInt,
        amount_denom -> BigInt,
        #[sql_name = "type"]
        type_ -> Integer,
    }
}

table! {
    taxtables (guid) {
        guid -> Text,
        name -> Text,
        refcount -> BigInt,
        invisible -> Integer,
        parent -> Nullable<Text>,
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

table! {
    vendors (guid) {
        guid -> Text,
        name -> Text,
        id -> Text,
        notes -> Text,
        currency -> Text,
        active -> Integer,
        tax_override -> Integer,
        addr_name -> Nullable<Text>,
        addr_addr1 -> Nullable<Text>,
        addr_addr2 -> Nullable<Text>,
        addr_addr3 -> Nullable<Text>,
        addr_addr4 -> Nullable<Text>,
        addr_phone -> Nullable<Text>,
        addr_fax -> Nullable<Text>,
        addr_email -> Nullable<Text>,
        terms -> Nullable<Text>,
        tax_inc -> Nullable<Text>,
        tax_table -> Nullable<Text>,
    }
}

table! {
    versions (table_name) {
        table_name -> Text,
        table_version -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(
    accounts,
    billterms,
    books,
    budget_amounts,
    budgets,
    commodities,
    employees,
    entries,
    invoices,
    jobs,
    lots,
    orders,
    prices,
    recurrences,
    schedxactions,
    slots,
    splits,
    taxtable_entries,
    taxtables,
    transactions,
    vendors,
    versions,
);
