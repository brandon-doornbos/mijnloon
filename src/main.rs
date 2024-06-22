use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use icalendar::{Calendar, Component, Event, EventLike};
use std::error::Error;
use std::io::prelude::*;

mod custom_event;
mod util;

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

    std::thread::spawn(move || loop {
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
            "n" => custom_event::new()?,
            "r" => custom_event::remove()?,
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
        let event = parse_work_day(&element_str, &summary);
        calendar.push(event);
    }
    println!("Done.");

    for (begin_datetime_str, end_datetime_str) in custom_event::get()? {
        // FIXME: error handling
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

fn parse_work_day(element_str: &str, summary: &str) -> Event {
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

    let mut date = NaiveDate::from_ymd_opt(
        year.parse::<i32>().unwrap(),
        month.parse::<u32>().unwrap(),
        day.parse::<u32>().unwrap(),
    )
    .unwrap();

    let begin_time = NaiveTime::from_hms_opt(
        begin_hours.parse::<u32>().unwrap(),
        begin_minutes.parse::<u32>().unwrap(),
        0,
    )
    .unwrap();
    let begin_datetime = NaiveDateTime::new(date, begin_time);

    let end_time = NaiveTime::from_hms_opt(
        end_hours.parse::<u32>().unwrap(),
        end_minutes.parse::<u32>().unwrap(),
        0,
    )
    .unwrap();
    if end_time < begin_time {
        date += chrono::TimeDelta::days(1);
    }
    let end_datetime = NaiveDateTime::new(date, end_time);

    Event::new()
        .summary(summary)
        .starts(begin_datetime)
        .ends(end_datetime)
        .done()
}
