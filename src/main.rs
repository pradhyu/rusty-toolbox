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

    /// Excel / Spreadsheet (.xlsx, .xls, .ods, .csv, .tsv) SQL Query Runner & Inspector
    #[command(alias = "excel", alias = "sheet", alias = "csv", alias = "tsv")]
    Xlsx {
        /// Path to .xlsx, .xls, .ods, or .csv spreadsheet file
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// SQL query to execute against worksheets (e.g. 'SELECT * FROM Sheet1')
        #[arg(value_name = "QUERY")]
        query: Option<String>,

        /// Launch interactive TUI spreadsheet browser
        #[arg(short, long)]
        interactive: bool,

        /// Export all worksheets as tables into an SQLite database file
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
        Commands::Xlsx {
            path,
            query,
            interactive,
            dump_sqlite,
        } => {
            modules::xlsx::execute_xlsx_command(path, query, interactive, dump_sqlite)?;
        }
    }

    Ok(())
}
