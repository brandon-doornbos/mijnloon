use std::error::Error;
use std::path::PathBuf;

use crate::config;
use chrono::{Datelike, NaiveDateTime};

fn full_path(path: &str) -> PathBuf {
    let mut full_path = PathBuf::from("./config");
    full_path.push(&path);
    full_path.with_extension("toml")
}

pub fn get(path: &str) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    println!("{path}: Getting custom events.");
    let config = config::load(&full_path(&path))?;
    println!("{path}: Done.");
    Ok(config.custom_events)
}

fn write(path: &str, custom_events: &Vec<(String, String)>) -> Result<(), Box<dyn Error>> {
    let full_path = full_path(&path);
    let mut config = config::load(&full_path)?;
    config.custom_events.clone_from(custom_events);
    config::store(&full_path, config)?;
    Ok(())
}

pub fn new(username: &str, start: &NaiveDateTime, end: &NaiveDateTime) {
    let mut custom_events = match get(username) {
        Ok(custom_events) => custom_events,
        Err(error) => {
            println!("{username}: Failed to get custom events: {error}. Please try again.");
            return;
        }
    };

    custom_events.push((start.to_string(), end.to_string()));

    if let Err(error) = write(username, &custom_events) {
        println!("{username}: Failed to write custom events to file: {error}. Please try again");
        return;
    }

    println!("{username}: Done.");
}

pub fn remove(username: &str, start: &NaiveDateTime, end: &NaiveDateTime) {
    let mut custom_events = match get(username) {
        Ok(custom_events) => custom_events,
        Err(error) => {
            println!("{username}: Failed to get custom events: {error}. Please try again.");
            return;
        }
    };

    let event = (start.to_string(), end.to_string());
    custom_events.retain(|x| x != &event);

    if let Err(error) = write(username, &custom_events) {
        println!("{username}: Failed to write custom events to file: {error}. Please try again");
        return;
    }
}

pub fn purge(path: &str) -> Result<(), Box<dyn Error>> {
    println!("{path}: Purging old custom events.");
    let mut custom_events = get(path)?;

    if custom_events.is_empty() {
        println!("{path}: No custom events found!");
        return Ok(());
    }

    let now = chrono::Local::now().naive_local();
    custom_events.retain(|(start, _)| {
        match chrono::NaiveDateTime::parse_from_str(start, "%Y-%m-%d %H:%M:%S") {
            Ok(datetime) => !(datetime < now && datetime.month() < now.month() - 1),
            Err(_) => false,
        }
    });

    write(path, &custom_events)?;

    println!("{path}: Done.");
    Ok(())
}
