use std::path::PathBuf;
use std::sync::Arc;

use crate::external_models::{ExternalTransaction, SheetParser};
use crate::sheets::{
    cell_to_date, cell_to_datetime, cell_to_decimal, cell_to_english_date, cell_to_german_date,
    cell_to_iso_date, cell_to_string,
};
use crate::utils::extract_date;
use anyhow::Result;
use calamine::{Data, DataType, Range};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
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
                        description: concat(&other_account_name, &comment),
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
                            .filter(|value| value.is_sign_positive()),
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
                        description: concat(&other_account_name, &description),
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
struct Executor {
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

    fn add_script(&mut self, path: &PathBuf) -> Result<()> {
        let ast = self
            .engine
            .compile_file(path.to_path_buf())
            .map_err(|e| anyhow::anyhow!("Rhai script compilation error: {}", e))?;
        self.add_ast(ast);
        Ok(())
    }

    fn add_source(&mut self, script: &str) -> Result<()> {
        let ast = self
            .engine
            .compile(script)
            .map_err(|e| anyhow::anyhow!("Rhai script compilation error: {}", e))?;
        self.add_ast(ast);
        Ok(())
    }

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
                .map_err(|e| anyhow::anyhow!("Rhai script execution error: {}", e))?;
            transactions.push(transaction);
        }
        Ok(transactions)
    }
}

fn convert_to_rhai_row(row: &[Data]) -> Vec<rhai::Dynamic> {
    // Convert each Calamine Data cell to a Rhai Dynamic type
    let rhai_row: Vec<rhai::Dynamic> = row
        .iter()
        .map(|cell| match cell {
            Data::Empty => rhai::Dynamic::UNIT,
            Data::String(s) => s.clone().into(),
            Data::Float(f) => (*f).into(),
            Data::Int(i) => (*i as f64).into(), // Rhai numbers are f64 by default
            Data::Bool(b) => (*b).into(),
            // Data::DateTime(excel_date_time) => {
            //     let (year,month,day,hour,minute,second, milli) = excel_date_time.to_ymd_hms_milli();
            //     let naive_date = NaiveDate::from_ymd_opt(year.into(), month.into(), day.into()).expect("Date exists");
            //     let time = NaiveTime::from_hms_milli_opt(hour.into(), minute.into(), second.into(), milli.into()).expect("Time exists");
            //     let naive_datetime = NaiveDateTime::new(naive_date, time);
            //     Dynamic::from(naive_datetime)
            // },
            _ => Dynamic::from(cell.clone())
        })
        .collect();
    rhai_row
}

