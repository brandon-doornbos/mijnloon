use std::error::Error;
use std::io::prelude::*;
use std::sync::{Arc, LazyLock};

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use common::{custom_event, Config};
use icalendar::{Calendar, Component, Event, EventLike};
use regex::Regex;
use reqwest::cookie::Jar;

pub fn write(config: &Config) -> Result<(), Box<dyn Error>> {
    let cookie_jar = Arc::new(Jar::default());
    let document_string = get_document_string(config, cookie_jar.clone())?;
    let calendars = make(&document_string, config, cookie_jar)?;

    println!("{}: Saving schedule...", config.username);
    for (i, calendar) in calendars.iter().enumerate() {
        let mut path = std::path::PathBuf::from("./ics");
        if !path.exists() {
            std::fs::create_dir(&path)?;
        }
        if i == 0 {
            path.push(&config.username);
        } else {
            path.push(config.username.clone() + &i.to_string());
        }
        path.set_extension("ics");
        let mut file = std::fs::File::create(path)?;
        write!(file, "{calendar}")?;
    }
    println!("{}: Done.", config.username);

    Ok(())
}

fn make(
    document_string: &str,
    config: &Config,
    cookie_jar: Arc<Jar>,
) -> Result<Vec<Calendar>, Box<dyn Error>> {
    let mut calendars = vec![];
    let timezone = iana_time_zone::get_timezone()?;
    for _ in &config.summaries {
        let mut calendar = Calendar::new();
        calendar.timezone(&timezone);
        calendars.push(calendar);
    }

    println!("{}: Parsing schedule...", config.username);
    let document = scraper::Html::parse_document(&document_string);
    let work_selector = scraper::Selector::parse("#cwerken")?;
    let work_days = document.select(&work_selector);
    for element in work_days {
        let element_str = element.html();
        let mut event = parse_work_day(&element_str, config, cookie_jar.clone())?;
        for (i, calendar) in calendars.iter_mut().enumerate() {
            event.summary(&config.summaries[i]);
            calendar.push(event.clone());
        }
    }
    println!("{}: Done.", config.username);

    custom_event::purge(&config.username)?;

    for (begin_datetime_str, end_datetime_str) in custom_event::get(&config.username)? {
        let begin_datetime =
            NaiveDateTime::parse_from_str(&begin_datetime_str, "%Y-%m-%d %H:%M:%S")?;
        let end_datetime = NaiveDateTime::parse_from_str(&end_datetime_str, "%Y-%m-%d %H:%M:%S")?;

        let mut event = Event::new()
            .starts(begin_datetime)
            .ends(end_datetime)
            .done();
        for (i, calendar) in calendars.iter_mut().enumerate() {
            event.summary(&config.summaries[i]);
            calendar.push(event.clone());
        }
    }

    Ok(calendars)
}

fn get_document_string(config: &Config, cookie_jar: Arc<Jar>) -> Result<String, Box<dyn Error>> {
    let client = reqwest::blocking::Client::builder()
        .cookie_provider(cookie_jar)
        .build()?;

    println!("{}: Logging in...", config.username);
    let login_form = [
        ("username", &config.username),
        ("password", &config.password),
    ];
    let response = client
        .post("https://jouwloon.nl/login")
        .form(&login_form)
        .send()?;

    if response.url().path() != "/login" {
        println!("{}: Done.", config.username);
    } else {
        return Err(format!(
            "{}: Failed to log in (wrong username or password?)",
            config.username
        )
        .into());
    }

    println!("{}: Getting schedule...", config.username);
    let body = client.get("https://jouwloon.nl/rooster").send()?.text()?;
    println!("{}: Done.", config.username);

    Ok(body)
}

fn get_work_day_description(
    year: &str,
    month: &str,
    day: &str,
    cookie_jar: Arc<Jar>,
) -> Result<String, Box<dyn Error>> {
    let client = reqwest::blocking::ClientBuilder::new()
        .cookie_provider(cookie_jar)
        .build()?;
    let response = client
        .get(format!(
            "https://jouwloon.nl/index.php/rooster/detail/{year}-{month}-{day}"
        ))
        .send()?
        .text()?;

    static DESCRIPTION_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r">\((.+?)\)<").unwrap());
    let (_, [description]) = DESCRIPTION_REGEX
        .captures(&response)
        .map(|caps| caps.extract())
        .ok_or("Unable to find description.")?;

    return Ok(description.to_string());
}

fn parse_work_day(
    element_str: &str,
    config: &Config,
    cookie_jar: Arc<Jar>,
) -> Result<Event, Box<dyn Error>> {
    static DATE_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"detail\((\d*),(\d*),(\d*)\);").unwrap());
    let (_, [year, month, day]) = DATE_REGEX
        .captures(element_str)
        .map(|caps| caps.extract())
        .ok_or("{}: Unable to find date.")?;

    static TIME_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(\d*):(\d*)-<br>(\d*):(\d*)").unwrap());
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

    let description = if config.descriptions {
        match get_work_day_description(year, month, day, cookie_jar) {
            Ok(description) => description,
            Err(error) => {
                println!(
                    "{}: Failed to get description for an event: {error}. Continuing anyway.",
                    config.username
                );
                String::new()
            }
        }
    } else {
        String::new()
    };

    Ok(Event::new()
        .description(&description)
        .starts(begin_datetime)
        .ends(end_datetime)
        .done())
}
