use crate::util::{stdin_get_date_time, stdin_read_int_ranged};
use file_lock::{FileLock, FileOptions};
use std::error::Error;
use std::io::prelude::*;

pub static PATH: &str = "custom_events.json";

pub fn get() -> Result<Vec<(String, String)>, Box<dyn Error>> {
    println!("Getting custom events.");

    let options = FileOptions::new().read(true);

    let mut lock = FileLock::lock(PATH, true, options)?;

    let mut custom_events_str = String::new();
    lock.file.read_to_string(&mut custom_events_str)?;

    let custom_events: Vec<(String, String)> = serde_json::from_str(&custom_events_str)?;

    println!("Done.");
    Ok(custom_events)
}

fn write(custom_events: &Vec<(String, String)>) -> Result<(), Box<dyn Error>> {
    let options = FileOptions::new().write(true).create(true).append(true);
    let mut lock = FileLock::lock(PATH, true, options)?;

    lock.file.set_len(0)?;

    let json = serde_json::to_string(&custom_events)?;
    lock.file.write_all(json.as_bytes())?;

    Ok(())
}

pub fn new() {
    println!("Adding new event.");

    let mut custom_events = match get() {
        Ok(custom_events) => custom_events,
        Err(error) => {
            println!("Failed to get custom events: {error}. Please try again.");
            return;
        }
    };

    println!("Please enter the date and time for the start of the event:");
    let custom_begin_datetime_str = stdin_get_date_time().to_string();

    println!("Please enter the date and time for the end of the event:");
    let custom_end_datetime_str = stdin_get_date_time().to_string();

    custom_events.push((custom_begin_datetime_str, custom_end_datetime_str));

    if let Err(error) = write(&custom_events) {
        println!("Failed to write custom events to file: {error}. Please try again");
        return;
    }

    println!("Done.");
}

pub fn remove() {
    let mut custom_events = match get() {
        Ok(custom_events) => custom_events,
        Err(error) => {
            println!("Failed to get custom events: {error}. Please try again.");
            return;
        }
    };

    if custom_events.is_empty() {
        println!("No custom events found!");
        return;
    }

    custom_events.sort_unstable();

    let mut i = 0;
    for (begin, end) in custom_events.iter() {
        println!("{}: {} - {}", i, begin, end);
        i += 1;
    }
    custom_events.remove(stdin_read_int_ranged(
        "Which event would you like to remove",
        0..=i - 1,
        0,
    ));

    if let Err(error) = write(&custom_events) {
        println!("Failed to write custom events to file: {error}. Please try again");
        return;
    }

    println!("Done.");
}
