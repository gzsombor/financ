#![recursion_limit = "128"]
#![allow(proc_macro_derive_resolution_fallback)]
#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate clap;
extern crate calamine;
extern crate chrono;
extern crate console;
extern crate guid_create;
extern crate regex;
#[macro_use]
extern crate lazy_static;

pub mod correlator;
mod dbmodifier;
mod external_models;
pub mod models;
mod query;
pub mod schema;
pub mod utils;

use clap::{App, AppSettings, Arg, ArgMatches, Shell, SubCommand};
use console::Term;
use correlator::CorrelationCommand;
use external_models::Matching;
use query::accounts::{DEFAULT_ACCOUNT_PARAMS, FROM_ACCOUNT_PARAMS, TARGET_ACCOUNT_PARAMS};
use query::currencies::CommoditiesQuery;
use query::transactions::TransactionQuery;
use utils::establish_connection;

fn main() {
    let matches = build_cli().get_matches();

    match matches.subcommand() {
        ("list-accounts", Some(cmd)) => handle_list_accounts(cmd),
        ("transactions", Some(cmd)) => handle_list_entries(cmd),
        ("correlate", Some(cmd)) => handle_correlate(cmd),
        ("commodities", Some(cmd)) => handle_list_currencies(cmd),
        ("completions", Some(cmd)) => handle_completions(cmd),
        (cmd, _) => println!("Unknown command: {}", cmd),
    }
}

fn build_cli() -> App<'static, 'static> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .subcommand(
            DEFAULT_ACCOUNT_PARAMS.add_arguments(
                SubCommand::with_name("list-accounts").arg(
                    Arg::with_name("limit")
                        .short("l")
                        .long("limit")
                        .help("Limit number of accounts")
                        .required(false)
                        .validator(is_a_number)
                        .takes_value(true),
                ),
            ),
        )
        .subcommand(
            DEFAULT_ACCOUNT_PARAMS.add_arguments(
                TARGET_ACCOUNT_PARAMS.add_arguments(
                    SubCommand::with_name("transactions")
                        .arg(
                            Arg::with_name("limit")
                                .short("l")
                                .long("limit")
                                .help("Limit number of splits")
                                .required(false)
                                .validator(is_a_number)
                                .takes_value(true),
                        )
                        .arg(
                            Arg::with_name("txid")
                                .short("x")
                                .long("transaction-id")
                                .help("Splits with the given transaction id ")
                                .required(false)
                                .takes_value(true),
                        )
                        .arg(
                            Arg::with_name("before")
                                .short("b")
                                .long("before")
                                .help("Splits before the given date in yyyy-mm-dd format")
                                .required(false)
                                .takes_value(true),
                        )
                        .arg(
                            Arg::with_name("after")
                                .short("f")
                                .long("after")
                                .help("Splits after the given date in yyyy-mm-dd format")
                                .required(false)
                                .takes_value(true),
                        )
                        .arg(
                            Arg::with_name("memo")
                                .short("e")
                                .long("memo")
                                .help("Splits with the given memo")
                                .required(false)
                                .takes_value(true),
                        )
                        .arg(
                            Arg::with_name("description")
                                .short("d")
                                .long("description")
                                .help("Transaction with the given description")
                                .required(false)
                                .takes_value(true),
                        )
                        .arg(
                            Arg::with_name("move-split")
                                .short("m")
                                .long("move-split")
                                .help("Move the found splits to the target account"),
                        ),
                ),
            ),
        )
        .subcommand(
            DEFAULT_ACCOUNT_PARAMS.add_arguments(
                FROM_ACCOUNT_PARAMS.add_arguments(
                    SubCommand::with_name("correlate")
                        .arg(
                            Arg::with_name("file")
                                .short("f")
                                .long("file")
                                .help("The file which contains a list of transaction to correlate")
                                .required(true)
                                .takes_value(true),
                        )
                        .arg(
                            Arg::with_name("sheet_name")
                                .short("s")
                                .long("sheet-name")
                                .help("The name of the sheet")
                                .required(true)
                                .takes_value(true),
                        )
                        .arg(
                            Arg::with_name("by_booking_date")
                                .short("d")
                                .long("by-booking-date")
                                .help("Match transactions by the booking date")
                                .required(false)
                                .takes_value(false),
                        )
                        .arg(
                            Arg::with_name("verbose")
                                .short("v")
                                .long("verbose")
                                .help("Verbose logging")
                                .required(false)
                                .takes_value(false),
                        ),
                ),
            ),
        )
        .subcommand(
            SubCommand::with_name("commodities")
                .arg(
                    Arg::with_name("limit")
                        .short("l")
                        .long("limit")
                        .help("Limit number of splits")
                        .required(false)
                        .validator(is_a_number)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("commodity-type")
                        .short("ct")
                        .long("commodity-type")
                        .help("List only a given type of commodities")
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("name")
                        .short("n")
                        .long("name")
                        .help("List only commodities with the given name")
                        .required(false)
                        .takes_value(true),
                ),
        )
        .subcommand(SubCommand::with_name("completions"))
        .setting(AppSettings::ArgRequiredElseHelp)
}

fn handle_list_accounts(ls_acc_cmd: &ArgMatches) {
    let connection = establish_connection();
    let q = DEFAULT_ACCOUNT_PARAMS.build(ls_acc_cmd, Some("limit"));
    q.execute_and_display(&connection);
}

fn handle_list_entries(cmd: &ArgMatches) {
    let connection = establish_connection();
    let account_query = DEFAULT_ACCOUNT_PARAMS.build(&cmd, None);
    let move_splits = cmd.is_present("move-split");
    let move_target_account = if move_splits {
        let target_account_query = TARGET_ACCOUNT_PARAMS.build(&cmd, None);
        let target_account = target_account_query.get_one(&connection, false);
        if target_account.is_none() {
            println!("Unable to determine the target account for the move-split command!");
            return;
        }
        target_account
    } else {
        None
    };
    let q = if let Some(account) = account_query.get_one(&connection, false) {
        println!("Listing transactions in {}", &account.name);
        TransactionQuery::from(cmd).with_account_id(account.guid)
    } else {
        println!("Listing transactions");
        TransactionQuery::from(cmd)
    };
    q.execute_and_process(&connection, &move_target_account);
}

fn handle_list_currencies(cmd: &ArgMatches) {
    let connection = establish_connection();
    let q = CommoditiesQuery::from(cmd);
    q.execute_and_display(&connection);
}

fn handle_correlate(cmd: &ArgMatches) {
    let input_file = value_t!(cmd, "file", String).unwrap();
    let sheet_name = value_t!(cmd, "sheet_name", String).unwrap();
    let account_query = DEFAULT_ACCOUNT_PARAMS.build(&cmd, None);
    let counterparty_account_query = FROM_ACCOUNT_PARAMS.build(&cmd, None);
    let verbose = cmd.is_present("verbose");

    let connection = establish_connection();
    let matching = if cmd.is_present("by_booking_date") {
        Matching::ByBooking
    } else {
        Matching::BySpending
    };

    let term = Term::stdout();
    let cmd = CorrelationCommand {
        input_file,
        sheet_name,
        matching,
        verbose,
        account_query,
        counterparty_account_query,
    };
    cmd.execute(&connection, &term).unwrap();
}

fn handle_completions(cmd: &ArgMatches) {
    build_cli().gen_completions("financ", Shell::Fish, ".");
}

fn is_a_number(v: String) -> Result<(), String> {
    match v.parse::<i64>() {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Value '{}' is not a number!", v)),
    }
}
