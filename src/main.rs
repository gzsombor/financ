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

use std::fs;
use std::io;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};
use cli::{
    Commands, CommoditiesArgs, CorrelateArgs, EvalScriptArgs, ListAccountsArgs, TransactionsArgs,
};
use console::{Term, style};

use crate::cli::Cli;
use crate::correlator::CorrelationCommand;
use crate::external_models::{Matching, SheetDefinition};
use crate::formats::{SheetFormat, build_rhai_engine};
use crate::query::accounts::ToAccountQuery;
use crate::query::currencies::CommoditiesQuery;
use crate::query::transactions::TransactionQuery;
use crate::utils::establish_connection;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::ListAccounts(args) => handle_list_accounts(&args),
        Commands::Transactions(args) => handle_list_entries(args),
        Commands::Commodities(args) => Ok(handle_commodities(args)),
        Commands::Correlate(args) => handle_correlate(args),
        Commands::EvalScript(args) => handle_eval_script(args),
        Commands::Completions { shell } => Ok(handle_shell_completions(shell)),
    }
    .unwrap();
}

fn handle_shell_completions(shell: Shell) -> usize {
    let mut cmd = Cli::command();
    eprintln!("Generating completion file for {shell:?}...");
    generate(shell, &mut cmd, "financ", &mut io::stdout());
    0
}

fn handle_list_accounts(args: &ListAccountsArgs) -> Result<usize> {
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
                "Target account missing, command: {target_account_query}!"
            ));
        }
        target_account
    } else {
        None
    };
    let q = if let Some(account) = account_query.get_one(&mut connection, false) {
        if let Some(target_account) = &move_target_account
            && target_account.commodity_guid != account.commodity_guid
        {
            term.write_line(&format!(
                "The two account has different commodities, unable to transfer between: {} - {}",
                style(&account).red(),
                style(target_account).red()
            ))?;
            return Err(anyhow!(
                "Different commodities: from account={account} target account={target_account}!"
            ));
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
    q.execute_and_process(&mut connection, move_target_account.as_ref(), &term)
}

fn handle_commodities(cmd: CommoditiesArgs) -> usize {
    let mut connection = establish_connection();
    let q = CommoditiesQuery::from(cmd);
    q.execute_and_display(&mut connection);
    0
}

fn handle_correlate(cmd: CorrelateArgs) -> Result<usize> {
    let mut connection = establish_connection();
    let matching = if cmd.by_booking_date {
        Matching::ByBooking
    } else {
        Matching::BySpending
    };

    let term = Term::stdout();
    let mut correlation_command = CorrelationCommand {
        input_file: cmd.input,
        sheet_name: cmd.sheet_name,
        matching,
        verbose: cmd.verbose,
        list_extra_transactions: cmd.list_extra_transactions,
        account_query: cmd.account.build(None),
        counterparty_account_query: cmd.from_account.build(None),
        fee_account_query: cmd.fee_account.build(None),
    };

    let format = if cmd.rhai_scripts.is_empty() {
        cmd.format
            .clone()
            .and_then(|x| SheetFormat::new(&x))
            .with_context(|| format!("Unknown format:'{}'!", cmd.format.unwrap_or_default()))?
    } else {
        load_rhai_format(&cmd.rhai_scripts)?
    };
    correlation_command.execute(&mut connection, &term, &format)
}

fn handle_eval_script(args: EvalScriptArgs) -> Result<usize> {
    let term = Term::stdout();
    let format = load_rhai_format(&args.rhai_scripts)?;

    let mut sheet_definition = SheetDefinition::new(&args.input)?;
    let external_transactions =
        sheet_definition.load(args.sheet_name, Matching::ByBooking, &format, &term)?;
    let shown = external_transactions.0.len().min(args.limit);
    for transaction in external_transactions.0.iter().take(args.limit) {
        term.write_line(&format!(" - {}", style(transaction).cyan()))?;
    }
    term.write_line(&format!(
        "Parsed {} {}",
        style(&external_transactions.0.len()).cyan(),
        style("transactions").blue()
    ))?;
    if shown < external_transactions.0.len() {
        term.write_line(&format!(
            "Showing {} {}",
            style(&shown).cyan(),
            style("transactions").blue()
        ))?;
    }
    Ok(0)
}

fn load_rhai_format(rhai_scripts: &[PathBuf]) -> Result<SheetFormat> {
    let engine = build_rhai_engine();
    let mut ast = engine.compile(
        "fn parse_sheet_row(row) { new_external_transaction(None, None, None, None, None, None, None, None, None) }",
    )?;
    for script_path in rhai_scripts {
        let script_content = fs::read_to_string(script_path).with_context(|| {
            format!("Failed to read Rhai script from {}", script_path.display())
        })?;
        let compiled_script = engine.compile(script_content).with_context(|| {
            format!(
                "Failed to compile Rhai script from {}",
                script_path.display()
            )
        })?;
        ast = ast.merge(&compiled_script);
    }
    Ok(SheetFormat::new_rhai(ast, "parse_sheet_row".to_string()))
}