fn concat(first: &Option<String>, second: &Option<String>) -> Option<String> {
    match (first, second) {
        (Some(f), Some(snd)) => {
            let mut x = f.clone();
            x.push(' ');
            x.push_str(snd);
            Some(x)
        }
        (Some(f), None) => Some(f.clone()),
        (None, Some(f)) => Some(f.clone()),
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

// Helper to convert rhai::Dynamic to calamine::Data
fn dynamic_to_data(d: rhai::Dynamic) -> Data {
    if d.is_string() {
        Data::String(d.into_string().unwrap())
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

// Helper to get float from rhai::Dynamic
fn get_float(d: rhai::Dynamic) -> f64 {
    if let Ok(f) = d.as_float() {
        f
    } else if let Ok(i) = d.as_int() {
        i as f64
    } else {
        0.0 // Default to 0.0 if not a float or int
    }
}

// Helper to convert rhai::Dynamic to Option<chrono::NaiveDate>
fn dynamic_to_option_naive_date(d: rhai::Dynamic) -> Option<chrono::NaiveDate> {
    if d.is_unit() {
        None
    } else {
        match d.try_cast_result::<chrono::NaiveDate>() {
            Ok(naive_date) => Some(naive_date),
            Err(dynamic) => match dynamic.try_cast_result::<Option<chrono::NaiveDate>>() {
                Ok(optional_date) => optional_date,
                Err(_) => None,
            },
        }
    }
}

// Helper to convert rhai::Dynamic to Option<rust_decimal::Decimal>
fn dynamic_to_option_decimal(d: rhai::Dynamic) -> Option<rust_decimal::Decimal> {
    if d.is_unit() {
        None
    } else {
        d.try_cast::<rust_decimal::Decimal>()
    }
}

// Helper to convert rhai::Dynamic to Option<String>
fn dynamic_to_option_string(d: rhai::Dynamic) -> Option<String> {
    if d.is_unit() {
        None
    } else {
        d.try_cast::<String>()
    }
}

pub fn build_rhai_engine() -> Engine {
    let mut engine = Engine::new();

    // Register Rust functions with Rhai-compatible wrappers
    engine.register_fn("get_float", get_float);
    engine.register_fn("cell_to_date", |d: rhai::Dynamic| {
        cell_to_date(&dynamic_to_data(d))
    });
    engine.register_fn("cell_to_datetime", |d: rhai::Dynamic| {
        cell_to_datetime(&dynamic_to_data(d))
    });
    engine.register_fn("cell_to_decimal", |d: rhai::Dynamic| {
        cell_to_decimal(&dynamic_to_data(d))
    });
    engine.register_fn("cell_to_english_date", |d: rhai::Dynamic| {
        cell_to_english_date(&dynamic_to_data(d))
    });
    engine.register_fn("cell_to_german_date", |d: rhai::Dynamic| {
        cell_to_german_date(&dynamic_to_data(d))
    });
    engine.register_fn("cell_to_iso_date", |d: rhai::Dynamic| {
        cell_to_iso_date(&dynamic_to_data(d))
    });
    engine.register_fn("cell_to_string", |d: rhai::Dynamic| {
        cell_to_string(&dynamic_to_data(d))
    });
    engine.register_fn("debug", |d: rhai::Dynamic| {
        println!("debug: {:?}", d);
        format!("DEBUG: {:?}", d)
    });
    engine.register_fn("extract_date", extract_date);
    engine.register_fn("concat", concat);
    engine.register_fn("cleanup_string", cleanup_string);
    engine.register_fn(
        "create_naive_date_ymd",
        |year: i64, month: i64, day: i64| {
            if let (Ok(year), Ok(month), Ok(day)) =
                (year.try_into(), month.try_into(), day.try_into())
            {
                Ok(chrono::NaiveDate::from_ymd_opt(year, month, day))
            } else {
                Err(())
            }
        },
    );

    // Register Decimal type and constructors manually
    engine.register_type::<Decimal>();
    engine.register_fn("new_decimal_from_f64", |val: f64| {
        Decimal::from_f64(val)
    });
    engine.register_fn("new_decimal_from_string", |val: String| {
        val.parse::<Decimal>()
    });

    // Register ExternalTransaction type
    engine.register_type::<ExternalTransaction>();
    engine.register_fn(
        "new_transaction",
        |date_dyn: rhai::Dynamic,
         booking_date_dyn: rhai::Dynamic,
         amount_dyn: rhai::Dynamic,
         category_dyn: rhai::Dynamic,
         description_dyn: rhai::Dynamic,
         other_account_dyn: rhai::Dynamic,
         other_account_name_dyn: rhai::Dynamic,
         textual_date_dyn: rhai::Dynamic,
         transaction_fee_dyn: rhai::Dynamic| {
            println!(
                "new_transaction: date: {:?} amount: {:?}, description: {:?}",
                date_dyn, amount_dyn, description_dyn
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

    engine
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use calamine::Range;
    use chrono::NaiveDate;
    use rhai::Dynamic;
    use rust_decimal::Decimal;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;


    #[test]
    fn test_rhai_sheet_parser() -> Result<()> {
        // 1. Create a dummy Rhai script file manually
        let script_content = r#"
            fn parse_sheet_row(row) {
                let date = cell_to_date(row[0]);
                let x = debug(date);
                let amount_val = get_float(row[1]); // Use get_float() directly
                let description_str = row[2].to_string();

                let amount = new_decimal_from_f64(amount_val); // Use new_decimal_from_f64
                if date {
                    new_transaction(date, (), amount, (), description_str, (), (), (), ())
                }
            }
        "#;

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

        // 3. Create a sample Range<Data>
        let mut range = Range::new((0, 0), (1, 3));
        range.set_value((0, 0), Data::String("2023.01.15.".to_string()));
        range.set_value((0, 1), Data::Float(123.45));
        range.set_value((0, 2), Data::String("Test Description".to_string()));

        // 4. Call parse_sheet with the Rhai SheetFormat
        let transactions = rhai_format.parse_sheet(&range)?;
        println!("transactions: {:?}", transactions);

        // 5. Assert that the returned ExternalTransaction is as expected
        assert_eq!(transactions.len(), 1);
        let transaction = &transactions[0];
        println!("transaction: {:?}", transaction);
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
            dynamic_to_data(string_dyn),
            Data::String("test".to_string())
        );

        // Test float conversion
        let float_dyn: Dynamic = 3.14.into();
        assert_eq!(dynamic_to_data(float_dyn), Data::Float(3.14));

        // Test int conversion
        let int_dyn: Dynamic = 42.into();
        assert_eq!(dynamic_to_data(int_dyn), Data::Int(42));

        // Test bool conversion
        let bool_dyn: Dynamic = true.into();
        assert_eq!(dynamic_to_data(bool_dyn), Data::Bool(true));

        // Test unit conversion
        let unit_dyn = Dynamic::UNIT;
        assert_eq!(dynamic_to_data(unit_dyn), Data::Empty);
    }

    #[test]
    fn test_get_float() {
        // Test float conversion
        let float_dyn: Dynamic = 3.14.into();
        assert_eq!(get_float(float_dyn), 3.14);

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
}
