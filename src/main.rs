#![recursion_limit = "128"]
#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate clap;
extern crate chrono;

pub mod commands;
pub mod models;
pub mod schema;
pub mod utils;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use commands::{list_accounts, list_entries};
use utils::{establish_connection, to_date};

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .subcommand(
            SubCommand::with_name("list-accounts")
                .arg(
                    Arg::with_name("limit")
                        .short("l")
                        .long("limit")
                        .help("Limit number of accounts")
                        .required(false)
                        .validator(is_a_number)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("name")
                        .short("n")
                        .long("name")
                        .help("Limit to accounts which name contains the specified string")
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("parent_guid")
                        .short("p")
                        .long("parent")
                        .help("Limit to the childs accounts")
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("type")
                        .short("t")
                        .long("type")
                        .help("Limit to specified account types")
                        .required(false)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("splits")
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
                    Arg::with_name("guid")
                        .short("g")
                        .long("guid")
                        .help("Splits with the given account id")
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("txid")
                        .short("t")
                        .long("txid")
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
                        .short("a")
                        .long("after")
                        .help("Splits after the given date in yyyy-mm-dd format")
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("memo")
                        .short("m")
                        .long("memo")
                        .help("Splits with the given memo")
                        .required(false)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("correlate").arg(
                Arg::with_name("file")
                    .short("f")
                    .long("file")
                    .help("The file which contains a list of transaction to correlate")
                    .required(true)
                    .takes_value(true),
            ),
        )
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();

    match matches.subcommand() {
        ("list-accounts", Some(ls_acc_cmd)) => handle_list_accounts(ls_acc_cmd),
        ("splits", Some(cmd)) => handle_list_entries(cmd),
        _ => (),
    }
}

fn handle_list_accounts(ls_acc_cmd: &ArgMatches) {
    let limit = value_t!(ls_acc_cmd, "limit", i64).unwrap_or(10);
    let name = value_t!(ls_acc_cmd, "name", String).ok();
    let parent = value_t!(ls_acc_cmd, "parent_guid", String).ok();
    let account_type = value_t!(ls_acc_cmd, "type", String).ok();

    let connection = establish_connection();
    list_accounts(&connection, limit, name, parent, account_type);
}

fn handle_list_entries(entries_cmd: &ArgMatches) {
    let limit = value_t!(entries_cmd, "limit", i64).unwrap_or(10);
    let txid = value_t!(entries_cmd, "txid", String).ok();
    let memo = value_t!(entries_cmd, "memo", String).ok();
    let guid = value_t!(entries_cmd, "guid", String).ok();
    let before = to_date(value_t!(entries_cmd, "before", String).ok());
    let after = to_date(value_t!(entries_cmd, "after", String).ok());

    let connection = establish_connection();
    list_entries(&connection, limit, txid, guid, memo, before, after);
}

fn is_a_number(v: String) -> Result<(), String> {
    match v.parse::<i64>() {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Value '{}' is not a number!", v)),
    }
}
