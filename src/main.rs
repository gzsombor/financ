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
mod formats;
pub mod models;
mod query;
pub mod schema;
mod sheets;
pub mod utils;

use std::io;

use clap::{App, AppSettings, Arg, ArgMatches, Shell, SubCommand};
use console::{style, Term};

use crate::correlator::CorrelationCommand;
use crate::external_models::Matching;
use crate::formats::create_format;
use crate::query::accounts::{DEFAULT_ACCOUNT_PARAMS, FROM_ACCOUNT_PARAMS, TARGET_ACCOUNT_PARAMS};
use crate::query::currencies::CommoditiesQuery;
use crate::query::transactions::TransactionQuery;
use crate::utils::establish_connection;

fn main() {
    let matches = build_cli().get_matches();

    match matches.subcommand() {
        ("list-accounts", Some(cmd)) => handle_list_accounts(cmd),
        ("transactions", Some(cmd)) => handle_list_entries(cmd),
        ("correlate", Some(cmd)) => handle_correlate(cmd),
        ("commodities", Some(cmd)) => handle_list_currencies(cmd),
        ("completions", Some(cmd)) => handle_completions(cmd),
        (cmd, _) => {
            println!("Unknown command: {}", cmd);
            Ok(0)
        }
    }
    .unwrap();
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
                            Arg::with_name("input")
                                .short("i")
                                .long("input")
                                .help("The file which contains a list of transaction to correlate")
                                .required(true)
                                .takes_value(true),
                        )
                        .arg(
                            Arg::with_name("sheet_name")
                                .short("s")
                                .long("sheet-name")
                                .help("The name of the sheet")
                                .required(false)
                                .takes_value(true),
                        )
                        .arg(
                            Arg::with_name("format")
                                .short("f")
                                .long("format")
                                .help("The format of the sheet")
                                .required(false)
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
                            Arg::with_name("list-extra-transactions")
                                .short("X")
                                .long("list-extra-transactions")
                                .help("List extra transactions not found in the external source")
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

fn handle_list_accounts(ls_acc_cmd: &ArgMatches) -> io::Result<usize> {
    let connection = establish_connection();
    let q = DEFAULT_ACCOUNT_PARAMS.build(ls_acc_cmd, Some("limit"));
    q.execute_and_display(&connection);
    Ok(0)
}

fn handle_list_entries(cmd: &ArgMatches) -> io::Result<usize> {
    let term = Term::stdout();

    let connection = establish_connection();
    let account_query = DEFAULT_ACCOUNT_PARAMS.build(&cmd, None);
    let move_splits = cmd.is_present("move-split");
    let move_target_account = if move_splits {
        let target_account_query = TARGET_ACCOUNT_PARAMS.build(&cmd, None);
        let target_account = target_account_query.get_one(&connection, false);
        if target_account.is_none() {
            term.write_line(&format!(
                "Unable to determine the target account for the move-split command:{:?}",
                style(target_account_query).red()
            ))?;
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Target account missing",
            ));
        }
        target_account
    } else {
        None
    };
    let q = if let Some(account) = account_query.get_one(&connection, false) {
        if let Some(target_account) = &move_target_account {
            if target_account.commodity_guid != account.commodity_guid {
                term.write_line(&format!(
                    "The two account has different commodities, unable to transfer between: {} - {}",
                    style(account).red(),
                    style(target_account).red()
                ))?;
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Different commodities!",
                ));
            }
        }

        term.write_line(&format!(
            "Listing transactions in {}",
            style(account.name).blue()
        ))?;
        TransactionQuery::from(cmd).with_account_id(account.guid)
    } else {
        term.write_line("Listing transactions")?;
        TransactionQuery::from(cmd)
    };
    // term.write_line(&format!("Limit is {}", style(q.limit).red()))?;
    return q.execute_and_process(&connection, &move_target_account, &term);
}

fn handle_list_currencies(cmd: &ArgMatches) -> io::Result<usize> {
    let connection = establish_connection();
    let q = CommoditiesQuery::from(cmd);
    q.execute_and_display(&connection)
}

fn handle_correlate(cmd: &ArgMatches) -> io::Result<usize> {
    let input_file = value_t!(cmd, "input", String).unwrap();
    let sheet_name = value_t!(cmd, "sheet_name", String).ok();
    let account_query = DEFAULT_ACCOUNT_PARAMS.build(&cmd, None);
    let counterparty_account_query = FROM_ACCOUNT_PARAMS.build(&cmd, None);
    let verbose = cmd.is_present("verbose");
    let format = value_t!(cmd, "format", String).ok();

    let connection = establish_connection();
    let matching = if cmd.is_present("by_booking_date") {
        Matching::ByBooking
    } else {
        Matching::BySpending
    };
    let list_extra_transactions = cmd.is_present("list-extra-transactions");

    let term = Term::stdout();
    let cmd = CorrelationCommand {
        input_file,
        sheet_name,
        matching,
        verbose,
        list_extra_transactions,
        account_query,
        counterparty_account_query,
    };
    let format = create_format(format).expect("Unknown format");
    cmd.execute(&connection, &term, &format)
}

fn handle_completions(_cmd: &ArgMatches) -> io::Result<usize> {
    build_cli().gen_completions("financ", Shell::Fish, ".");
    Ok(0)
}

fn is_a_number(v: String) -> Result<(), String> {
    match v.parse::<i64>() {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Value '{}' is not a number!", v)),
    }
}
