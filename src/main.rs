use clap::{Arg, ArgAction, Command};
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};

#[derive(Debug, Deserialize)]
struct Record {
    database_name: String,
    table_name: String,
    column_name: String,
}

fn load_data(path: &str) -> Vec<String> {
    let file_path = format!("{}/data.csv", path);
    let mut data = Vec::new();
    let file = File::open(file_path).expect("Failed to open CSV file");
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        data.push(line.trim().to_string());
    }

    data
}

fn load_data_with_db(path: &str, db: &str) -> HashMap<String, HashMap<String, Vec<String>>> {
    let file_path = format!("{}/data_{}.csv", path, db);
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(&file_path)
        .expect("Failed to open CSV file");

    let mut data: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();

    for result in rdr.deserialize() {
        let record: Record = result.expect("Failed to deserialize record");
        let db_name = record.database_name.trim().to_string();
        let tb_name = record.table_name.trim().to_string();
        let col_name = record.column_name.trim().to_string();

        data.entry(db_name)
            .or_default()
            .entry(tb_name)
            .or_default()
            .push(col_name);
    }

    data
}

fn main() {
    let matches = Command::new("autocompletels")
        .version("1.0")
        .author("Erwan B")
        .about("CLI tool for database autocompletion")
        .arg(
            Arg::new("path")
                .long("path")
                .action(ArgAction::Set)
                .required(true)
                .help("Path to the data file"),
        )
        .arg(
            Arg::new("init")
                .long("init")
                .action(ArgAction::SetTrue)
                .help("Initialize the CSV file"),
        )
        .arg(
            Arg::new("db")
                .long("db")
                .action(ArgAction::Append)
                .help("Database name"),
        )
        .arg(
            Arg::new("tb")
                .long("tb")
                .action(ArgAction::Append)
                .help("Table name"),
        )
        .get_matches();

    if matches.get_flag("init") {
        // Call the sh process to create the CSV file
        std::process::Command::new("output_tree.sh")
            .output()
            .expect("Failed to execute process");
        println!("CSV file initialized.");
    } else {
        let path: String = matches.get_one::<String>("path").unwrap().to_string();
        let db_names: Vec<_> = matches
            .get_many::<String>("db")
            .unwrap_or_default()
            .collect();
        let tb_names: Vec<_> = matches
            .get_many::<String>("tb")
            .unwrap_or_default()
            .collect();

        if db_names.is_empty() {
            let data = load_data(&path);
            println!("{}", data.join(","));
            // for item in data {
            //     println!("{}", item);
            // }
        } else if tb_names.is_empty() {
            for db_name in db_names {
                let data = load_data_with_db(&path, db_name);
                if let Some(tables) = data.get(db_name) {
                    println!("{}", tables.keys().cloned().collect::<Vec<_>>().join(","));
                } else {
                    println!("Database {} not found.", db_name);
                }
            }
        } else {
            if db_names.len() != tb_names.len() {
                eprintln!("Error: The number of --db and --tb arguments must be the same.");
                return;
            }

            let mut results = Vec::new();
            for (db_name, tb_name) in db_names.iter().zip(tb_names.iter()) {
                let data = load_data_with_db(&path, db_name);
                if let Some(tables) = data.get(*db_name) {
                    if let Some(columns) = tables.get(*tb_name) {
                        results.push(columns.join(","));
                    } else {
                        results.push(format!(
                            "Table {} not found in database {}.",
                            tb_name, db_name
                        ));
                    }
                } else {
                    results.push(format!("Database {} not found.", db_name));
                }
            }
            println!("{}", results.join(","));
        }
    }
}
