use clap::{Arg, ArgAction, Command};
use csv::Reader;
use lazy_static::lazy_static;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Deserialize)]
struct Record {
    database_name: String,
    table_name: String,
    column_name: String,
}

lazy_static! {
    static ref DATA: Mutex<HashMap<String, HashMap<String, Vec<String>>>> = Mutex::new(HashMap::new());
}

fn load_data() {
    let mut rdr = Reader::from_path("data.csv").expect("Failed to open CSV file");
    let mut data = DATA.lock().unwrap();

    for result in rdr.deserialize() {
        let mut record: Record = result.expect("Failed to deserialize record");
        record.database_name = record.database_name.trim().to_string();
        record.table_name = record.table_name.trim().to_string();
        record.column_name = record.column_name.trim().to_string();

        data.entry(record.database_name.clone())
            .or_insert_with(HashMap::new)
            .entry(record.table_name.clone())
            .or_insert_with(Vec::new)
            .push(record.column_name);
    }
}

fn main() {
    let matches = Command::new("autocompletels")
        .version("1.0")
        .author("Erwan B")
        .about("CLI tool for database autocompletion")
        .arg(Arg::new("init")
            .long("init")
            .action(ArgAction::SetTrue)
            .help("Initialize the CSV file"))
        .arg(Arg::new("db")
            .long("db")
            .action(ArgAction::Append)
            .help("Database name"))
        .arg(Arg::new("tb")
            .long("tb")
            .action(ArgAction::Append)
            .help("Table name"))
        .get_matches();

    if matches.get_flag("init") {
        // Call the bteq process to create the CSV file
        std::process::Command::new("output_tree.sh").output().expect("Failed to execute process");
        println!("CSV file initialized.");
    } else {
        load_data();

        let db_names: Vec<_> = matches.get_many::<String>("db").unwrap_or_default().collect();
        let tb_names: Vec<_> = matches.get_many::<String>("tb").unwrap_or_default().collect();

        let data = DATA.lock().unwrap();

        if db_names.is_empty() {
            println!("{}", data.keys().cloned().collect::<Vec<_>>().join(","));
            return;
        }

        if tb_names.is_empty() {
            for db_name in db_names {
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
                if let Some(tables) = data.get(*db_name) {
                    if let Some(columns) = tables.get(*tb_name) {
                        results.push(columns.join(","));
                    } else {
                        results.push(format!("Table {} not found in database {}.", tb_name, db_name));
                    }
                } else {
                    results.push(format!("Database {} not found.", db_name));
                }
            }
            println!("{}", results.join(","));
        }
    }
}

