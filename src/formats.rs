use std::sync::Arc;

#[cfg(test)]
use std::path::Path;

use crate::external_models::{ExternalTransaction, ExternalTransactionBuilder, SheetParser};
use crate::sheets::{
    cell_to_date, cell_to_datetime, cell_to_decimal, cell_to_english_date, cell_to_german_date,
    cell_to_iso_date, cell_to_string,
};
use crate::utils::extract_date;
use anyhow::Result;
use calamine::{Data, DataType, Range};
use chrono::{NaiveDate, NaiveDateTime};
use rhai::{AST, Dynamic, Engine, Scope};

use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

#[derive(Debug, Clone)]
pub enum SheetFormat {
    Otp,
    Otp2020,
    Granit,
    BankAustria,
    Transferwise,
    Magnet,
    Rhai { executor: Arc<Executor> },
}

impl SheetFormat {
    pub fn new(name: &str) -> Option<SheetFormat> {
        match name.to_lowercase().as_str() {
            "otp" => Some(SheetFormat::Otp),
            "otp2020" => Some(SheetFormat::Otp2020),
            "granit" => Some(SheetFormat::Granit),
            "bankaustria" => Some(SheetFormat::BankAustria),
            "transferwise" => Some(SheetFormat::Transferwise),
            "magnet" => Some(SheetFormat::Magnet),
            _ => None,
        }
    }

    #[allow(clippy::arc_with_non_send_sync)]
    pub fn new_rhai(ast: AST, parser_fn: String) -> SheetFormat {
        let mut executor = Executor::new();
        executor.ast = Some(ast);
        executor.parser_fn = Some(parser_fn);
        SheetFormat::Rhai {
            executor: Arc::new(executor),
        }
    }
}

