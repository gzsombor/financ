#![recursion_limit = "128"]
#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate clap;

pub mod commands;
pub mod models;
pub mod schema;
pub mod utils;

use clap::{App, Arg, SubCommand};
use commands::{list_accounts, list_entries};
use utils::establish_connection;

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
                    Arg::with_name("memo")
                        .short("m")
                        .long("memo")
                        .help("Splits with the given memo")
                        .required(false)
                        .takes_value(true),
                ),
        )
        .get_matches();

    if let Some(ls_acc_cmd) = matches.subcommand_matches("list-accounts") {
        let limit = value_t!(ls_acc_cmd, "limit", i64).unwrap_or(10);
        let name = value_t!(ls_acc_cmd, "name", String).ok();
        let parent = value_t!(ls_acc_cmd, "parent_guid", String).ok();
        let account_type = value_t!(ls_acc_cmd, "type", String).ok();

        let connection = establish_connection();
        list_accounts(&connection, limit, name, parent, account_type);
    }
    if let Some(entries_cmd) = matches.subcommand_matches("splits") {
        let limit = value_t!(entries_cmd, "limit", i64).unwrap_or(10);
        let txid = value_t!(entries_cmd, "txid", String).ok();
        let memo = value_t!(entries_cmd, "memo", String).ok();
        let guid = value_t!(entries_cmd, "guid", String).ok();

        let connection = establish_connection();
        list_entries(&connection, limit, txid, guid, memo);
    }
}

fn is_a_number(v: String) -> Result<(), String> {
    match v.parse::<i64>() {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Value '{}' is not a number!", v)),
    }
}
