use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use icalendar::{Calendar, Component, Event, EventLike};
use once_cell::sync::Lazy;
use regex::Regex;
use std::error::Error;
use std::io::prelude::*;

mod custom_event;
mod util;

fn main() {
    let stdin = std::io::stdin();

    let mut username = String::new();
    loop {
        println!("Please enter your JouwLoon username:");
        match stdin.read_line(&mut username) {
            Ok(_) => break,
            Err(error) => println!("That didn't work: {error}, try again please."),
        }
        username.clear();
    }

    let password = rpassword::prompt_password("Password: ").unwrap();

    let mut summary = String::new();
    loop {
        println!("Calendar event title (enter for default, \"Werken\"):");
        match stdin.read_line(&mut summary) {
            Ok(_) => break,
            Err(error) => println!("That didn't work: {error}, try again please."),
        }
        summary.clear();
    }
    summary = summary.trim().to_owned();
    if summary.is_empty() {
        summary += "Werken";
    }

    let mut filename = String::new();
    loop {
        println!("Filename to save (enter for default, \"schedule.ics\"):");
        match stdin.read_line(&mut filename) {
            Ok(_) => break,
            Err(error) => println!("That didn't work: {error}, try again please."),
        }
        filename.clear();
    }
    filename = filename.trim().to_owned();
    if filename.is_empty() {
        filename += "schedule.ics";
    }

    let mut first = true;
    std::thread::spawn(move || loop {
        if !first {
            println!("Waiting 1 hour...");
            std::thread::sleep(std::time::Duration::from_secs(3600));
        } else {
            first = false;
        }

        let document_string = get_document_string(&username, &password);
        if let Err(error) = document_string {
            println!("Failed to reach JouwLoon: {error}. Trying again later.");
            continue;
        }

        let calendar = make_schedule(&document_string.unwrap(), &summary);
        if let Err(error) = calendar {
            println!("Failed to make schedule: {error}. Trying again later.");
            continue;
        }

        println!("Saving schedule...");
        let file = std::fs::File::create(&filename);
        if let Err(error) = calendar {
            println!("Failed to create schedule file: {error}. Trying again later.");
            continue;
        }

        if let Err(error) = write!(file.unwrap(), "{}", calendar.unwrap()) {
            println!("Failed to write to schedule file: {error}. Trying again later.");
            continue;
        }
        println!("Done.");
    });

    loop {
        let mut command = String::new();
        stdin.read_line(&mut command).ok();

        match command.trim() {
            "n" => custom_event::new(),
            "r" => custom_event::remove(),
            _ => {
                println!("Unknown command, use 'n' to manually add an event or 'r' to remove one.");
                continue;
            }
        }
    }
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

fn make_schedule(document_string: &str, summary: &str) -> Result<Calendar, Box<dyn Error>> {
    let mut calendar = Calendar::new();
    let timezone = iana_time_zone::get_timezone()?;
    calendar.timezone(&timezone);

    println!("Parsing schedule...");
    let document = scraper::Html::parse_document(&document_string);
    let work_selector = scraper::Selector::parse("#cwerken")?;
    let work_days = document.select(&work_selector);
    for element in work_days {
        let element_str = element.html();
        match parse_work_day(&element_str, &summary) {
            Ok(event) => {
                calendar.push(event);
            }
            Err(error) => println!("Failed to parse work day: {}. Skipping.", error),
        }
    }
    println!("Done.");

    for (begin_datetime_str, end_datetime_str) in custom_event::get()? {
        let begin_datetime =
            NaiveDateTime::parse_from_str(&begin_datetime_str, "%Y-%m-%d %H:%M:%S")?;
        let end_datetime = NaiveDateTime::parse_from_str(&end_datetime_str, "%Y-%m-%d %H:%M:%S")?;

        let event = Event::new()
            .summary(summary)
            .starts(begin_datetime)
            .ends(end_datetime)
            .done();
        calendar.push(event);
    }

    Ok(calendar)
}

fn parse_work_day(element_str: &str, summary: &str) -> Result<Event, Box<dyn Error>> {
    static DATE_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"detail\((\d*),(\d*),(\d*)\);").unwrap());
    let (_, [year, month, day]) = DATE_REGEX
        .captures(element_str)
        .map(|caps| caps.extract())
        .ok_or("Unable to find date.")?;

    static TIME_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(\d*):(\d*)-<br>(\d*):(\d*)").unwrap());
    let (_, [begin_hours, begin_minutes, end_hours, end_minutes]) = TIME_REGEX
        .captures(element_str)
        .map(|caps| caps.extract())
        .ok_or("Unable to find times.")?;

    let mut date = NaiveDate::from_ymd_opt(
        year.parse::<i32>()?,
        month.parse::<u32>()?,
        day.parse::<u32>()?,
    )
    .ok_or("Invalid date.")?;

    let begin_time = NaiveTime::from_hms_opt(
        begin_hours.parse::<u32>()?,
        begin_minutes.parse::<u32>()?,
        0,
    )
    .ok_or("Invalid begin time.")?;
    let begin_datetime = NaiveDateTime::new(date, begin_time);

    let end_time =
        NaiveTime::from_hms_opt(end_hours.parse::<u32>()?, end_minutes.parse::<u32>()?, 0)
            .ok_or("Invalid end time.")?;
    if end_time < begin_time {
        date += chrono::TimeDelta::days(1);
    }
    let end_datetime = NaiveDateTime::new(date, end_time);

    Ok(Event::new()
        .summary(summary)
        .starts(begin_datetime)
        .ends(end_datetime)
        .done())
}
