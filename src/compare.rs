//..+....1....+....2....+....3....+.....4....+....5....+....6....+....7....+....8...
// compare.rs - Compare two IDocs in format CSV (2026-08-01 bar8tl)
//
use csv::Writer;
use rusqlite::Connection;
use serde::{Serialize, Deserialize};
use std::fs::File;

#[derive(Deserialize, Debug, Clone, Default)]
struct FieldTp {
  seq:  String,
  id :  String,
  dsc:  String,
  val:  String
}

#[derive(Serialize, Debug, Clone, Default)]
struct CompTp {
  rel1: String,
  // seq1: String,
  id1 : String,
  dsc1: String,
  val1: String,
  rel2: String,
  // seq2: String,
  id2 : String,
  dsc2: String,
  val2: String,
  opt2: String
}

pub fn compare(dbpath: &String, flnm1: &String, flnm2: &String) {
  let file1 = File::open(flnm1).unwrap();
  let file2 = File::open(flnm2).unwrap();
  let ofnam = flnm2.replace(".csv", "_comparison.csv");
  let mut w = Writer::from_path(ofnam).expect("Open file error");
  let cnn = Connection::open(dbpath).expect("Error opening DB");
  let mut rec1 : FieldTp = Default::default();
  let mut rec2 : FieldTp = Default::default();
  let mut seq1 : i32;
  let mut seq2 : i32;
  let mut eof1 = false;
  let mut eof2 = false;
  let mut rdr1 = csv::Reader::from_reader(file1);
  let mut rdr2 = csv::Reader::from_reader(file2);
  let mut itr1 = rdr1.deserialize();
  let mut itr2 = rdr2.deserialize();
  if let Some(line) = itr1.next() { rec1 = line.unwrap(); } else { eof1 = true; }
  if let Some(line) = itr2.next() { rec2 = line.unwrap(); } else { eof2 = true; }
  while !eof1 && !eof2 {
    seq1 = rec1.seq.parse::<i32>().unwrap();
    seq2 = rec2.seq.parse::<i32>().unwrap();
    if seq1 == seq2 {
      let mut rel = "=";
      if rec1.val != rec2.val {
        rel = "<>";
        if rec1.id == "E1EDP16.WMENG" && rec2.id == "E1EDP16.WMENG" {
          let wmeng1 = rec1.val.parse::<f32>().unwrap();
          let wmeng2 = rec2.val.parse::<f32>().unwrap();
          if wmeng1 == wmeng2 {
            rel = "=";
          }
        }
      }
      w.serialize(CompTp {
        rel1: rel.to_string(),
        // seq1: rec1.seq.clone(),
        id1 : rec1.id .clone(),
        dsc1: rec1.dsc.clone(),
        val1: rec1.val.clone(),
        rel2: rel.to_string(),
        // seq2: rec2.seq.clone(),
        id2 : rec2.id .clone(),
        dsc2: rec2.dsc.clone(),
        val2: rec2.val.clone(),
        opt2: get_segtype(&cnn, rec2.id.clone())
      }).expect("error");
      if let Some(line) = itr1.next() { rec1 = line.unwrap(); } else { eof1 = true; }
      if let Some(line) = itr2.next() { rec2 = line.unwrap(); } else { eof2 = true; }
    } else if seq1 < seq2 {
      w.serialize(CompTp {
        rel1: "+".to_string(),
        // seq1: rec1.seq.clone(),
        id1 : rec1.id .clone(),
        dsc1: rec1.dsc.clone(),
        val1: rec1.val.clone(),
        rel2: "-".to_string(),
        // seq2:  "".to_string(),
        id2 :  "".to_string(),
        dsc2:  "".to_string(),
        val2:  "".to_string(),
        opt2:  "".to_string()
      }).expect("error");
      if let Some(line) = itr1.next() { rec1 = line.unwrap(); } else { eof1 = true; }
    } else if seq1 > seq2 {
      w.serialize(CompTp {
        rel1: "-".to_string(),
        // seq1:  "".to_string(),
        id1 :  "".to_string(),
        dsc1:  "".to_string(),
        val1:  "".to_string(),
        rel2: "+".to_string(),
        // seq2: rec2.seq.clone(),
        id2 : rec2.id .clone(),
        dsc2: rec2.dsc.clone(),
        val2: rec2.val.clone(),
        opt2: get_segtype(&cnn, rec2.id.clone())
      }).expect("error");
      if let Some(line) = itr2.next() { rec2 = line.unwrap(); } else { eof2 = true; }
    }
  }
  while !eof1 {
    w.serialize(CompTp {
      rel1: "+".to_string(),
      // seq1: rec1.seq.clone(),
      id1 : rec1.id .clone(),
      dsc1: rec1.dsc.clone(),
      val1: rec1.val.clone(),
      rel2: "-".to_string(),
      // seq2:  "".to_string(),
      id2 :  "".to_string(),
      dsc2:  "".to_string(),
      val2:  "".to_string(),
      opt2:  "".to_string()
    }).expect("error");
    if let Some(line) = itr1.next() { rec1 = line.unwrap(); } else { eof1 = true; }
  }
  while !eof2 {
    w.serialize(CompTp {
      rel1: "-".to_string(),
      // seq1:  "".to_string(),
      id1 :  "".to_string(),
      dsc1:  "".to_string(),
      val1:  "".to_string(),
      rel2: "+".to_string(),
      // seq2: rec2.seq.clone(),
      id2 : rec2.id .clone(),
      dsc2: rec2.dsc.clone(),
      val2: rec2.val.clone(),
      opt2: get_segtype(&cnn, rec2.id.clone())
    }).expect("error");
    if let Some(line) = itr2.next() { rec2 = line.unwrap(); } else { eof2 = true; }
  }
  w.flush().expect("flush error");
}

fn get_segtype(cnn: &Connection, id: String) -> String {
  let mut fldtp : String = String::new();
  let _ = cnn.query_row("SELECT ftyp from s4dlfr WHERE id=?;", [id.clone()],
    |field| { Ok( fldtp = field.get(0).unwrap()) });
  return fldtp;
}
