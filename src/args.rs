//..+....1....+....2....+....3....+.....4....+....5....+....6....+....7....+....8...
// args.rs - Declare structure of CLI subcommands and arguments for IDOCs comparison
// (COMPARE program) (2026-08-01 bar8tl)
//
use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Cli {
  #[command(subcommand)]
  command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
  /// Convert IDOC formats
  Convert  {
    /// IDOC file name
    file: String,
    /// IDOC source SAP system
    sap : String,
    /// Convert IDOC format from SAP hierarchical text to CSV
    #[arg(short, long)]
    csv : bool,
    /// Convert IDOC format from EDI flat text to CSV
    #[arg(short, long)]
    ufld: bool,
  },
  /// Compare two CSV formatted IDOCs
  Compare {
    /// File IDOC 1 in CSV format
    file1: String,
    /// File IDOC 2 in CSV format
    file2: String,
  },
}
