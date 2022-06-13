use clap::{Arg, Command};
use pdf_extract::*;
use regex;
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq, Clone)]
struct Schwimmer {
    name: String,
    jahrgang: String,
    verein: String,
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Bahn {
    bahn: String,
    schwimmer: Schwimmer,
    zeit: String,
    byte_offset: usize,
}
#[derive(Debug, Eq, PartialEq, Clone)]
struct Lauf {
    lauf: String,
    time: String,
    bahn_list: Vec<Bahn>,
    byte_offset: usize,
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Wettkampf {
    wettkampf: String,
    lauf_list: Vec<Lauf>,
    byte_offset: usize,
}
/// Takes a Vector from Wettkampf and saves a formated .csv file in the root folder
/// # Arguments is a Vec<Wettkampf>
/// # Output wk.csv
fn convert_to_csv(wk: Vec<Wettkampf>, output_name: &str) {
    let mut csv_string = String::new();
    csv_string
        .push_str("WK;Uhrzeit;Lauf;Bahn;Name;Jahrgang;Verein;Zeit;ZZ;ZZ;ZZ;ZZ;ZZ;ZZ;ZZ;ZZ;\n");
    for w in wk {
        for l in w.lauf_list {
            for b in l.bahn_list {
                csv_string.push_str(&w.wettkampf);
                csv_string.push_str(";");
                csv_string.push_str(&l.time);
                csv_string.push_str(";");
                csv_string.push_str(&l.lauf);
                csv_string.push_str(";");
                csv_string.push_str(&b.bahn);
                csv_string.push_str(";");
                csv_string.push_str(&b.schwimmer.name);
                csv_string.push_str(";");
                csv_string.push_str(&b.schwimmer.jahrgang);
                csv_string.push_str(";");
                csv_string.push_str(&b.schwimmer.verein);
                csv_string.push_str(";");
                csv_string.push_str(&b.zeit);
                csv_string.push_str(";;;;;;;;;\n");
            }
        }
    }
    std::fs::write(output_name, csv_string).unwrap();
}

fn main() {
    // Commandline Args
    let matches = Command::new("PDF to CSV converter")
        .version("0.1.0")
        .author("Asepsis")
        .about("Converts a PDF to a CSV file")
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .takes_value(true)
                .help("Sets the file to use"),
        )
        .arg(
            Arg::new("verein")
                .short('v')
                .long("verein")
                .value_name("VEREIN")
                .takes_value(true)
                .help("Sets the verein to use"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("OUTPUT")
                .takes_value(true)
                .help("Sets the output filename"),
        )
        .get_matches();

    let file_path = matches.value_of("file").unwrap();
    let verein_name = matches.value_of("verein").unwrap_or("");
    let output_name = matches.value_of("output").unwrap_or("wk.csv");

    //File handling
    let content = match extract_text(file_path) {
        Ok(data) => { 
            println!("Successfully loaded file."); 
            data
        },
        Err(_) => { 
            println!("Problem opening the file.\nProgramm will exit.");
            return
        }
    };

    println!("File path: {}", file_path);
    println!("Verein name: {}", verein_name);
    println!("Output name: {}", output_name);
    

    //Find all Wettkampf and there positions in the text
    let re_wk = regex::Regex::new(r"(Wettkampf\s\d+)\s-\s(\d+m\s+\S+)\s(\S.+)").unwrap();
    let mut wk_list: Vec<Wettkampf> = Vec::new();
    re_wk.captures_iter(&content).for_each(|cap_wk| {
        let wk = Wettkampf {
            wettkampf: cap_wk[2].to_string(),
            lauf_list: Vec::new(),
            byte_offset: cap_wk.get(0).unwrap().start(),
        };

        wk_list.push(wk);
    });

    //Find all Lauf and there positions in the text
    let mut lauf_list: Vec<Lauf> = Vec::new();
    let re_lf = regex::Regex::new(r"(Lauf\s+)(\d+)/(\d+)\s\(ca.\s(\d+:\d+)\sUhr\)").unwrap();
    re_lf.captures_iter(&content).for_each(|cap_lf| {
        let lf = Lauf {
            lauf: cap_lf[2].to_string(),
            time: cap_lf[4].to_string(),
            bahn_list: Vec::new(),
            byte_offset: cap_lf.get(0).unwrap().start(),
        };

        lauf_list.push(lf);
    });

    //Swimmer HashMap
    let mut schwimmer_list: HashMap<String, Schwimmer> = HashMap::new();

    //Find all Bahn and there positions in the text
    let mut bahn_list: Vec<Bahn> = Vec::new();
    let re_bahn = regex::Regex::new(
        r"(?:\s*Bahn\s+\d+\s*)*Bahn\s+(\d+)\s+(\D+)\s+(\d+(?:/AK\s\d+)?)\s+(.+)\s+(\d+:\d+,\d+)",
    )
    .unwrap();
    re_bahn.captures_iter(&content).for_each(|cap_bahn| {

        let new_schwimmer = Schwimmer {
            name: cap_bahn[2].trim_end().to_string(),
            jahrgang: cap_bahn[3].to_string(),
            verein: cap_bahn[4].trim_end().to_string(),
        };


        let bahn = Bahn {
            bahn: cap_bahn[1].to_string(),
            schwimmer: new_schwimmer.clone(),
            zeit: cap_bahn[5].to_string(),
            byte_offset: cap_bahn.get(0).unwrap().start(),
        };
        
        if bahn.schwimmer.verein == verein_name.to_string() {
            schwimmer_list.insert(cap_bahn[2].trim_end().to_string(), new_schwimmer);
            bahn_list.push(bahn);
        } else if verein_name == "" {
            schwimmer_list.insert(cap_bahn[2].trim_end().to_string(), new_schwimmer);
            bahn_list.push(bahn);
        }
    });
    // println!("Bahn:\n{}", schwimmer_list.len());
    // println!("Found Swimmer:\n{:#?}", schwimmer_list);
    let amount_of_starts = bahn_list.len();
    //Add Bahn to the appropriate Lauf
    lauf_list.iter_mut().rev().for_each(|lf| {
        lf.bahn_list.extend(
            bahn_list
                .iter()
                .cloned()
                .filter(|bahn| bahn.byte_offset > lf.byte_offset),
        );
        bahn_list.retain(|bahn| bahn.byte_offset < lf.byte_offset);
    });

    //Remove all empty bahn_lists
    lauf_list.retain(|lf| !lf.bahn_list.is_empty());

    //Add Lauf to the appropriate Wettkampf
    wk_list.iter_mut().rev().for_each(|wk| {
        wk.lauf_list.extend(
            lauf_list
                .iter()
                .cloned()
                .filter(|lf| lf.byte_offset > wk.byte_offset),
        );
        lauf_list.retain(|lf| lf.byte_offset < wk.byte_offset);
    });

    //Remove all empty Wettkampf
    wk_list.retain(|wk| !wk.lauf_list.is_empty());

    convert_to_csv(wk_list, output_name);

    println!("Found: {} Swimmers", schwimmer_list.len());
    println!("Found: {} Starts", amount_of_starts);
    println!("Successfully converted PDF to CSV");
    
}
