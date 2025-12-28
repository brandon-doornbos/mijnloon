use std::error::Error;
use std::hash::{Hash, Hasher};
use std::io::prelude::*;

use crate::{config, config::Config, custom_event};
use chrono::NaiveDateTime;
use icalendar::{Calendar, Component, Event as IEvent, EventLike};
use reqwest::blocking::Client;

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub start_dt: String,
    pub end_dt: String,
    pub desc: Option<String>,
    pub hash: Option<String>,
}

pub fn write(config: &mut Config) -> Result<(), Box<dyn Error>> {
    cache_events(config)?;
    write_cached(config)?;

    Ok(())
}

pub fn write_cached(config: &Config) -> Result<(), Box<dyn Error>> {
    let calendars = make(config)?;

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

fn cache_events(config: &mut Config) -> Result<(), Box<dyn Error>> {
    println!("{}: Parsing schedule...", config.username);

    let schedule_json = get(config)?;
    config.events.clear();
    for day in json::parse(&schedule_json)?.members_mut() {
        for shift in day["roosterdienst"].members_mut() {
            let desc = shift["afdeling"]
                .take_string()
                .ok_or("This should be a string")?;
            let start_dt = shift["vanafDatum"]
                .take_string()
                .ok_or("This should be a datetime string")?;
            let end_dt = shift["totDatum"]
                .take_string()
                .ok_or("This should be a datetime string")?;

            let mut hasher = std::hash::DefaultHasher::new();
            start_dt.hash(&mut hasher);
            end_dt.hash(&mut hasher);
            let hash = hasher.finish().to_string();

            config.events.push(Event {
                start_dt,
                end_dt,
                desc: Some(desc),
                hash: Some(hash),
            });
        }
    }
    let path = std::path::Path::new("config")
        .join(&config.username)
        .with_extension("toml");

    config::store(&path, (*config).clone()).map_err(|error| {
        format!(
            "Failed to load config: {}, because: {error}. Please try again.",
            config.username
        )
    })?;

    println!("{}: Done.", config.username);

    Ok(())
}

fn make(config: &Config) -> Result<Vec<Calendar>, Box<dyn Error>> {
    let mut calendars = vec![];
    let timezone = iana_time_zone::get_timezone()?;
    for _ in &config.summaries {
        let mut calendar = Calendar::new();
        calendar.timezone(&timezone);
        calendars.push(calendar);
    }

    custom_event::purge(&config.username)?;
    let custom_events = custom_event::get(&config.username)?;

    'outer: for e in &config.events {
        for ce in &custom_events {
            if e.hash == ce.hash {
                continue 'outer;
            }
        }
        let mut ical_event = IEvent::new();
        ical_event.starts(NaiveDateTime::parse_from_str(
            &e.start_dt,
            "%Y-%m-%dT%H:%M:%S",
        )?);
        ical_event.ends(NaiveDateTime::parse_from_str(
            &e.end_dt,
            "%Y-%m-%dT%H:%M:%S",
        )?);
        if let Some(desc) = &e.desc {
            ical_event.description(desc);
        }

        for (i, calendar) in calendars.iter_mut().enumerate() {
            ical_event.summary(&config.summaries[i]);
            calendar.push(ical_event.clone());
        }
    }

    for e in custom_events {
        let start_dt = NaiveDateTime::parse_from_str(&e.start_dt, "%Y-%m-%d %H:%M:%S")?;
        let end_dt = NaiveDateTime::parse_from_str(&e.end_dt, "%Y-%m-%d %H:%M:%S")?;

        let mut event = IEvent::new();
        event.starts(start_dt);
        event.ends(end_dt);
        if let Some(desc) = &e.desc {
            event.description(desc);
        }

        for (i, calendar) in calendars.iter_mut().enumerate() {
            event.summary(&config.summaries[i]);
            calendar.push(event.clone());
        }
    }

    Ok(calendars)
}

fn login(config: &Config, client: &Client) -> Result<(), Box<dyn Error>> {
    println!("{}: Logging in...", config.username);

    let viewstate_html = client
        .get("https://jouwloon.nl/Login.aspx")
        .send()?
        .text()?;
    let viewstate_document = scraper::Html::parse_document(&viewstate_html);
    let viewstate_selector = scraper::Selector::parse("#__VIEWSTATE")?;
    let eventvalidation_selector = scraper::Selector::parse("#__EVENTVALIDATION")?;

    let login_form = [
        (
            "ctl00$ContentPlaceHolder1$input_Gebruikersnaam",
            &config.username,
        ),
        (
            "ctl00$ContentPlaceHolder1$input_Wachtwoord",
            &config.password,
        ),
        (
            "ctl00$ContentPlaceHolder1$Button_Inloggen",
            &String::from("Inloggen"),
        ),
        (
            "__VIEWSTATE",
            &viewstate_document
                .select(&viewstate_selector)
                .next()
                .ok_or("No __VIEWSTATE found!")?
                .value()
                .attr("value")
                .ok_or("__VIEWSTATE contains no value!")?
                .to_owned(),
        ),
        (
            "__EVENTVALIDATION",
            &viewstate_document
                .select(&eventvalidation_selector)
                .next()
                .ok_or("No __EVENTVALIDATION found!")?
                .value()
                .attr("value")
                .ok_or("__EVENTVALIDATION contains no value!")?
                .to_owned(),
        ),
    ];
    let response = client
        .post("https://jouwloon.nl/Login.aspx")
        .form(&login_form)
        .send()?;

    if response.url().path() == "/Dashboard.aspx" {
        println!("{}: Done.", config.username);
        Ok(())
    } else {
        Err(format!(
            "{}: Failed to log in (wrong username or password?)",
            config.username
        )
        .into())
    }
}

fn get(config: &Config) -> Result<String, Box<dyn Error>> {
    let cookie_jar = std::sync::Arc::new(reqwest::cookie::Jar::default());
    let client = Client::builder().cookie_provider(cookie_jar).build()?;

    login(config, &client)?;

    println!("{}: Getting schedule...", config.username);

    let customer_info = client
        .get("https://jouwloon.nl/api/beveiliging/klanten")
        .send()?
        .text()?;

    let schedule_form = [
        ("klantenData", customer_info),
        (
            "start",
            chrono::Local::now()
                .checked_sub_months(chrono::Months::new(12))
                .unwrap()
                .to_rfc3339(),
        ),
        (
            "end",
            chrono::Local::now()
                .checked_add_months(chrono::Months::new(12))
                .unwrap()
                .to_rfc3339(),
        ),
    ];
    let body = client
        .post("https://jouwloon.nl/api/rooster/GetKalender/")
        .form(&schedule_form)
        .send()?
        .text()?;

    println!("{}: Done.", config.username);

    Ok(body)
}
