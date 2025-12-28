use std::error::Error;
use std::path::PathBuf;

use chrono::{Datelike, NaiveDateTime};

use crate::config;
use crate::schedule::Event;

fn full_path(path: &str) -> PathBuf {
    let mut full_path = PathBuf::from("./config");
    full_path.push(&path);
    full_path.with_extension("toml")
}

pub fn get(path: &str) -> Result<Vec<Event>, Box<dyn Error>> {
    println!("{path}: Getting custom events.");
    let config = config::load(&full_path(&path))?;
    println!("{path}: Done.");
    Ok(config.custom_events)
}

fn write(path: &str, custom_events: &Vec<Event>) -> Result<(), Box<dyn Error>> {
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

    custom_events.push(Event {
        start_dt: start.to_string(),
        end_dt: end.to_string(),
        desc: None,
        hash: None,
    });

    if let Err(error) = write(username, &custom_events) {
        println!("{username}: Failed to write custom events to file: {error}. Please try again");
        return;
    }

    println!("{username}: Done.");
}

pub fn update(
    username: &str,
    orig_start: &str,
    orig_end: &str,
    start: &NaiveDateTime,
    end: &NaiveDateTime,
) {
    let mut config = match config::load(&full_path(username)) {
        Ok(config) => config,
        Err(error) => {
            println!("{username}: Failed to get config: {error}. Please try again.");
            return;
        }
    };

    let mut existing = false;
    for e in &mut config.custom_events {
        if e.start_dt == orig_start && e.end_dt == orig_end {
            e.start_dt = start.to_string();
            e.end_dt = end.to_string();
            existing = true;
            break;
        }
    }

    if !existing {
        let orig_start_rfc = orig_start.splitn(2, ' ').collect::<Vec<&str>>().join("T");
        let orig_end_rfc = orig_end.splitn(2, ' ').collect::<Vec<&str>>().join("T");
        for e in config.events {
            if e.start_dt == orig_start_rfc && e.end_dt == orig_end_rfc {
                config.custom_events.push(Event {
                    start_dt: start.to_string(),
                    end_dt: end.to_string(),
                    desc: e.desc,
                    hash: e.hash,
                });
                break;
            }
        }
    }

    if let Err(error) = write(username, &config.custom_events) {
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

    let event = Event {
        start_dt: start.to_string(),
        end_dt: end.to_string(),
        desc: None,
        hash: None,
    };
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
    custom_events.retain(|e| {
        match chrono::NaiveDateTime::parse_from_str(&e.start_dt, "%Y-%m-%d %H:%M:%S") {
            Ok(datetime) => !(datetime < now && datetime.month() < now.month() - 1),
            Err(_) => false,
        }
    });

    write(path, &custom_events)?;

    println!("{path}: Done.");
    Ok(())
}
