#![feature(let_chains)]

use crate::{
    storage::{
        credentials::clear_credentials,
        keys::{add_key, filter_keys},
    },
    sync::sync_toggle,
};
use clap::{Parser, Subcommand};
use colored::Colorize;

mod storage;
mod sync;
mod tempo;
mod toggl;
mod utils;

const EFFECTSOFT_ASCII: &str = r"  ______ ______ ______ ______ _____ _______ _____  ____  ______ _______ 
|  ____|  ____|  ____|  ____/ ____|__   __/ ____|/ __ \|  ____|__   __|
| |__  | |__  | |__  | |__ | |       | | | (___ | |  | | |__     | |   
|  __| |  __| |  __| |  __|| |       | |  \___ \| |  | |  __|    | |   
| |____| |    | |    | |___| |____   | |  ____) | |__| | |       | |   
|______|_|    |_|    |______\_____|  |_| |_____/ \____/|_|       |_|
";

#[derive(Subcommand, Debug)]
enum Command {
    /// - Run through the whole sync process
    Sync,
    /// - Allows you to add some keys without having to log for them
    AddKeys,
    /// - Go through the list of available keys and remove old ones
    FilterKeys,
    /// - Remove the credentials files
    ClearCredentials,
}

#[derive(Parser, Debug)]
#[command(author="Nicholas Brostrom", version, about="Sync Toggl to Tempo", long_about = None)]
struct Args {
    #[clap(subcommand)]
    cmd: Command,
    #[clap(short, long, default_value_t = false)]
    fast: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{}", EFFECTSOFT_ASCII.red());
    let arguments = Args::parse();
    match arguments.cmd {
        Command::Sync => sync_toggle().await,
        Command::AddKeys => add_key(),
        Command::FilterKeys => filter_keys(),
        Command::ClearCredentials => clear_credentials(),
    }
}
