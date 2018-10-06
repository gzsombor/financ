#![recursion_limit = "128"]
#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate clap;
extern crate calamine;
extern crate chrono;
extern crate regex;
#[macro_use]
extern crate lazy_static;

pub mod correlator;
pub mod models;
mod query;
pub mod schema;
pub mod utils;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use correlator::{correlate, Matching};
use query::accounts::AccountQuery;
use query::currencies::CommoditiesQuery;
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
        )).subcommand(AccountQuery::add_arguments(
            SubCommand::with_name("transactions")
                .arg(
                    Arg::with_name("limit")
                        .short("l")
                        .long("limit")
                        .help("Limit number of splits")
                        .required(false)
                        .validator(is_a_number)
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
        )).subcommand(AccountQuery::add_arguments(
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
                ).arg(
                    Arg::with_name("by_booking_date")
                        .short("d")
                        .long("by-booking-date")
                        .help("Match transactions by the booking date")
                        .required(false)
                        .takes_value(false),
                ),
        )).subcommand(SubCommand::with_name("commodities")
                .arg(
                    Arg::with_name("limit")
                        .short("l")
                        .long("limit")
                        .help("Limit number of splits")
                        .required(false)
                        .validator(is_a_number)
                        .takes_value(true),
                ).arg(
                    Arg::with_name("type")
                        .short("t")
                        .long("type")
                        .help("List only a given type of commodities")
                        .required(false)
                        .takes_value(true),
                ).arg(
                    Arg::with_name("name")
                        .short("n")
                        .long("name")
                        .help("List only commodities with the given name")
                        .required(false)
                        .takes_value(true),
                )
        ).setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();

    match matches.subcommand() {
        ("list-accounts", Some(cmd)) => handle_list_accounts(cmd),
        ("transactions", Some(cmd)) => handle_list_entries(cmd),
        ("correlate", Some(cmd)) => handle_correlate(cmd),
        ("commodities", Some(cmd)) => handle_list_currencies(cmd),
        (cmd,_)  => println!("Unknown command: {}", cmd),
    }
}

fn handle_list_accounts(ls_acc_cmd: &ArgMatches) {
    let connection = establish_connection();
    let q = AccountQuery::from(ls_acc_cmd);
    q.execute_and_display(&connection);
}

fn handle_list_entries(cmd: &ArgMatches) {
    let connection = establish_connection();
    let account_query = AccountQuery::from(cmd);
    if let Some(account) = account_query.get_one(&connection) {
        println!("Listing transactions in {}", &account.name);
        let q = TransactionQuery::from(cmd).with_account_id(account.guid);
        q.execute_and_display(&connection);
    }
}

fn handle_list_currencies(cmd: &ArgMatches) {
    let connection = establish_connection();
    let q = CommoditiesQuery::from(cmd);
    q.execute_and_display(&connection);
}

fn handle_correlate(cmd: &ArgMatches) {
    let input_file = value_t!(cmd, "file", String).unwrap();
    let sheet_name = value_t!(cmd, "sheet_name", String).unwrap();
    let account = AccountQuery::from(cmd);

    let connection = establish_connection();
    let matching = match cmd.is_present("by_booking_date") {
        true => Matching::ByBooking,
        _ => Matching::BySpending,
    };

    correlate(&connection, input_file, sheet_name, matching, account);
}

fn is_a_number(v: String) -> Result<(), String> {
    match v.parse::<i64>() {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Value '{}' is not a number!", v)),
    }
}
