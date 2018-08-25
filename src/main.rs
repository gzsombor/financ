#![recursion_limit = "128"]
#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate clap;
extern crate calamine;
extern crate chrono;

pub mod correlator;
pub mod models;
mod query;
pub mod schema;
pub mod utils;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use correlator::correlate;
use query::accounts::AccountQuery;
use query::transactions::TransactionQuery;
use utils::establish_connection;

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .subcommand(AccountQuery::add_arguments(
            SubCommand::with_name("list-accounts").arg(
                Arg::with_name("limit")
                    .short("l")
                    .long("limit")
                    .help("Limit number of accounts")
                    .required(false)
                    .validator(is_a_number)
                    .takes_value(true),
            ),
        )).subcommand(
            SubCommand::with_name("splits")
                .arg(
                    Arg::with_name("limit")
                        .short("l")
                        .long("limit")
                        .help("Limit number of splits")
                        .required(false)
                        .validator(is_a_number)
                        .takes_value(true),
                ).arg(
                    Arg::with_name("account")
                        .short("a")
                        .long("account")
                        .help("Splits with the given account id")
                        .required(false)
                        .takes_value(true),
                ).arg(
                    Arg::with_name("txid")
                        .short("t")
                        .long("txid")
                        .help("Splits with the given transaction id ")
                        .required(false)
                        .takes_value(true),
                ).arg(
                    Arg::with_name("before")
                        .short("b")
                        .long("before")
                        .help("Splits before the given date in yyyy-mm-dd format")
                        .required(false)
                        .takes_value(true),
                ).arg(
                    Arg::with_name("after")
                        .short("f")
                        .long("after")
                        .help("Splits after the given date in yyyy-mm-dd format")
                        .required(false)
                        .takes_value(true),
                ).arg(
                    Arg::with_name("memo")
                        .short("m")
                        .long("memo")
                        .help("Splits with the given memo")
                        .required(false)
                        .takes_value(true),
                ).arg(
                    Arg::with_name("description")
                        .short("d")
                        .long("description")
                        .help("Transaction with the given description")
                        .required(false)
                        .takes_value(true),
                ),
        ).subcommand(AccountQuery::add_arguments(
            SubCommand::with_name("correlate")
                .arg(
                    Arg::with_name("file")
                        .short("f")
                        .long("file")
                        .help("The file which contains a list of transaction to correlate")
                        .required(true)
                        .takes_value(true),
                ).arg(
                    Arg::with_name("sheet_name")
                        .short("s")
                        .long("sheet-name")
                        .help("The name of the sheet")
                        .required(true)
                        .takes_value(true),
                ),
        )).setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();

    match matches.subcommand() {
        ("list-accounts", Some(cmd)) => handle_list_accounts(cmd),
        ("splits", Some(cmd)) => handle_list_entries(cmd),
        ("correlate", Some(cmd)) => handle_correlate(cmd),
        _ => (),
    }
}

fn handle_list_accounts(ls_acc_cmd: &ArgMatches) {
    let connection = establish_connection();
    let q = AccountQuery::from(ls_acc_cmd);
    q.execute_and_display(&connection);
}

fn handle_list_entries(entries_cmd: &ArgMatches) {
    let connection = establish_connection();
    let q = TransactionQuery::from(entries_cmd);
    q.execute_and_display(&connection);
}

fn handle_correlate(cmd: &ArgMatches) {
    let input_file = value_t!(cmd, "file", String).unwrap();
    let sheet_name = value_t!(cmd, "sheet_name", String).unwrap();
    let account = value_t!(cmd, "account", String).unwrap();

    let connection = establish_connection();
    correlate(&connection, input_file, sheet_name, account);
}

fn is_a_number(v: String) -> Result<(), String> {
    match v.parse::<i64>() {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Value '{}' is not a number!", v)),
    }
}
