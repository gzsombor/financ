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
use commands::list_accounts;
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
                ),
        )
        .get_matches();

    if let Some(ls_acc_cmd) = matches.subcommand_matches("list-accounts") {
        let limit = value_t!(ls_acc_cmd, "limit", i64).unwrap_or(10);
        let name = value_t!(ls_acc_cmd, "name", String).ok();
        let parent = value_t!(ls_acc_cmd, "parent_guid", String).ok();

        let connection = establish_connection();
        list_accounts(&connection, limit, name, parent);
    }
}

fn is_a_number(v: String) -> Result<(), String> {
    match v.parse::<i64>() {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Value '{}' is not a number!", v)),
    }
}
