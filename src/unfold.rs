//..+....1....+....2....+....3....+.....4....+....5....+....6....+....7....+....8...
// unfold.rs - Convert IDOC format from EDI flat format to columnar CSV format for
// subsequent IDOCs comparison (COMPARE program) (2026-08-01 bar8tl)
//
use crate::fnlib::{OccurTp, get_occur};
use csv::Writer;
use rusqlite::Connection;
use serde::Serialize;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone, Default)]
struct SegmtTp {
  sgkey: String,
  techs: String,
  users: String,
  seqno: i32
}

#[derive(Debug, Clone, Default)]
struct ItemsTp {
  dname: String,
  dtext: String,
  seqno: i32,
  strps: i32,
  endps: i32
}

#[derive(Serialize, Debug, Clone, Default)]
struct FieldTp {
  seq: String,
  id : String,
  dsc: String,
  val: String
}

pub fn unfold(dbpath: &String, flnam: &String, fltyp: &String) {
  let mut s: SegmtTp = Default::default();
  let mut segmt = String::new();
  let mut wsgky = String::new();
  let mut sgmid = String::new();
  let mut socur : i32  = 0;
  let mut idcsq : i32  = -1;
  let mut k1aoc : i32  = 6;
  let mut p15oc : i32  = 3;
  let mut txtoc : i32  = 0;
  let mut first  = true;
  let mut inyadt = false;
  let mut inyc10 = false;
  let ifnam = flnam.clone();
  let ofnam = ifnam.replace(".txt", ".csv");
  let ifile = File::open(ifnam).unwrap();
  let mut w = Writer::from_path(ofnam).expect("Open file error");
  let cnn = Connection::open(dbpath).expect("Error opening DB");
  let rdr = BufReader::new(ifile);
  for wlin in rdr.lines() {
    let line  = wlin.unwrap();
    if line.len() == 0 { // ignores lines in blank
      continue;
    }

    // For equal segment types assign sequential occurrence number
    s.sgkey = line[0..7 ].trim().to_string(); // segment tsim ID
    s.techs = line[0..10].trim().to_string(); // segment technical ID
    if first {
      wsgky = s.sgkey.clone();
      first = false;
    }
    if s.sgkey == wsgky {
      socur += 1;
    } else {
      socur = 1;
      wsgky = s.sgkey.clone();
    }

    // Determine segment number according IDoc type definition
    if s.sgkey == "EDI_DC4".to_string() {
      s.techs = "CONTROL".to_string();
      s.users = "EDIDC".to_string();
      s.seqno = 0;
      idcsq  += 1;
      socur   = 0;
      k1aoc   = 6;
      p15oc   = 3;
      cnn.execute("DELETE FROM wkidoc;", ()).expect("Table not reset");
    } else {
      cnn.query_row(
        "SELECT rclas,dtype,seqno FROM items WHERE idocn=? and rname=? and \
        substring(dname,1,7)=?;", ["DELFOR02","SEGMENT",&s.sgkey[0..7]], |row| {
        Ok({
          s.techs = row.get(0).unwrap(); // segment technical name
          s.users = row.get(1).unwrap(); // segment users name
          s.seqno = row.get(2).unwrap(); // segment sequence number
        })
      }).expect("Error: Segment type not found in repository");
      segmt = s.users.clone();
    }

    // Get segment's fields number and start/end position into the text line
    let mut stmt = cnn.prepare(
      "SELECT dname,dtext,seqno,strps,endps FROM items where idocn=? and \
       rname=? order by strps;"
      ).unwrap();
    let rows = stmt.query_map(["DELFOR02", &s.techs], |row| { Ok(
      ItemsTp {
        dname: row.get(0).unwrap(), // field name
        dtext: row.get(1).unwrap(), // field description
        seqno: row.get(2).unwrap(), // field sequence number
        strps: row.get(3).unwrap(), // field start position
        endps: row.get(4).unwrap()  // field end position
      })
    }).unwrap();
    let sgmdt: Vec<char> = line.chars().collect();

    // Scan IDoc field by field according start-end position, determine final
    // occurrence number and write the resulting CSV lines
    for row in rows {
      let d = row.unwrap();
      let eolps: i32;

      // Make sure not to look for fields characters when line length is less than
      // end position
      match sgmdt.get((d.endps-1) as usize) {
        Some(_val) => { eolps = d.endps.clone() },
        None       => { eolps = line.len() as i32; continue }
      }
      match sgmdt.get((d.strps-0) as usize) {
        Some(_val) => {},
        None       => { continue }
      }
      let val = line[(d.strps-1) as usize..eolps as usize].trim();

      // In case of segment type r3/s4 equivalences, get the equivalence occurrence
      // number otherwise use a new occurrence number
      if s.users == "E1EDKA1" {
        let p = OccurTp {sap: fltyp.clone(), sgnam: segmt.clone(),
          qlkey: d.dname.clone(), qlval: val.to_string(),
          sgocc: socur.clone()};
        socur = get_occur(&cnn, &p, &mut sgmid, &mut k1aoc, &mut txtoc);
      } else if s.users == "E1EDP15" {
        let p = OccurTp {sap: fltyp.clone(), sgnam: segmt.clone(),
          qlkey: d.dname.clone(), qlval: val.to_string(),
          sgocc: socur.clone()};
        socur = get_occur(&cnn, &p, &mut sgmid, &mut p15oc, &mut txtoc);
        if d.dname == "TDNAME" && val == "YADT" { // Put YADT qualfs together
          if !inyadt {
            inyadt = true;
          } else {
            continue;
          }
        }
        if d.dname == "TDNAME" && val == "YC10" { // Put YC10 qualfs together
          if !inyc10 {
            inyc10 = true;
          } else {
            continue;
          }
        }
      } else {
        sgmid = s.users.clone();
        txtoc = 0;
      }

      // Write the output CSV field line
      if val != "" && val != "0" {
        cnn.execute("INSERT INTO wkidoc VALUES (?1,?2,?3,?4)", (
          format!("{:01}{:02}{:02}{:01}{:02}", idcsq, s.seqno, socur, txtoc,
            d.seqno),
          format!("{}.{}", sgmid, d.dname), d.dtext, val))
            .expect("Row not inserted");
      }
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
