use clap::Parser;
use colored::*;
use pdf_extract::*;
use regex;
use std::{collections::HashMap, fs::File, io::prelude::*, io::BufReader};

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Path to the PDF file to be processed
    #[clap(short, long)]
    file: String,
    /// Output file name
    #[clap(short, long, default_value = "wk.csv")]
    output: String,
    /// Name of the club
    #[clap(short, long)]
    club: String,
    /// Turn on debug mode
    #[clap(short, long)]
    debug: bool,
    /// Turn on check mode
    #[clap(short, long)]
    validate: bool,
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Swimmer {
    name: String,
    year: String,
    club: String,
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Lane {
    lane: String,
    swimmer: Swimmer,
    time: String,
    byte_offset: usize,
}
#[derive(Debug, Eq, PartialEq, Clone)]
struct Run {
    run: String,
    time: String,
    lane_list: Vec<Lane>,
    byte_offset: usize,
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Competition {
    competition: String,
    run_list: Vec<Run>,
    byte_offset: usize,
}

/// Takes a Vector from Wettkampf and saves a formated .csv file in the root folder
/// # Arguments is a Vec<Wettkampf>
/// # Output wk.csv
fn convert_to_csv(wk: Vec<Competition>, output_name: &str) {
    let mut csv_string = String::new();
    csv_string
        .push_str("WK;Uhrzeit;Lauf;Bahn;Name;Jahrgang;Verein;Zeit;ZZ;ZZ;ZZ;ZZ;ZZ;ZZ;ZZ;ZZ;\n");
    for w in wk {
        for l in w.run_list {
            for b in l.lane_list {
                csv_string.push_str(&w.competition);
                csv_string.push_str(";");
                csv_string.push_str(&l.time);
                csv_string.push_str(";");
                csv_string.push_str(&l.run);
                csv_string.push_str(";");
                csv_string.push_str(&b.lane);
                csv_string.push_str(";");
                csv_string.push_str(&b.swimmer.name);
                csv_string.push_str(";");
                csv_string.push_str(&b.swimmer.year);
                csv_string.push_str(";");
                csv_string.push_str(&b.swimmer.club);
                csv_string.push_str(";");
                csv_string.push_str(&b.time);
                csv_string.push_str(";;;;;;;;;\n");
            }
        }
    }
    std::fs::write(output_name, csv_string).unwrap();
}

fn main() {
    // Commandline Args
    let args = Cli::parse();
    let file_path = args.file;
    let club_name = args.club;
    let output_name = args.output;
    let debug = args.debug;
    let validate = args.validate;

    //File handling
    let content = match extract_text(&file_path) {
        Ok(data) => {
            println!("{}", "Successfully loaded file.".green());
            data
        }
        Err(_) => {
            println!("{}", "Problem opening the file.\nProgramm will exit.".red());
            return;
        }
    };

    // Save content to file
    if debug {
        let mut file = File::create("debug.txt").unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    println!("File path: {}", file_path.magenta());
    println!("Club name: {}", club_name.magenta());
    println!("Output name: {}", output_name.magenta());

    //Find all Wettkampf and there positions in the text
    let re_comp = regex::Regex::new(r"(Wettkampf\s\d+)\s-\s(\d+\s*m\s+\S+)\s(\S.+)").unwrap();
    let mut comp_list: Vec<Competition> = Vec::new();
    re_comp.captures_iter(&content).for_each(|cap_comp| {
        let comp = Competition {
            competition: cap_comp[2].to_string(),
            run_list: Vec::new(),
            byte_offset: cap_comp.get(0).unwrap().start(),
        };
        if debug {
            println!("{}", "Competition found: ".red());
            println!("{}", comp.competition.magenta());
        }
        comp_list.push(comp);
    });

    //Find all Lauf and there positions in the text
    let mut run_list: Vec<Run> = Vec::new();
    let re_run = regex::Regex::new(r"(Lauf\s+)(\d+)/(\d+)\s\(ca.\s(\d+:\d+)\sUhr\)").unwrap();
    re_run.captures_iter(&content).for_each(|cap_run| {
        let run = Run {
            run: cap_run[2].to_string(),
            time: cap_run[4].to_string(),
            lane_list: Vec::new(),
            byte_offset: cap_run.get(0).unwrap().start(),
        };

        run_list.push(run);
    });

    //Swimmer HashMap
    let mut swimmer_list: HashMap<String, Swimmer> = HashMap::new();

    //Find all Bahn and there positions in the text
    let mut lane_list: Vec<Lane> = Vec::new();
    let re_lane = regex::Regex::new(
        r"(?:\s*Bahn\s+\d+\s*)*Bahn\s+(\d+)\s+(\D+)\s+(\d+(?:/AK\s\d+)?)\s+(.+)\s+(\d+:\d+,\d+)",
    )
    .unwrap();
    re_lane.captures_iter(&content).for_each(|cap_lane| {
        let new_swimmer = Swimmer {
            name: cap_lane[2].trim_end().to_string(),
            year: cap_lane[3].to_string(),
            club: cap_lane[4].trim_end().to_string(),
        };

        let lane = Lane {
            lane: cap_lane[1].to_string(),
            swimmer: new_swimmer.clone(),
            time: cap_lane[5].to_string(),
            byte_offset: cap_lane.get(0).unwrap().start(),
        };

        if lane.swimmer.club == club_name.to_string() {
            if debug {
                println!("{}: {:#?}", "Swimmer".red(), new_swimmer);
                println!("{}: {:#?}", "Lane".red(), lane);
            }
            swimmer_list.insert(cap_lane[2].trim_end().to_string(), new_swimmer);
            lane_list.push(lane);
        } else if club_name == "" {
            swimmer_list.insert(cap_lane[2].trim_end().to_string(), new_swimmer);
            lane_list.push(lane);
        }
    });

    //Save amounts of starts
    let amount_of_starts = lane_list.len();

    //Add Bahn to the appropriate Lauf
    run_list.iter_mut().rev().for_each(|run| {
        run.lane_list.extend(
            lane_list
                .iter()
                .cloned()
                .filter(|lane| lane.byte_offset > run.byte_offset),
        );
        lane_list.retain(|lane| lane.byte_offset < run.byte_offset);
    });

    //Remove all empty bahn_lists
    run_list.retain(|run| !run.lane_list.is_empty());

    //Add Lauf to the appropriate Wettkampf
    comp_list.iter_mut().rev().for_each(|comp| {
        comp.run_list.extend(
            run_list
                .iter()
                .cloned()
                .filter(|run| run.byte_offset > comp.byte_offset),
        );
        run_list.retain(|run| run.byte_offset < comp.byte_offset);
    });

    //Remove all empty Wettkampf
    comp_list.retain(|comp| !comp.run_list.is_empty());

    if debug {
        println!("{:#?}", comp_list);
        println!("{:#?}", swimmer_list);
    }

    convert_to_csv(comp_list, &output_name);

    println!("Swimmers found: {}", swimmer_list.len().to_string().cyan());
    println!("Starts found: {}", amount_of_starts.to_string().cyan());

    //Check if amount of lines in the CSV file is equal to amount of lines after read PDF file
    //In fact the CSV file has one line more than the PDF file reader because of the header
    if validate {
        let csv_file = File::open(output_name).unwrap();
        let mut buf_reader = BufReader::new(csv_file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents).unwrap();
        let csv_lines = contents.lines().count();

        if csv_lines - 1 == amount_of_starts {
            println!("{}", "Successfully converted PDF to CSV".green());
        } else {
            println!("{}", "Problem checking CSV file.".red());
            println!("{}", "Something went wrong. Programm will exit.".red());
        }
    } else {
        println!("{}", "Converted PDF to CSV".yellow());
    }
}
