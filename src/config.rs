//..+....1....+....2....+....3....+.....4....+....5....+....6....+....7....+....8...
// config.rs - Read general and function specific setting to build the configuration
// parameters structure for IDOCs comparison (COMPARE program) (2026-08-01 bar8tl)
//
use rblib::ownpath::ownpath;
use serde::Deserialize;
use std::fs::read_to_string;
use toml::from_str;

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigTp {
  pub general: GeneralTp,
  pub compare: CompareTp
}

#[derive(Clone, Debug, Deserialize)]
pub struct GeneralTp {
  pub home : String,
  pub dbopt: String
}

#[derive(Clone, Debug, Deserialize)]
pub struct CompareTp {
  pub idcdr1: String,
  pub idcdr2: String
}

pub fn get_config(fname: &str) -> ConfigTp {
  let confg = read_to_string(fname).expect("Failed to read file");
  let mut rc: ConfigTp = from_str(&confg).expect("Failed to deserialize");
  rc.compare.idcdr1 = ownpath(&rc.general.home, &rc.compare.idcdr1);
  rc.compare.idcdr2 = ownpath(&rc.general.home, &rc.compare.idcdr2);
  return rc;
}
