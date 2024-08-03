use std::error::Error;
use std::path::PathBuf;

use chrono::Datelike;

use crate::{load_config, store_config, util};

fn full_path(path: &str) -> PathBuf {
    let mut full_path = PathBuf::from("./config");
    full_path.push(&path);
    full_path.with_extension("toml")
}

pub fn get(path: &str) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    println!("{path}: Getting custom events.");
    let config = load_config(&full_path(&path))?;
    println!("{path}: Done.");
    Ok(config.custom_events)
}

fn write(path: &str, custom_events: &Vec<(String, String)>) -> Result<(), Box<dyn Error>> {
    let full_path = full_path(&path);
    let mut config = load_config(&full_path)?;
    config.custom_events.clone_from(custom_events);
    store_config(&full_path, config)?;
    Ok(())
}

pub fn new() {
    let path = util::stdin_read_str("To which configuration would you like to add an event?", "");

    let mut custom_events = match get(&path) {
        Ok(custom_events) => custom_events,
        Err(error) => {
            println!("{path}: Failed to get custom events: {error}. Please try again.");
            return;
        }
    };

    println!("{path}: Please enter the date and time for the start of the event:");
    let custom_begin_datetime_str = util::stdin_get_date_time().to_string();

    println!("{path}: Please enter the date and time for the end of the event:");
    let custom_end_datetime_str = util::stdin_get_date_time().to_string();

    custom_events.push((custom_begin_datetime_str, custom_end_datetime_str));

    if let Err(error) = write(&path, &custom_events) {
        println!("{path}: Failed to write custom events to file: {error}. Please try again");
        return;
    }

    println!("{path}: Done.");
}

pub fn remove() {
    let path = util::stdin_read_str(
        "From which configuration would you like to remove an event?",
        "",
    );

    let mut custom_events = match get(&path) {
        Ok(custom_events) => custom_events,
        Err(error) => {
            println!("{path}: Failed to get custom events: {error}. Please try again.");
            return;
        }
    };

    if custom_events.is_empty() {
        println!("{path}: No custom events found!");
        return;
    }

    custom_events.sort_unstable();

    let mut i = 0;
    for (begin, end) in custom_events.iter() {
        println!("{}: {} - {}", i, begin, end);
        i += 1;
    }
    custom_events.remove(util::stdin_read_int_ranged(
        "Which event would you like to remove",
        0..=i - 1,
        0,
    ));

    if let Err(error) = write(&path, &custom_events) {
        println!("{path}: Failed to write custom events to file: {error}. Please try again");
        return;
    }

    println!("{path}: Done.");
}

pub fn purge(path: &str) -> Result<(), Box<dyn Error>> {
    println!("{path}: Purging old custom events.");
    let mut custom_events = get(path)?;

    if custom_events.is_empty() {
        println!("{path}: No custom events found!");
        return Ok(());
    }

    let now = chrono::Local::now().naive_local();
    custom_events.retain(|(begin, _)| {
        match chrono::NaiveDateTime::parse_from_str(begin, "%Y-%m-%d %H:%M:%S") {
            Ok(datetime) => !(datetime < now && datetime.month() < now.month() - 1),
            Err(_) => false,
        }
    });

    write(path, &custom_events)?;

    println!("{path}: Done.");
    Ok(())
}
