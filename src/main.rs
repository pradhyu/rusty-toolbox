mod modules;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "rtb")]
#[command(about = "🧰 Rusty Toolbox (rtb): Ultra-fast, cross-platform developer tools", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// HyperSQL (HSQLDB) Inspector, Query Runner, and SQLite Converter
    Hsql {
        /// Path to HSQLDB folder, .script file, or .properties file
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// SQL query to execute directly against the database
        #[arg(value_name = "QUERY")]
        query: Option<String>,

        /// Launch interactive TUI database browser
        #[arg(short, long)]
        interactive: bool,

        /// Export HSQLDB tables to an SQLite database file
        #[arg(long, value_name = "OUT_SQLITE_FILE")]
        dump_sqlite: Option<PathBuf>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Hsql {
            path,
            query,
            interactive,
            dump_sqlite,
        } => {
            modules::hsql::execute_hsql_command(path, query, interactive, dump_sqlite)?;
        }
    }

    Ok(())
}
