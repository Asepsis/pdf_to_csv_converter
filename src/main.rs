use pdf_extract::*;
use regex;
use std::env;

#[derive(Debug, Eq, PartialEq, Clone)]
struct Bahn {
    bahn: String,
    name: String,
    jahrgang: String,
    verein: String,
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
fn convert_to_csv(wk: Vec<Wettkampf>) {
    let mut csv_string = String::new();
    csv_string.push_str("WK;Uhrzeit;Lauf;Bahn;Name;Jahrgang;Verein;Zeit;ZZ;ZZ;ZZ;ZZ;ZZ;ZZ;ZZ;ZZ;\n");
    for w in wk {
        // csv_string.push_str(&w.wettkampf);
        // csv_string.push_str(";");
        for l in w.lauf_list {
            // csv_string.push_str(&l.lauf);
            // csv_string.push_str(";");
            for b in l.bahn_list {
                csv_string.push_str(&w.wettkampf);
                csv_string.push_str(";");
                csv_string.push_str(&l.time);
                csv_string.push_str(";");
                csv_string.push_str(&l.lauf);
                csv_string.push_str(";");
                csv_string.push_str(&b.bahn);
                csv_string.push_str(";");
                csv_string.push_str(&b.name);
                csv_string.push_str(";");
                csv_string.push_str(&b.jahrgang);
                csv_string.push_str(";");
                csv_string.push_str(&b.verein);
                csv_string.push_str(";");
                csv_string.push_str(&b.zeit);
                csv_string.push_str(";;;;;;;;;\n");
            }
        }
    }
    std::fs::write("wk.csv", csv_string).unwrap();

}


fn main() {

    let args: Vec<String> = env::args().collect();

    //Extract text from PDF
    let file_path = &args[1]; //"wk.pdf";
    let verein_name = &args[2]; //"Verein";

    println!("File path: {}", file_path);
    println!("Verein name: {}", verein_name);

    let content = extract_text(file_path).unwrap();
    // println!("Content: {}", content);

    //Save as txt for debug purposes
    //std::fs::write("message.txt", &content).unwrap();

    //Try to solve Bahn 1 issues if Bahn 8 in is empty in the Lauf before
    // remove all lines with (Bahn\s+)(\d+)(:\s+)$ from content
    // let re = regex::Regex::new(r"(Bahn\s+)(\d+)(:\s+)$").unwrap();
    // let clean_content = re.replace_all(&content, "");
    // println!("{:#?}", clean_content);

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

    //Find all Bahn and there positions in the text
    let mut bahn_list: Vec<Bahn> = Vec::new();
    let re_bahn =
        regex::Regex::new(r"(Bahn\s+)(\d)\s\s(\w.+)\s\s+\s(\d\d\d\d)\s\s+(\w.+)\s\s+(\d\d:\d\d,\d\d)").unwrap();
    re_bahn.captures_iter(&content).for_each(|cap_bahn| {
        let bahn = Bahn {
            bahn: cap_bahn[2].to_string(),
            name: cap_bahn[3].to_string(),
            jahrgang: cap_bahn[4].to_string(),
            verein: cap_bahn[5].to_string(),
            zeit: cap_bahn[6].to_string(),
            byte_offset: cap_bahn.get(0).unwrap().start(),
        };

        if bahn.verein == verein_name.to_string() {
            bahn_list.push(bahn);
        }
        // bahn_list.push(bahn);
    });
    // println!("{:#?}", bahn_list);

    //Add Bahn to the appropriate Lauf
    lauf_list.iter_mut().rev().for_each(|lf| {
        lf.bahn_list.extend(
            bahn_list
                .iter()
                .cloned()
                .filter(|bahn| bahn.byte_offset > lf.byte_offset)
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
                .filter(|lf| lf.byte_offset > wk.byte_offset)
        );
        lauf_list.retain(|lf| lf.byte_offset < wk.byte_offset);
    });

    //Remove all empty Wettkampf
    wk_list.retain(|wk| !wk.lauf_list.is_empty());


    convert_to_csv(wk_list);

}

