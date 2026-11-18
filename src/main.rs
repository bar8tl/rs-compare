//..+....1....+....2....+....3....+.....4....+....5....+....6....+....7....+....8...
// main.rs - Main entry point to IDOCs comparison program (COMPARE program)
// (2026-08-01 bar8tl)
//
mod compare;
mod config;
mod fnlib;
mod tocsv;
mod unfold;

use crate::compare::compare;
use crate::config::get_config;
use crate::tocsv::tocsv;
use crate::unfold::unfold;

include!("args.rs");

const CONFIG_FILENAME: &str = "..\\..\\extras\\.comparerc.toml";

fn main() {
  let cli = Cli::parse();
  let rc = get_config(CONFIG_FILENAME);
  match &cli.command {
    Some(Commands::Convert{ file, sap, csv, ufld }) => {
             if *ufld { unfold(&rc.general.dbopt, &file, &sap);
      } else if *csv  { tocsv (&rc.general.dbopt, &file, &sap);
      }
    }
    Some(Commands::Compare{ file1, file2 }) => {
      compare(&rc.general.dbopt, 
        &format!("{}{}", &rc.compare.idcdr1, &file1).to_string(),
        &format!("{}{}", &rc.compare.idcdr2, &file2).to_string());
    }
    None => {}
  }
}
