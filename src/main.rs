use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};
use icalendar::{Component, EventLike};
use std::error::Error;
use std::io::prelude::*;

static CUSTOM_EVENTS_FILE: &str = "custom_events.json";

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = std::io::stdin();

    println!("Please enter your JouwLoon username:");
    let mut username = String::new();
    stdin.read_line(&mut username)?;

    let password = rpassword::prompt_password("Password: ").unwrap();

    println!("Calendar event title (enter for default, \"Werken\"):");
    let mut summary = String::new();
    stdin.read_line(&mut summary)?;
    summary = summary.trim().to_owned();
    if summary.is_empty() {
        summary += "Werken";
    }

    println!("Filename to save (enter for default, \"schedule.ics\"):");
    let mut filename = String::new();
    stdin.read_line(&mut filename)?;
    filename = filename.trim().to_owned();
    if filename.is_empty() {
        filename += "schedule.ics";
    }

    std::thread::spawn(move || {
        match get_document_string(&username, &password) {
            Ok(document_string) => {
                let calendar = make_schedule(&document_string, &summary).unwrap();

                println!("Saving schedule...");
                match std::fs::File::create(&filename) {
                    Ok(mut output) => {
                        write!(output, "{}", calendar).unwrap();
                        println!("Done.");
                    }
                    Err(error) => {
                        println!("Failed to save schedule: {}. Trying again later.", error)
                    }
                }
            }
            Err(error) => println!("Failed to get schedule: {}, trying again later.", error),
        }

        println!("Waiting 1 hour...");
        std::thread::sleep(std::time::Duration::from_secs(3600));
    });

    loop {
        let mut command = String::new();
        stdin.read_line(&mut command)?;

        match command.trim() {
            "n" => new_custom_event()?,
            "r" => remove_custom_event()?,
            _ => {
                println!("Unknown command, use 'n' to manually add an event or 'r' to remove one.");
                continue;
            }
        }
    }
}

fn remove_custom_event() -> Result<(), Box<dyn Error>> {
    let mut custom_events = get_custom_events()?;

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

    let options = file_lock::FileOptions::new()
        .write(true)
        .create(true)
        .append(true);

    // FIXME: error handling
    let mut filelock = match file_lock::FileLock::lock(CUSTOM_EVENTS_FILE, true, options) {
        Ok(lock) => lock,
        Err(err) => panic!("Error getting file lock: {}", err),
    };

    // FIXME: error handling
    filelock.file.set_len(0)?;

    // FIXME: error handling
    let json = serde_json::to_string(&custom_events)?;
    filelock.file.write_all(json.as_bytes())?;

    println!("Done.");
    Ok(())
}

// TODO: add range to check for valid values
fn stdin_read_int<T: std::str::FromStr + std::fmt::Display>(prompt: &str, default: T) -> T {
    let stdin = std::io::stdin();
    let mut buffer = String::new();
    loop {
        println!("{} ({}):", prompt, default);
        buffer.clear();
        // FIXME: error handling
        stdin.read_line(&mut buffer).unwrap();

        if let Ok(value) = buffer.trim().parse::<T>() {
            return value;
        } else if buffer.trim().is_empty() {
            return default;
        }
    }
}

// TODO: check for valid date
fn stdin_get_date_time() -> NaiveDateTime {
    let now = chrono::Local::now();

    // FIXME: error handling
    let date = NaiveDate::from_ymd_opt(
        stdin_read_int(&"Year", now.year()),
        stdin_read_int(&"Month", now.month()),
        stdin_read_int(&"Day", now.day()),
    )
    .unwrap();
    let time = chrono::NaiveTime::from_hms_opt(
        stdin_read_int(&"Hour", now.hour()),
        stdin_read_int(&"Minutes", 0),
        0,
    )
    .unwrap();
    chrono::NaiveDateTime::new(date, time)
}

fn new_custom_event() -> Result<(), Box<dyn Error>> {
    println!("Adding new event.");

    let mut custom_events = get_custom_events()?;

    println!("Please enter the date and time for the start of the event:");
    let custom_begin_datetime_str = stdin_get_date_time().to_string();
    println!("Please enter the date and time for the end of the event:");
    let custom_end_datetime_str = stdin_get_date_time().to_string();

    custom_events.push((custom_begin_datetime_str, custom_end_datetime_str));

    let options = file_lock::FileOptions::new()
        .write(true)
        .create(true)
        .append(true);

    // FIXME: error handling
    let mut filelock = match file_lock::FileLock::lock(CUSTOM_EVENTS_FILE, true, options) {
        Ok(lock) => lock,
        Err(err) => panic!("Error getting file lock: {}", err),
    };

    // FIXME: error handling
    filelock.file.set_len(0)?;

    // FIXME: error handling
    let json = serde_json::to_string(&custom_events)?;
    filelock.file.write_all(json.as_bytes())?;

    println!("Done.");
    Ok(())
}

