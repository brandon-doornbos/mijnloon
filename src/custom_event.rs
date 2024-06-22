use crate::util::{stdin_get_date_time, stdin_read_int};
use file_lock::{FileLock, FileOptions};
use std::error::Error;
use std::io::prelude::*;

pub static PATH: &str = "custom_events.json";

pub fn get() -> Result<Vec<(String, String)>, Box<dyn Error>> {
    println!("Getting custom events.");

    let options = FileOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .read(true);

    // FIXME: error handling
    let mut lock = match FileLock::lock(PATH, true, options) {
        Ok(lock) => lock,
        Err(err) => panic!("Error getting file lock: {}", err),
    };

    let mut custom_events_str = String::new();
    // FIXME: error handling
    lock.file.read_to_string(&mut custom_events_str)?;

    let custom_events: Vec<(String, String)> =
        serde_json::from_str(&custom_events_str).unwrap_or_default();

    println!("Done.");
    Ok(custom_events)
}

pub fn new() -> Result<(), Box<dyn Error>> {
    println!("Adding new event.");

    let mut custom_events = get()?;

    println!("Please enter the date and time for the start of the event:");
    let custom_begin_datetime_str = stdin_get_date_time().to_string();
    println!("Please enter the date and time for the end of the event:");
    let custom_end_datetime_str = stdin_get_date_time().to_string();

    custom_events.push((custom_begin_datetime_str, custom_end_datetime_str));

    let options = FileOptions::new().write(true).create(true).append(true);

    // FIXME: error handling
    let mut lock = match FileLock::lock(PATH, true, options) {
        Ok(lock) => lock,
        Err(err) => panic!("Error getting file lock: {}", err),
    };

    // FIXME: error handling
    lock.file.set_len(0)?;

    // FIXME: error handling
    let json = serde_json::to_string(&custom_events)?;
    lock.file.write_all(json.as_bytes())?;

    println!("Done.");
    Ok(())
}

pub fn remove() -> Result<(), Box<dyn Error>> {
    let mut custom_events = get()?;

    if custom_events.is_empty() {
        println!("No custom events found!");
        return Ok(());
    }

    let mut i = 0;
    for (begin, end) in custom_events.iter() {
        println!("{}: {} - {}", i, begin, end);
        i += 1;
    }
    custom_events.remove(stdin_read_int("Which event would you like to remove", 0));

    let options = FileOptions::new().write(true).create(true).append(true);

    // FIXME: error handling
    let mut lock = match FileLock::lock(PATH, true, options) {
        Ok(lock) => lock,
        Err(err) => panic!("Error getting file lock: {}", err),
    };

    // FIXME: error handling
    lock.file.set_len(0)?;

    // FIXME: error handling
    let json = serde_json::to_string(&custom_events)?;
    lock.file.write_all(json.as_bytes())?;

    println!("Done.");
    Ok(())
}
