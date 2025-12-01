fn parse_sheet_row(row) {
    // row is an array of dynamic values (strings, floats, ints, bools, or UNIT for empty cells)

    // Example: assuming the sheet has Date, Amount, Description in columns 0, 1, 2
    let date_str = row[0].to_string();
    let amount_val = row[1].as_float().unwrap_or(0.0); // Safely get float, default to 0.0
    let description_str = row[2].to_string();

    // Create date directly using registered Rust function
    let date = create_naive_date_ymd(2023, 1, 15);

    // Convert amount float to Decimal using a registered Rust function
    let amount = new_decimal_from_f64(amount_val);

    // Create a new ExternalTransaction object using a registered Rust function
    new_external_transaction(
        date,               // date: Option<NaiveDate>
        (),                 // booking_date: Option<NaiveDate>
        amount,             // amount: Option<Decimal>
        (),                 // category: Option<String>
        description_str,    // description: Option<String>
        (),                 // other_account: Option<String>
        (),                 // other_account_name: Option<String>
        (),                 // textual_date: Option<NaiveDate>
        ()                  // transaction_fee: Option<Decimal>
    )
}
