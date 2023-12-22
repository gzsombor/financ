use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    ListAccounts(ListAccountsArgs),
    Transactions(TransactionsArgs),
    Correlate(CorrelateArgs),
    Commodities(CommoditiesArgs),
}

#[derive(Args)]
pub struct ListAccountsArgs {
    // Limit number of accounts
    #[arg(long = "limit", short = 'l')]
    pub limit: Option<i64>,

    #[command(flatten)]
    pub account: DefaultAccountParams,
}

#[derive(Args)]
pub struct TransactionsArgs {
    // Limit number of splits
    #[arg(long = "limit", short = 'l')]
    pub limit: Option<i64>,

    // Splits with the given transaction id
    #[arg(long = "transaction-id", short = 'x')]
    pub txid: Option<String>,

    // Splits before the given date in yyyy-mm-dd format
    #[arg(long = "before", short = 'b')]
    pub before: Option<String>,

    // Splits after the given date in yyyy-mm-dd format
    #[arg(long = "after", short = 'f')]
    pub after: Option<String>,

    // Splits with the given memo
    #[arg(long = "memo", short = 'e')]
    pub memo: Option<String>,

    // Splits with the given description
    #[arg(long = "description", short = 'd')]
    pub description: Option<String>,

    // Move the found splits to the target account
    #[arg(long = "move-split", short = 'm')]
    pub move_split: bool,

    #[command(flatten)]
    pub account: DefaultAccountParams,
    #[command(flatten)]
    pub target_account: TargetAccountParams,
}

#[derive(Args)]
pub struct CorrelateArgs {
    // The file which contains a list of transaction to correlate
    #[arg(long = "input", short = 'i')]
    pub input: String,

    // The name of the sheet
    #[arg(long = "sheet-name", short = 's')]
    pub sheet_name: Option<String>,

    // The format of the sheet
    #[arg(long = "format", short = 'f')]
    pub format: Option<String>,

    // Match transactions by the booking date
    #[arg(long = "by-booking-date", short = 'd')]
    pub by_booking_date: bool,

    // List extra transactions not found in the external source
    #[arg(long = "list-extra-transactions", short = 'X')]
    pub list_extra_transactions: bool,

    // Verbose logging
    #[arg(long = "verbose", short = 'v')]
    pub verbose: bool,

    #[command(flatten)]
    pub account: DefaultAccountParams,

    #[command(flatten)]
    pub target_account: TargetAccountParams,

    #[command(flatten)]
    pub from_account: FromAccountParams,

    #[command(flatten)]
    pub fee_account: FeeAccountParams,
}

#[derive(Args)]
pub struct CommoditiesArgs {
    // List only a given type of commodities
    #[arg(long = "commodity-type", short = 'c')]
    pub commodity_type: Option<String>,

    // List only commodities with the given name
    #[arg(long = "name", short = 'n')]
    pub name: Option<String>,

    // Limit number of commodities
    #[arg(long = "limit", short = 'l')]
    pub limit: Option<i64>,
}

#[derive(Args)]
pub struct DefaultAccountParams {
    #[arg(long = "account-name", short = 'n')]
    pub name: Option<String>,
    #[arg(long = "account-parent", short = 'p')]
    pub parent_guid: Option<String>,
    #[arg(long = "account-guid", short = 'g')]
    pub guid: Option<String>,
    #[arg(long = "account-type", short = 't')]
    pub account_type: Option<String>,
}

#[derive(Args)]
pub struct TargetAccountParams {
    #[arg(long = "target-account-name", short = 'r')]
    pub name: Option<String>,
    #[arg(long = "target-account-parent", short = 'P')]
    pub parent_guid: Option<String>,
    #[arg(long = "target-account-guid", short = 'G')]
    pub guid: Option<String>,
    #[arg(long = "target-account-type", short = 'T')]
    pub account_type: Option<String>,
}

#[derive(Args)]
pub struct FeeAccountParams {
    #[arg(long = "fee-account-name", short = 'E')]
    pub name: Option<String>,
    #[arg(long = "fee-account-parent", short = 'R')]
    pub parent_guid: Option<String>,
    #[arg(long = "fee-account-guid", short = 'U')]
    pub guid: Option<String>,
    #[arg(long = "fee-account-type", short = 'Y')]
    pub account_type: Option<String>,
}

#[derive(Args)]
pub struct FromAccountParams {
    #[arg(long = "from-account-name", short = 'N')]
    pub name: Option<String>,
    #[arg(long = "from-account-parent", short = 'P')]
    pub parent_guid: Option<String>,
    #[arg(long = "from-account-guid", short = 'G')]
    pub guid: Option<String>,
    #[arg(long = "from-account-type", short = 'T')]
    pub account_type: Option<String>,
}
