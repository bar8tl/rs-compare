	//..+....1....+....2....+....3....+.....4....+....5....+....6....+....7....+....8...
// tocsv.rs - Convert IDOC content from SAP Hierarchical text format to Columnar CSV
// format for subsquent IDOCs comparison (COMPARE program) (2026-08-01 bar8tl)
//
use crate::fnlib::{OccurTp, get_occur};
use csv::Writer;
use rusqlite::Connection;
use serde::Serialize;
use std::fs::File;
use std::io::{BufRead, BufReader};

const EDIDC : &str = "EDIDC";
const EDIDD : &str = "EDIDD";
const EDIDS : &str = "EDIDS";
const SEGNUM: &str = "SEGNUM";
const SEGNAM: &str = "SEGNAM";

#[derive(Serialize, Debug, Clone, Default)]
struct FieldTp {
  seq: String,
  id : String,
  dsc: String,
  val: String
}

pub fn tocsv(dbpath: &String, flnam: &String, fltyp: &String) {
  let ifnam = flnam.clone();
  let ofnam = ifnam.replace(".txt", ".csv");
  let mut sectn = String::new();
  let mut segmt = String::new();
  let mut wsgky = String::new();
  let mut techs = String::new();
  let mut users = String::new();
  let mut sgmid = String::new();
  let mut segno : i32 = 0;
  let mut fldno : i32 = 0;
  let mut socur : i32 = 0;
  let mut idcsq : i32 = -1;
  let mut k1aoc : i32 = 6;
  let mut p15oc : i32 = 2;
  let mut txtoc : i32 = 0;
  let mut first  = true;
  let mut inyadt = false;
  let mut inyc10 = false;
  let ifile = File::open(ifnam).unwrap();
  let mut w = Writer::from_path(ofnam).expect("Open file error");
  let cnn = Connection::open(dbpath).expect("Error opening DB");
  let rdr = BufReader::new(ifile);
  for wlin in rdr.lines() {
    let wlin = wlin.unwrap();
    let line = wlin.trim();
    let tokn : Vec<&str> = line.split('\t').collect();
    if line.len() == 0 { // ignores lines in blank
      continue;
    }

    // Obtain IDoc number and initialize wkidoc table (for segment correct sort)
    if tokn.len() == 1 && line[0..11] == "IDoc Number".to_string() {
      let idtkn: Vec<&str> = line.split(" : ").collect();
      idcsq += 1;
      cnn.execute("DELETE FROM wkidoc;", ()).expect("Table not reset");
      cnn.execute("INSERT INTO wkidoc VALUES (?1,?2,?3,?4)",
        (format!("{:01}0000000", idcsq), "IDOC", "IDoc Number", idtkn[1]))
        .expect("Row not inserted");
      continue;
    }

    // Ignore lines no containing tabulators (after to have obtained IDOC number)
    if tokn.len() <= 1 {
      continue;
    }

    // Determine data section to analyze
    if tokn[0] == EDIDC || tokn[0] == EDIDD || tokn[0] == EDIDS {
      sectn = tokn[0].to_string();
      continue;
    }

    // Check in segment number to analyze
    if tokn[0] == SEGNUM && tokn.len() == 3 {
      continue;
    }

    // Check in segment name to analyze and determine ocurrence number for segments
    // with multiple occurrences, i.e. e1edka1, e1edp15, e1edp16
    if tokn[0] == SEGNAM && tokn.len() == 3 {
      segmt = tokn[2].to_string(); // ex: E1EDK09
      cnn.query_row("SELECT rclas,dtype,seqno FROM items where idocn=? and \
        rname=? and dtype=?;", ["DELFOR02","SEGMENT",&segmt], |row| { Ok({
          techs = row.get(0).unwrap(); // segment technical name, ex: E2EDK0903
          users = row.get(1).unwrap(); // segment users name, ex: E1EDK09
          segno = row.get(2).unwrap(); // segment sap sequence number, ex: 1
        })
      }).expect("Error: Segment type not found in repository");
      if first {
        wsgky = segmt.clone();
        sgmid = segmt.clone();
        first = false;
      }
      if segmt == wsgky {
        socur += 1;
      } else {
        wsgky = segmt.clone();
        socur = 1;
      }
      continue;
    }

    // Process fields of each data section
    // Convert Control Record field by field
    if sectn == EDIDC {
      cnn.query_row("SELECT seqno FROM items where idocn=? and rname=? and \
        dname=?;", ["DELFOR02","CONTROL",tokn[0]], |row| { Ok({
          fldno = row.get(0).unwrap();
        })
      }).expect("Error");
      if tokn.len() == 2 {
        cnn.execute("INSERT INTO wkidoc VALUES (?1,?2,?3,?4)", (
          format!("{:01}{:02}{:02}0{:02}", idcsq, 0, 1, fldno), format!("{}.{}",
          sectn, tokn[0]), tokn[1], "")).expect("Row not inserted");
      } else if tokn.len() == 3 {
        cnn.execute("INSERT INTO wkidoc VALUES (?1,?2,?3,?4)", (
          format!("{:01}{:02}{:02}0{:02}", idcsq, 0, 1, fldno), format!("{}.{}",
          sectn, tokn[0]), tokn[1], tokn[2])).expect("Row not inserted");
      }

    // Convert Data Records field by field
    } else if sectn == EDIDD {
      cnn.query_row("SELECT seqno FROM items where idocn=? and rname=? and \
        dname=?;", ["DELFOR02",techs.as_str(),tokn[0]], |row| { Ok({
          fldno = row.get(0).unwrap();
        })
      }).expect("Error");

      if users == "E1EDKA1" {
        let pftkn: Vec<&str> = tokn[2].split(':').collect();
        let p = OccurTp {sap: fltyp.clone(), sgnam: segmt.clone(),
          qlkey: tokn[0].to_string(), qlval: pftkn[0].trim().to_string(),
          sgocc: socur.clone()};
        socur = get_occur(&cnn, &p, &mut sgmid, &mut k1aoc, &mut txtoc);
      } else if users == "E1EDP15" {
        let p = OccurTp {sap: fltyp.clone(), sgnam: segmt.clone(),
          qlkey: tokn[0].to_string(), qlval: tokn[2].to_string(),
          sgocc: socur.clone()};
        socur = get_occur(&cnn, &p, &mut sgmid, &mut p15oc, &mut txtoc);
        if tokn[0] == "TDNAME" && tokn[2] == "YADT" { // Put YADT qualfs together
          if !inyadt {
            inyadt = true;
          } else {
            continue;
          }
        }
        if tokn[0] == "TDNAME" && tokn[2] == "YC10" { // Put YC10 qualfs together
          if !inyc10 {
            inyc10 = true;
          } else {
            continue;
          }
        }
      } else {
        sgmid = segmt.clone();
        txtoc = 0;
      }

      if tokn.len() == 2 {
        cnn.execute("INSERT INTO wkidoc VALUES (?1,?2,?3,?4)", (
          format!("{:01}{:02}{:02}{:01}{:02}", idcsq, segno, socur, txtoc, fldno),
          format!("{}.{}", sgmid, tokn[0]), tokn[1],
          "")).expect("Row not inserted");
      } else if tokn.len() == 3 {
        cnn.execute("INSERT INTO wkidoc VALUES (?1,?2,?3,?4)", (
          format!("{:01}{:02}{:02}{:01}{:02}", idcsq, segno, socur, txtoc, fldno),
          format!("{}.{}", sgmid, tokn[0]), tokn[1],
          tokn[2])).expect("Row not inserted");
      }

    // Ignore Status Records at all
    } else if sectn == EDIDS {
      continue;
    }
  }

  // Sorts segments/fields in correct sequence and write the CSV line
  let mut stmt = cnn.prepare(
    "SELECT seq,id,dsc,val FROM wkidoc order by seq;").unwrap();
  let rows = stmt.query_map([], |row| { Ok(
    FieldTp {
      seq: row.get(0).unwrap(),
      id : row.get(1).unwrap(),
      dsc: row.get(2).unwrap(),
      val: row.get(3).unwrap()
    })
  }).unwrap();
  for row in rows {
    let d = row.unwrap();
    if d.val != "" && d.val != "0" {
      w.serialize(d).expect("error");
    }
  }
  w.flush().expect("flush error");
}