impl SheetParser for SheetFormat {
    #[allow(clippy::too_many_lines)]
    fn parse_sheet(&self, range: &Range<Data>) -> Result<Vec<ExternalTransaction>> {
        match self {
            SheetFormat::Otp => Ok(range
                .rows()
                .filter(|row| row[0] != Data::Empty)
                .map(|row| {
                    let descrip = cell_to_string(&row[8]);
                    let parsed_date = extract_date(&descrip);
                    ExternalTransaction {
                        date: cell_to_date(&row[2]),
                        booking_date: cell_to_date(&row[3]),
                        amount: cell_to_decimal(&row[4]),
                        category: cell_to_string(&row[1]),
                        description: descrip,
                        other_account: cell_to_string(&row[6]),
                        other_account_name: cell_to_string(&row[7]),
                        textual_date: parsed_date,
                        transaction_fee: None,
                    }
                })
                .collect()),
            SheetFormat::Otp2020 => Ok(range
                .rows()
                .filter(|row| row[0] != Data::Empty)
                .map(|row| {
                    let spend_date = cell_to_datetime(&row[2]);
                    let description = cell_to_string(&row[7]);
                    let parsed_date = extract_date(&description);
                    ExternalTransaction {
                        date: spend_date.map(|datetime| datetime.date()),
                        booking_date: cell_to_date(&row[3]),
                        amount: cell_to_decimal(&row[4]),
                        category: cell_to_string(&row[1]),
                        description,
                        other_account: cell_to_string(&row[5]),
                        other_account_name: cell_to_string(&row[6]),
                        textual_date: parsed_date,
                        transaction_fee: None,
                    }
                })
                .collect()),
            SheetFormat::Granit => Ok(range
                .rows()
                .filter(|row| row[1].is_float())
                .map(|row| {
                    let date = cell_to_iso_date(&row[4]);
                    let other_account_name = cell_to_string(&row[7])
                        .or_else(|| cell_to_string(&row[9]))
                        .map(cleanup_string);
                    let comment = cell_to_string(&row[11]);
                    ExternalTransaction {
                        date,
                        booking_date: None,
                        amount: cell_to_decimal(&row[1]),
                        category: cell_to_string(&row[6]),
                        description: concat(other_account_name.as_ref(), comment.as_ref()),
                        other_account: cell_to_string(&row[8]),
                        other_account_name,
                        textual_date: None,
                        transaction_fee: None,
                    }
                })
                .collect()),
            SheetFormat::BankAustria => Ok(range
                .rows()
                .skip(1)
                .filter(|row| row[6].is_float())
                .map(|row| {
                    let date = cell_to_german_date(&row[1]);
                    let booking_date = cell_to_german_date(&row[1]);
                    let amount = cell_to_decimal(&row[6]);
                    let other_account = if let Some(amount_value) = amount
                        && amount_value.is_sign_negative()
                    {
                        cell_to_string(&row[12])
                    } else {
                        cell_to_string(&row[9])
                    };
                    ExternalTransaction {
                        date,
                        booking_date,
                        amount,
                        category: None,
                        description: cell_to_string(&row[3]).map(|s| s.trim().to_owned()),
                        other_account,
                        other_account_name: None,
                        textual_date: None,
                        transaction_fee: None,
                    }
                })
                .collect()),
            SheetFormat::Transferwise => Ok(range
                .rows()
                .skip(1)
                .filter(|row| row[2].is_float())
                .map(|row| {
                    let date = cell_to_english_date(&row[1]);
                    let amount = cell_to_decimal(&row[2]);
                    let other_account_name =
                        cell_to_string(&row[13]).or_else(|| cell_to_string(&row[11]));
                    let other_account = cell_to_string(&row[12]);

                    ExternalTransaction {
                        date,
                        booking_date: None,
                        amount,
                        category: None,
                        description: cell_to_string(&row[4]).map(|s| s.trim().to_owned()),
                        other_account,
                        other_account_name,
                        textual_date: None,
                        transaction_fee: cell_to_decimal(&row[14])
                            .filter(Decimal::is_sign_positive),
                    }
                })
                .collect()),
            SheetFormat::Magnet => Ok(range
                .rows()
                .skip(1)
                .filter(|row| row[6].is_float())
                .map(|row| {
                    let date = cell_to_date(&row[1]);
                    let booking_date = cell_to_date(&row[2]);
                    let amount = cell_to_decimal(&row[6]);
                    let other_account = cell_to_string(&row[4]);
                    let other_account_name = cell_to_string(&row[3]);
                    let description = cell_to_string(&row[5]);

                    ExternalTransaction {
                        date,
                        booking_date,
                        amount,
                        category: None,
                        description: concat(other_account_name.as_ref(), description.as_ref()),
                        other_account,
                        other_account_name,
                        textual_date: None,
                        transaction_fee: None,
                    }
                })
                .collect()),
            SheetFormat::Rhai { executor } => executor.process_rhai_script(range),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Executor {
    engine: Engine,
    ast: Option<AST>,
    parser_fn: Option<String>,
}

impl Executor {
    fn new() -> Self {
        let engine = build_rhai_engine();
        Executor {
            engine,
            ast: None,
            parser_fn: None,
        }
    }

    #[cfg(test)]
    fn add_script(&mut self, path: &Path) -> Result<()> {
        let ast = self
            .engine
            .compile_file(path.to_path_buf())
            .map_err(|e| anyhow::anyhow!("Rhai script compilation error: {e}"))?;
        self.add_ast(ast);
        Ok(())
    }

    #[cfg(test)]
    fn add_ast(&mut self, ast: AST) {
        match &self.ast {
            None => {
                self.ast = Some(ast);
            }
            Some(old_ast) => {
                let new_ast = old_ast.merge(&ast);
                self.ast = Some(new_ast);
            }
        }
    }

    fn process_rhai_script(&self, range: &Range<Data>) -> Result<Vec<ExternalTransaction>> {
        let Some(parser_fn) = self.parser_fn.as_deref() else {
            return Err(anyhow::anyhow!("Parser function not set"));
        };
        let Some(ast) = &self.ast else {
            return Err(anyhow::anyhow!("AST not set"));
        };
        let mut transactions = Vec::new();
        let mut scope = Scope::new();
        for row in range.rows() {
            let rhai_row = convert_to_rhai_row(row);

            // Call the Rhai function
            let transaction: ExternalTransaction = self
                .engine
                .call_fn(&mut scope, ast, parser_fn, (rhai_row,))
                .map_err(|e| {
                    let pos = e.position();
                    if let Some(line) = pos.line()
                        && let Some(source_str) = ast.source()
                    {
                        // Get the specific line for better error reporting
                        let error_line = source_str.lines().nth(line.saturating_sub(1));
                        if let Some(line_content) = error_line {
                            println!("Error on line {line}: {line_content}");
                        }
                    }

                    anyhow::anyhow!("Rhai script execution error: {e}")
                })?;
            if !transaction.is_empty() {
                transactions.push(transaction);
            }
        }
        Ok(transactions)
    }
}

#[allow(clippy::cast_precision_loss)]
fn convert_to_rhai_row(row: &[Data]) -> Vec<Dynamic> {
    // Convert each Calamine Data cell to a Rhai Dynamic type
    let rhai_row: Vec<Dynamic> = row
        .iter()
        .map(|cell| match cell {
            Data::Empty | Data::Error(_) => Dynamic::UNIT,
            Data::String(s) => s.clone().into(),
            Data::Float(f) => (*f).into(),
            Data::Int(i) => (*i as f64).into(), // Rhai numbers are f64 by default
            Data::Bool(b) => (*b).into(),
            Data::DateTime(excel_date_time) => excel_date_time
                .as_datetime()
                .map(|date_time| Dynamic::from(date_time.format("%Y.%m.%d.").to_string()))
                .unwrap_or_default(),
            Data::DateTimeIso(date_time) | Data::DurationIso(date_time) => date_time.clone().into(),
        })
        .collect();
    rhai_row
}

fn concat(first: Option<&String>, second: Option<&String>) -> Option<String> {
    match (first, second) {
        (Some(f), Some(snd)) => {
            let mut x = f.clone();
            x.push(' ');
            x.push_str(snd);
            Some(x)
        }
        (Some(f), None) | (None, Some(f)) => Some(f.clone()),
        (_, _) => None,
    }
}

fn cleanup_string(input: String) -> String {
    let casefix = if input.to_uppercase() == input {
        input.to_lowercase()
    } else {
        input
    };
    casefix
        .replace("A'", "Á")
        .replace("I'", "Í")
        .replace("E'", "É")
        .replace("O'", "Ó")
        .replace("U'", "Ú")
        .replace("U:", "Ü")
        .replace("O:", "Ö")
        .replace("a'", "á")
        .replace("i'", "í")
        .replace("e'", "é")
        .replace("o'", "ó")
        .replace("u'", "ú")
        .replace("u:", "ü")
        .replace("o:", "ö")
}

// Helper to convert Dynamic to calamine::Data
fn dynamic_to_data(d: &Dynamic) -> Data {
    if let Ok(s) = d.clone().into_string() {
        Data::String(s)
    } else if let Ok(i) = d.as_int() {
        Data::Int(i)
    } else if let Ok(f) = d.as_float() {
        Data::Float(f)
    } else if let Ok(b) = d.as_bool() {
        Data::Bool(b)
    } else {
        Data::Empty
    }
}

// Helper to get float from Dynamic
#[allow(clippy::cast_precision_loss, clippy::needless_pass_by_value)]
fn get_float(d: Dynamic) -> f64 {
    if let Ok(f) = d.as_float() {
        f
    } else if let Ok(i) = d.as_int() {
        i as f64
    } else {
        0.0 // Default to 0.0 if not a float or int
    }
}

// Helper to convert Dynamic to Option<NaiveDate>
fn dynamic_to_option_naive_date(d: Dynamic) -> Option<NaiveDate> {
    if d.is_unit() {
        None
    } else {
        match d.try_cast_result::<NaiveDate>() {
            Ok(naive_date) => Some(naive_date),
            Err(dynamic) => dynamic
                .try_cast_result::<Option<NaiveDate>>()
                .unwrap_or_default(),
        }
    }
}

// Helper to convert Dynamic to Option<Decimal>
fn dynamic_to_option_decimal(d: Dynamic) -> Option<Decimal> {
    if d.is_unit() {
        None
    } else {
        d.clone()
            .try_cast::<Decimal>()
            .or_else(|| d.try_cast::<Option<Decimal>>().flatten())
    }
}

// Helper to convert Dynamic to Option<String>
fn dynamic_to_option_string(d: Dynamic) -> Option<String> {
    if d.is_unit() {
        None
    } else {
        d.clone()
            .try_cast::<String>()
            .or_else(|| d.try_cast::<Option<String>>().flatten())
    }
}

// Check whether a rhai value represents a present value: a non-unit value
// that is not an empty (None) Option. Rhai keeps Option<T> return values of
// registered functions as actual Option<T> values, unrelated to the unit.
fn has_value(d: Dynamic) -> bool {
    if d.is_unit() {
        return false;
    }
    if let Some(opt) = d.clone().try_cast::<Option<NaiveDate>>() {
        return opt.is_some();
    }
    if let Some(opt) = d.clone().try_cast::<Option<NaiveDateTime>>() {
        return opt.is_some();
    }
    if let Some(opt) = d.clone().try_cast::<Option<String>>() {
        return opt.is_some();
    }
    if let Some(opt) = d.clone().try_cast::<Option<Decimal>>() {
        return opt.is_some();
    }
    if let Some(opt) = d.clone().try_cast::<Option<f64>>() {
        return opt.is_some();
    }
    if let Some(opt) = d.clone().try_cast::<Option<i64>>() {
        return opt.is_some();
    }
    if let Some(opt) = d.try_cast::<Option<bool>>() {
        return opt.is_some();
    }
    true
}

#[allow(clippy::too_many_lines)]
pub fn build_rhai_engine() -> Engine {
    let mut engine = Engine::new();

    // Register Rust functions with Rhai-compatible wrappers
    engine.register_fn("get_float", get_float);
    engine.register_fn("cell_to_date", |d: Dynamic| {
        cell_to_date(&dynamic_to_data(&d))
    });
    engine.register_fn("cell_to_datetime", |d: Dynamic| {
        cell_to_datetime(&dynamic_to_data(&d))
    });
    engine.register_fn("cell_to_decimal", |d: Dynamic| {
        cell_to_decimal(&dynamic_to_data(&d))
    });
    engine.register_fn("cell_to_english_date", |d: Dynamic| {
        cell_to_english_date(&dynamic_to_data(&d))
    });
    engine.register_fn("cell_to_german_date", |d: Dynamic| {
        cell_to_german_date(&dynamic_to_data(&d))
    });
    engine.register_fn("cell_to_iso_date", |d: Dynamic| {
        cell_to_iso_date(&dynamic_to_data(&d))
    });
    engine.register_fn("cell_to_string", |d: Dynamic| {
        cell_to_string(&dynamic_to_data(&d))
    });
    engine.register_fn("debug", |d: Dynamic| {
        println!("debug: {d:?}");
        format!("DEBUG: {d:?}")
    });
    engine.register_fn("is_unit", |value: Dynamic| value.is_unit());
    engine.register_fn("has_value", |d: Dynamic| has_value(d));
    engine.register_fn("extract_date", extract_date);
    engine.register_fn("concat", |first: Option<String>, second: Option<String>| {
        concat(first.as_ref(), second.as_ref())
    });
    engine.register_fn("cleanup_string", cleanup_string);
    engine.register_fn(
        "create_naive_date_ymd",
        |year: i64, month: i64, day: i64| {
            if let (Ok(year), Ok(month), Ok(day)) =
                (year.try_into(), month.try_into(), day.try_into())
            {
                NaiveDate::from_ymd_opt(year, month, day)
            } else {
                None
            }
        },
    );

    // Register Decimal type and constructors manually
    engine.register_type::<Decimal>();
    engine.register_fn("new_decimal_from_f64", |val: f64| Decimal::from_f64(val));
    engine.register_fn("new_decimal_from_string", |val: String| {
        val.parse::<Decimal>()
    });

    // Register ExternalTransaction type and keep old function for backward compatibility
    engine.register_type::<ExternalTransaction>();
    engine.register_fn(
        "new_external_transaction",
        |date_dyn: Dynamic,
         booking_date_dyn: Dynamic,
         amount_dyn: Dynamic,
         category_dyn: Dynamic,
         description_dyn: Dynamic,
         other_account_dyn: Dynamic,
         other_account_name_dyn: Dynamic,
         textual_date_dyn: Dynamic,
         transaction_fee_dyn: Dynamic| {
            println!(
                "new_external_transaction: date: {date_dyn:?} amount: {amount_dyn:?}, description: {description_dyn:?}"
            );
            ExternalTransaction {
                date: dynamic_to_option_naive_date(date_dyn),
                booking_date: dynamic_to_option_naive_date(booking_date_dyn),
                amount: dynamic_to_option_decimal(amount_dyn),
                category: dynamic_to_option_string(category_dyn),
                description: dynamic_to_option_string(description_dyn),
                other_account: dynamic_to_option_string(other_account_dyn),
                other_account_name: dynamic_to_option_string(other_account_name_dyn),
                textual_date: dynamic_to_option_naive_date(textual_date_dyn),
                transaction_fee: dynamic_to_option_decimal(transaction_fee_dyn),
            }
        },
    );

    // Register ExternalTransactionBuilder type and methods for cleaner API
    engine.register_type::<ExternalTransactionBuilder>();
    engine.register_fn("new_transaction", ExternalTransactionBuilder::new);
    engine.register_fn(
        "with_date",
        |builder: ExternalTransactionBuilder, date: Option<NaiveDate>| builder.with_date(date),
    );
    engine.register_fn(
        "with_booking_date",
        |builder: ExternalTransactionBuilder, booking_date: Option<NaiveDate>| {
            builder.with_booking_date(booking_date)
        },
    );
    engine.register_fn(
        "with_amount",
        |builder: ExternalTransactionBuilder, amount: Option<Decimal>| builder.with_amount(amount),
    );
    engine.register_fn(
        "with_category",
        |builder: ExternalTransactionBuilder, category: Option<String>| {
            builder.with_category(category)
        },
    );
    engine.register_fn(
        "with_category",
        |builder: ExternalTransactionBuilder, category: String| {
            builder.with_category(Some(category))
        },
    );
    engine.register_fn(
        "with_description",
        |builder: ExternalTransactionBuilder, description: Option<String>| {
            builder.with_description(description)
        },
    );
    engine.register_fn(
        "with_description",
        |builder: ExternalTransactionBuilder, description: String| {
            builder.with_description(Some(description))
        },
    );
    engine.register_fn(
        "with_other_account",
        |builder: ExternalTransactionBuilder, other_account: Option<String>| {
            builder.with_other_account(other_account)
        },
    );
    engine.register_fn(
        "with_other_account",
        |builder: ExternalTransactionBuilder, other_account: String| {
            builder.with_other_account(Some(other_account))
        },
    );
    engine.register_fn(
        "with_other_account_name",
        |builder: ExternalTransactionBuilder, other_account_name: Option<String>| {
            builder.with_other_account_name(other_account_name)
        },
    );
    engine.register_fn(
        "with_other_account_name",
        |builder: ExternalTransactionBuilder, other_account_name: String| {
            builder.with_other_account_name(Some(other_account_name))
        },
    );
    engine.register_fn(
        "with_textual_date",
        |builder: ExternalTransactionBuilder, textual_date: Option<NaiveDate>| {
            builder.with_textual_date(textual_date)
        },
    );
    engine.register_fn(
        "with_transaction_fee",
        |builder: ExternalTransactionBuilder, transaction_fee: Option<Decimal>| {
            builder.with_transaction_fee(transaction_fee)
        },
    );
    engine.register_fn("create", |builder: ExternalTransactionBuilder| {
        builder.create()
    });

    engine
}

#[cfg(test)]
mod tests {
    #![allow(clippy::arc_with_non_send_sync)]
    use super::*;
    use anyhow::Result;
    use calamine::{CellErrorType, ExcelDateTime, ExcelDateTimeType, Range};

    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;

    #[test]
    fn test_rhai_sheet_parser() -> Result<()> {
        // 1. Create a dummy Rhai script file using the cleaner builder pattern
        let script_content = r"
            fn parse_sheet_row(row) {
                // Extract amount and description from row
                let amount_val = get_float(row[1]);
                let description_str = row[2].to_string();

                // Convert amount to decimal
                let amount = new_decimal_from_f64(amount_val);

                // Create a fixed date for testing
                let date = create_naive_date_ymd(2023, 1, 15);

                // Create transaction using builder pattern (much cleaner!)
                new_transaction()
                    .with_date(date)
                    .with_amount(amount)
                    .with_description(description_str)
                    .create()
            }
        ";

        let script_file_name = "test_rhai_script.rhai";
        let script_path = PathBuf::from(script_file_name);
        let mut file = fs::File::create(&script_path)?;
        file.write_all(script_content.as_bytes())?;

        // Ensure the file is cleaned up after the test
        let _cleanup = scopeguard::guard(script_path.clone(), |path| {
            let _ = fs::remove_file(path);
        });

        // 2. Load this script using SheetFormat::new_rhai
        let mut executor = Executor::new();
        executor.add_script(&script_path)?;
        executor.parser_fn = Some("parse_sheet_row".to_string());
        let rhai_format = SheetFormat::Rhai {
            executor: Arc::new(executor),
        };

        // 3. Create a sample Range<Data> with only one row
        let mut range = Range::new((0, 0), (0, 3));
        range.set_value((0, 0), Data::String("2023.01.15.".to_string()));
        range.set_value((0, 1), Data::Float(123.45));
        range.set_value((0, 2), Data::String("Test Description".to_string()));

        // 4. Call parse_sheet with the Rhai SheetFormat
        let transactions = rhai_format.parse_sheet(&range)?;
        println!("transactions: {transactions:?}");

        // 5. Assert that the returned ExternalTransaction is as expected
        assert_eq!(transactions.len(), 1);
        let transaction = &transactions[0];
        println!("transaction: {transaction:?}");
        assert_eq!(
            transaction.date,
            Some(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap())
        );
        assert_eq!(transaction.amount, Some(Decimal::new(12345, 2)));
        assert_eq!(
            transaction.description,
            Some("Test Description".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_dynamic_to_data() {
        // Test string conversion
        let string_dyn: Dynamic = "test".into();
        assert_eq!(
            dynamic_to_data(&string_dyn),
            Data::String("test".to_string())
        );

        // Test float conversion
        let float_dyn: Dynamic = 3.15.into();
        assert_eq!(dynamic_to_data(&float_dyn), Data::Float(3.15));

        // Test int conversion
        let int_dyn: Dynamic = 42.into();
        assert_eq!(dynamic_to_data(&int_dyn), Data::Int(42));

        // Test bool conversion
        let bool_dyn: Dynamic = true.into();
        assert_eq!(dynamic_to_data(&bool_dyn), Data::Bool(true));

        // Test unit conversion
        let unit_dyn = Dynamic::UNIT;
        assert_eq!(dynamic_to_data(&unit_dyn), Data::Empty);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_get_float() {
        // Test float conversion
        let float_dyn: Dynamic = 3.15.into();
        assert_eq!(get_float(float_dyn), 3.15);

        // Test int conversion
        let int_dyn: Dynamic = 42.into();
        assert_eq!(get_float(int_dyn), 42.0);

        // Test unit conversion
        let unit_dyn = Dynamic::UNIT;
        assert_eq!(get_float(unit_dyn), 0.0);
    }

    #[test]
    fn test_dynamic_to_option_naive_date() {
        // Test valid date conversion
        let date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();
        let date_dyn: Dynamic = Dynamic::from(date);
        assert_eq!(dynamic_to_option_naive_date(date_dyn), Some(date));

        // Test unit conversion
        let unit_dyn = Dynamic::UNIT;
        assert_eq!(dynamic_to_option_naive_date(unit_dyn), None);

        let date = NaiveDate::from_ymd_opt(2023, 1, 15);
        let date_dyn: Dynamic = Dynamic::from(date);
        assert_eq!(dynamic_to_option_naive_date(date_dyn), date);
    }

    #[test]
    fn test_dynamic_to_option_decimal() {
        // Test valid decimal conversion
        let decimal = Decimal::new(12345, 2);
        let decimal_dyn: Dynamic = Dynamic::from(decimal);
        assert_eq!(dynamic_to_option_decimal(decimal_dyn), Some(decimal));

        // Test unit conversion
        let unit_dyn = Dynamic::UNIT;
        assert_eq!(dynamic_to_option_decimal(unit_dyn), None);
    }

    #[test]
    fn test_dynamic_to_option_string() {
        // Test valid string conversion
        let string_dyn: Dynamic = "test".into();
        assert_eq!(
            dynamic_to_option_string(string_dyn),
            Some("test".to_string())
        );

        // Test unit conversion
        let unit_dyn = Dynamic::UNIT;
        assert_eq!(dynamic_to_option_string(unit_dyn), None);
    }

    #[test]
    fn test_better_rhai_style_builder_pattern() -> Result<()> {
        // Test the better.rhai style builder pattern
        let script_content = r"
            fn parse_sheet_row(row) {
                let amount_val = get_float(row[1]);
                let description_str = row[2].to_string();
                let date = create_naive_date_ymd(2023, 1, 15);
                let amount = new_decimal_from_f64(amount_val);

                // Create transaction using builder pattern (like better.rhai)
                new_transaction()
                    .with_date(date)
                    .with_amount(amount)
                    .with_description(description_str)
                    .create()
            }
        ";

        let script_file_name = "test_better_style.rhai";
        let script_path = PathBuf::from(script_file_name);
        let mut file = fs::File::create(&script_path)?;
        file.write_all(script_content.as_bytes())?;

        let _cleanup = scopeguard::guard(script_path.clone(), |path| {
            let _ = fs::remove_file(path);
        });

        let mut executor = Executor::new();
        executor.add_script(&script_path)?;
        executor.parser_fn = Some("parse_sheet_row".to_string());
        let rhai_format = SheetFormat::Rhai {
            executor: Arc::new(executor),
        };

        let mut range = Range::new((0, 0), (0, 3));
        range.set_value((0, 0), Data::String("2023.01.15.".to_string()));
        range.set_value((0, 1), Data::Float(123.45));
        range.set_value((0, 2), Data::String("Test Description".to_string()));

        let transactions = rhai_format.parse_sheet(&range)?;

        assert_eq!(transactions.len(), 1);
        let transaction = &transactions[0];
        assert_eq!(
            transaction.date,
            Some(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap())
        );
        assert_eq!(transaction.amount, Some(Decimal::new(12345, 2)));
        assert_eq!(
            transaction.description,
            Some("Test Description".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_has_value() {
        assert!(!has_value(Dynamic::UNIT));
        assert!(!has_value(Dynamic::from(None::<String>)));
        assert!(has_value(Dynamic::from(Some("x".to_string()))));
        assert!(!has_value(Dynamic::from(None::<Decimal>)));
        assert!(has_value(Dynamic::from(Some(Decimal::new(1, 0)))));
        assert!(!has_value(Dynamic::from(None::<i64>)));
        assert!(has_value(Dynamic::from(Some(42_i64))));
        assert!(has_value(Dynamic::from(3.15)));
        assert!(has_value(Dynamic::from(42_i64)));
        assert!(has_value(Dynamic::from(true)));
        assert!(has_value(Dynamic::from("plain")));
        assert!(!has_value(Dynamic::from(None::<NaiveDate>)));
        assert!(has_value(Dynamic::from(Some(
            NaiveDate::from_ymd_opt(2023, 1, 15).unwrap()
        ))));
    }

    #[test]
    fn test_dynamic_to_option_string_with_option() {
        // Rhai stores the whole Option<T> as the value, verify both conversions
        assert_eq!(
            dynamic_to_option_string(Dynamic::from(Some("hello".to_string()))),
            Some("hello".to_string())
        );
        assert_eq!(
            dynamic_to_option_string(Dynamic::from(None::<String>)),
            None
        );
        assert_eq!(dynamic_to_option_string(Dynamic::UNIT), None);
    }

    #[test]
    fn test_dynamic_to_option_decimal_with_option() {
        assert_eq!(
            dynamic_to_option_decimal(Dynamic::from(Some(Decimal::new(12345, 2)))),
            Some(Decimal::new(12345, 2))
        );
        assert_eq!(
            dynamic_to_option_decimal(Dynamic::from(None::<Decimal>)),
            None
        );
        assert_eq!(dynamic_to_option_decimal(Dynamic::UNIT), None);
    }

    #[test]
    fn test_convert_to_rhai_row_datetime() {
        // Data::DateTime cells are converted to a parseable "yyyy.mm.dd." string
        let row = vec![
            Data::DateTime(ExcelDateTime::new(
                44060.0,
                ExcelDateTimeType::DateTime,
                false,
            )),
            Data::DateTimeIso("2023-01-15".to_string()),
            Data::Error(CellErrorType::NA),
        ];
        let rhai_row = convert_to_rhai_row(&row);
        assert_eq!(rhai_row[0].clone().cast::<String>(), "2020.08.17.");
        assert_eq!(rhai_row[1].clone().cast::<String>(), "2023-01-15");
        assert_eq!(rhai_row[2].type_id(), Dynamic::UNIT.type_id());
        assert!(has_value(rhai_row[0].clone()));
        assert!(!has_value(rhai_row[2].clone()));
    }

    #[test]
    fn test_rhai_script_can_call_concat() -> Result<()> {
        // concat must be callable from a script with owned Option<String> args
        let script_content = r"
            fn parse_sheet_row(row) {
                let part1 = cell_to_string(row[0]);
                let part2 = cell_to_string(row[1]);
                new_transaction().with_description(concat(part1, part2)).create()
            }
        ";

        let script_file_name = "test_concat_script.rhai";
        let script_path = PathBuf::from(script_file_name);
        let mut file = fs::File::create(&script_path)?;
        file.write_all(script_content.as_bytes())?;

        let _cleanup = scopeguard::guard(script_path.clone(), |path| {
            let _ = fs::remove_file(path);
        });

        let mut executor = Executor::new();
        executor.add_script(&script_path)?;
        executor.parser_fn = Some("parse_sheet_row".to_string());
        let rhai_format = SheetFormat::Rhai {
            executor: Arc::new(executor),
        };

        let mut range = Range::new((0, 0), (0, 1));
        range.set_value((0, 0), Data::String("Hello".to_string()));
        range.set_value((0, 1), Data::String("World".to_string()));

        let transactions = rhai_format.parse_sheet(&range)?;

        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].description, Some("Hello World".to_string()));

        Ok(())
    }

    #[test]
    fn test_rhai_parser_skips_empty_transactions() -> Result<()> {
        // A script returning an empty transaction (e.g. for the header row)
        // should have those rows dropped from the parsed result.
        let script_content = r"
            fn parse_sheet_row(row) {
                let date = cell_to_date(row[0]);
                if has_value(date) {
                    new_transaction().with_date(date).create()
                } else {
                    new_transaction().create()
                }
            }
        ";

        let script_file_name = "test_skip_empty.rhai";
        let script_path = PathBuf::from(script_file_name);
        let mut file = fs::File::create(&script_path)?;
        file.write_all(script_content.as_bytes())?;

        let _cleanup = scopeguard::guard(script_path.clone(), |path| {
            let _ = fs::remove_file(path);
        });

        let mut executor = Executor::new();
        executor.add_script(&script_path)?;
        executor.parser_fn = Some("parse_sheet_row".to_string());
        let rhai_format = SheetFormat::Rhai {
            executor: Arc::new(executor),
        };

        let mut range = Range::new((0, 0), (2, 0));
        range.set_value((0, 0), Data::String("Header".to_string()));
        range.set_value((1, 0), Data::String("2023.01.15.".to_string()));
        range.set_value((2, 0), Data::String("2023.02.20.".to_string()));

        let transactions = rhai_format.parse_sheet(&range)?;

        assert_eq!(transactions.len(), 2);
        assert_eq!(
            transactions[0].date,
            Some(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap())
        );
        assert_eq!(
            transactions[1].date,
            Some(NaiveDate::from_ymd_opt(2023, 2, 20).unwrap())
        );

        Ok(())
    }
}
