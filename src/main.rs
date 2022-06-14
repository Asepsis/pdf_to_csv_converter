use clap::{Arg, Command};
use pdf_extract::*;
use regex;
use std::{collections::HashMap, io::BufReader, io::prelude::*, fs::File};
use colored::*;

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
        .version("0.1.1")
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
        .arg(
            Arg::new("check")
                .short('c')
                .long("check")
                .value_name("CHECK")
                .takes_value(false)
                .help("Compares amount of lines in the CSV file to the amount of lines after read PDF file"),
        )
        .get_matches();

    let file_path = matches.value_of("file").unwrap();
    let verein_name = matches.value_of("verein").unwrap_or("");
    let output_name = matches.value_of("output").unwrap_or("wk.csv");
    let check = matches.is_present("check");

    //File handling
    let content = match extract_text(file_path) {
        Ok(data) => { 
            println!("{}", "Successfully loaded file.".green()); 
            data
        },
        Err(_) => { 
            println!("{}", "Problem opening the file.\nProgramm will exit.".red());
            return
        }
    };

    println!("File path: {}", file_path.magenta());
    println!("Verein name: {}", verein_name.magenta());
    println!("Output name: {}", output_name.magenta());
    

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

    //Save amounts of starts
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

    println!("Swimmers found: {}", schwimmer_list.len().to_string().cyan());
    println!("Starts found: {}", amount_of_starts.to_string().cyan());
    

    if check {
        let csv_file = File::open(output_name).unwrap();
        let mut buf_reader = BufReader::new(csv_file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents).unwrap();
        //Count lines
        let csv_lines = contents.lines().count();

        if csv_lines-1 == amount_of_starts {
            println!("{}", "Successfully converted PDF to CSV".green());
        } else {
            println!("{}", "Problem checking CSV file.".red());
            println!("{}", "Something went wrong. Programm will exit.".red());
        }
    } else {
        println!("{}", "Converted PDF to CSV".yellow());
    }
    
    
    
}
