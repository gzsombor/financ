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
extern crate anyhow;
#[macro_use]
extern crate lazy_static;

mod cli;
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

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use cli::{Commands, CommoditiesArgs, CorrelateArgs, ListAccountsArgs, TransactionsArgs};
use console::{style, Term};

use crate::cli::Cli;
use crate::correlator::CorrelationCommand;
use crate::external_models::Matching;
use crate::formats::create_format;
use crate::query::accounts::ToAccountQuery;
use crate::query::currencies::CommoditiesQuery;
use crate::query::transactions::TransactionQuery;
use crate::utils::establish_connection;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::ListAccounts(args) => handle_list_accounts(args),
        Commands::Transactions(args) => handle_list_entries(args),
        Commands::Commodities(args) => handle_commodities(args),
        Commands::Correlate(args) => handle_correlate(args),
        Commands::Completions { shell } => handle_shell_completions(shell),
    }
    .unwrap();
}

fn handle_shell_completions(shell: Shell) -> Result<usize> {
    let mut cmd = Cli::command();
    eprintln!("Generating completion file for {shell:?}...");
    generate(shell, &mut cmd, "financ", &mut io::stdout());
    Ok(0)
}

fn handle_list_accounts(args: ListAccountsArgs) -> Result<usize> {
    let mut connection = establish_connection();
    let q = args.account.build(args.limit);
    q.execute_and_display(&mut connection);
    Ok(0)
}

fn handle_list_entries(args: TransactionsArgs) -> Result<usize> {
    let term = Term::stdout();

    let mut connection = establish_connection();
    let account_query = args.account.build(None);
    let move_target_account = if args.move_split {
        let target_account_query = args.target_account.build(None);
        let target_account = target_account_query.get_one(&mut connection, false);
        if target_account.is_none() {
            term.write_line(&format!(
                "Unable to determine the target account for the move-split command:{:?}",
                style(&target_account_query).red()
            ))?;
            return Err(anyhow!(
                "Target account missing, command: {}!",
                &target_account_query
            ));
        }
        target_account
    } else {
        None
    };
    let q = if let Some(account) = account_query.get_one(&mut connection, false) {
        if let Some(target_account) = &move_target_account {
            if target_account.commodity_guid != account.commodity_guid {
                term.write_line(&format!(
                    "The two account has different commodities, unable to transfer between: {} - {}",
                    style(&account).red(),
                    style(target_account).red()
                ))?;
                return Err(anyhow!(
                    "Different commodities: from account={} target account={}!",
                    &account,
                    target_account
                ));
            }
        }

        term.write_line(&format!(
            "Listing transactions in {}",
            style(account.name).blue()
        ))?;
        TransactionQuery::from(args).with_account_id(account.guid)
    } else {
        term.write_line("Listing transactions")?;
        TransactionQuery::from(args)
    };
    // term.write_line(&format!("Limit is {}", style(q.limit).red()))?;
    q.execute_and_process(&mut connection, &move_target_account, &term)
}

fn handle_commodities(cmd: CommoditiesArgs) -> Result<usize> {
    let mut connection = establish_connection();
    let q = CommoditiesQuery::from(cmd);
    q.execute_and_display(&mut connection)
}

fn handle_correlate(cmd: CorrelateArgs) -> Result<usize> {
    let format = cmd.format;

    let mut connection = establish_connection();
    let matching = if cmd.by_booking_date {
        Matching::ByBooking
    } else {
        Matching::BySpending
    };

    let term = Term::stdout();
    let mut cmd = CorrelationCommand {
        input_file: cmd.input,
        sheet_name: cmd.sheet_name,
        matching,
        verbose: cmd.verbose,
        list_extra_transactions: cmd.list_extra_transactions,
        account_query: cmd.account.build(None),
        counterparty_account_query: cmd.from_account.build(None),
        fee_account_query: cmd.fee_account.build(None),
    };
    let format = create_format(&format)
        .with_context(|| format!("Unknown format:'{}'!", format.unwrap_or_default()))?;
    cmd.execute(&mut connection, &term, &format)
}
