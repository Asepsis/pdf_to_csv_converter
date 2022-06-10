# PDF to CSV converter
## Description
This programm will convert a pdf for swim-meets to a csv file.
You have to mention a team name by executing the programm with the command line.

The main Branch will convert meldeergebnis from EasyWK to a CSV file.

This is pretty specific and only works propably only with meldeergebnis from EasyWK.

## Compile
To build the .exe just run the following command in the terminal:
`cargo build --release`

## Example
1. Download the PDF from the source
2. Build the .exe
3. execute the programm in the command line

### Source:
https://hsv-sued.de/wp-content/uploads/2022/05/220514-ME-Darmstadt.pdf
### Input
`rs_pdf_extract.exe 220514-ME-Darmstadt.pdf "SVS Griesheim"`
### Ouput
wk.csv


## Table of Contents / Example output

Below you can see an example output from the programm.
`ZZ` is the short name for Zwischenzeit.

|WK|Uhrzeit|Lauf|Bahn|Name|Jahrgang|Verein|Zeit|ZZ|ZZ|ZZ|ZZ|ZZ|ZZ|ZZ|ZZ|
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
|100m Freistil|09:14|3|2|Schwimmer Name|2000|SVS Griesheim|01:43,94||||||||