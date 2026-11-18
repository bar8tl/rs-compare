//..+....1....+....2....+....3....+.....4....+....5....+....6....+....7....+....8...
// fnlib.rs - Functions library for IDOCs comparison (COMPARE program)
// (2026-08-01 bar8tl)
//
use rusqlite::Connection;

#[derive(Debug, Clone, Default)]
pub struct OccurTp {
  pub sap  : String,
  pub sgnam: String,
  pub qlkey: String,
  pub qlval: String,
  pub sgocc: i32
}

// Get segment type KA1/P15 occurence according r4/s4 qualifiers equivalences
pub fn get_occur(cnn: &Connection, p: &OccurTp,
  sgmid: &mut String, sgmoc: &mut i32, txtoc:  &mut i32) -> i32 {
  let occ: i32;
  // for only key fields into qualified segments
  if p.sgnam == "E1EDKA1" && p.qlkey == "PARVW"  ||
     p.sgnam == "E1EDP15" && p.qlkey == "TDNAME" {
    *sgmid = format!("{}+{}", p.sgnam.clone(), p.qlval.clone()); // full segment-id

    // for each e1edp15.tdnam=yadt, increment the text sequence number
    if p.sgnam == "E1EDP15" && (p.qlval == "YADT" || p.qlval == "YC10") {
      *txtoc = *txtoc + 1;
    }

    // look for segment name and key field within qualifier equivalences table
    let mut occur: i32 = 0;
    let s = format!("SELECT seq from qualf WHERE segm=? and {}=?;", p.sap);
    let _ = cnn.query_row(&s, [p.sgnam.clone(), p.qlval.clone()], |qual| {
      Ok( occur = qual.get(0).unwrap() )
    });
    // if qualif segm w/equival, return equival occ# else return next avail occur#
    if occur > 0 {
      occ = occur;
    } else {
      *sgmoc = *sgmoc + 1;
       occ   = *sgmoc;
    }
  } else {
  // for remaining fields into qualif segments, keep the current ocurr#
    occ = p.sgocc.clone();
  }
  return occ;
}