fn get_custom_events() -> Result<Vec<(String, String)>, Box<dyn Error>> {
    println!("Getting custom events.");

    let options = file_lock::FileOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .read(true);

    // FIXME: error handling
    let mut filelock = match file_lock::FileLock::lock(CUSTOM_EVENTS_FILE, true, options) {
        Ok(lock) => lock,
        Err(err) => panic!("Error getting file lock: {}", err),
    };

    let mut custom_events_str = String::new();
    // FIXME: error handling
    filelock.file.read_to_string(&mut custom_events_str)?;

    let custom_events: Vec<(String, String)> =
        serde_json::from_str(&custom_events_str).unwrap_or_default();

    println!("Done.");
    Ok(custom_events)
}

fn get_document_string(username: &str, password: &str) -> Result<String, Box<dyn Error>> {
    let client = reqwest::blocking::Client::builder()
        .cookie_store(true)
        .build()?;

    println!("Logging in...");
    let login_form = [("username", username), ("password", password)];
    let response = client
        .post("https://jouwloon.nl/login")
        .form(&login_form)
        .send()?;

    if response.url().path() != "/login" {
        println!("Done.");
    } else {
        return Err("Failed to log in (wrong username or password?)".into());
    }

    println!("Getting schedule...");
    let body = client.get("https://jouwloon.nl/rooster").send()?.text()?;
    println!("Done.");

    Ok(body)
}

fn make_schedule(
    document_string: &str,
    summary: &str,
) -> Result<icalendar::Calendar, Box<dyn Error>> {
    let mut calendar = icalendar::Calendar::new();
    let timezone = iana_time_zone::get_timezone()?;
    calendar.timezone(&timezone);

    println!("Parsing schedule...");
    let document = scraper::Html::parse_document(&document_string);
    let work_selector = scraper::Selector::parse("#cwerken")?;
    let work_days = document.select(&work_selector);
    for element in work_days {
        let element_str = element.html();
        let event = parse_work_day(&element_str, &summary);
        calendar.push(event);
    }
    println!("Done.");

    for (begin_datetime_str, end_datetime_str) in get_custom_events()? {
        // FIXME: error handling
        let begin_datetime =
            chrono::NaiveDateTime::parse_from_str(&begin_datetime_str, "%Y-%m-%d %H:%M:%S")?;
        let end_datetime =
            chrono::NaiveDateTime::parse_from_str(&end_datetime_str, "%Y-%m-%d %H:%M:%S")?;

        let event = icalendar::Event::new()
            .summary(summary)
            .starts(begin_datetime)
            .ends(end_datetime)
            .done();
        calendar.push(event);
    }

    Ok(calendar)
}

fn parse_work_day(element_str: &str, summary: &str) -> icalendar::Event {
    let date_regex = regex::Regex::new(r"detail\((\d*),(\d*),(\d*)\);").unwrap();
    let Some((_, [year, month, day])) = date_regex.captures(element_str).map(|caps| caps.extract())
    else {
        panic!("No date found!");
    };

    let times_regex = regex::Regex::new(r"(\d*):(\d*)-<br>(\d*):(\d*)").unwrap();
    let Some((_, [begin_hours, begin_minutes, end_hours, end_minutes])) =
        times_regex.captures(element_str).map(|caps| caps.extract())
    else {
        panic!("No times found!");
    };

    let mut date = chrono::NaiveDate::from_ymd_opt(
        year.parse::<i32>().unwrap(),
        month.parse::<u32>().unwrap(),
        day.parse::<u32>().unwrap(),
    )
    .unwrap();

    let begin_time = chrono::NaiveTime::from_hms_opt(
        begin_hours.parse::<u32>().unwrap(),
        begin_minutes.parse::<u32>().unwrap(),
        0,
    )
    .unwrap();
    let begin_datetime = chrono::NaiveDateTime::new(date, begin_time);

    let end_time = chrono::NaiveTime::from_hms_opt(
        end_hours.parse::<u32>().unwrap(),
        end_minutes.parse::<u32>().unwrap(),
        0,
    )
    .unwrap();
    if end_time < begin_time {
        date += chrono::TimeDelta::days(1);
    }
    let end_datetime = chrono::NaiveDateTime::new(date, end_time);

    icalendar::Event::new()
        .summary(summary)
        .starts(begin_datetime)
        .ends(end_datetime)
        .done()
}
