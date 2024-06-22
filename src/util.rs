use chrono::{Datelike, Timelike};

pub fn stdin_read_str(prompt: &str, default: &str) -> String {
    let stdin = std::io::stdin();
    let mut buffer = String::new();
    loop {
        print!("{prompt}");
        if !default.is_empty() {
            print!(" ({default}):");
        }
        println!();
        if let Err(error) = stdin.read_line(&mut buffer) {
            println!("Something went wrong: {error}. Please try again.");
            buffer.clear();
            continue;
        }
        buffer = buffer.trim().to_string();
        if buffer.is_empty() {
            return default.to_string();
        }
        return buffer;
    }
}

pub fn stdin_read_int<T>(prompt: &str, default: T) -> T
where
    T: std::str::FromStr + std::fmt::Display,
{
    let stdin = std::io::stdin();
    let mut buffer = String::new();
    loop {
        println!("{} ({}):", prompt, default);
        buffer.clear();

        if let Err(error) = stdin.read_line(&mut buffer) {
            println!("Something went wrong: {error}. Please try again.");
            continue;
        }

        if let Ok(value) = buffer.trim().parse::<T>() {
            return value;
        } else if buffer.trim().is_empty() {
            return default;
        }

        println!("That value is invalid, please try again.");
    }
}

pub fn stdin_get_date_time() -> chrono::NaiveDateTime {
    let now = chrono::Local::now();

    let date = loop {
        if let Some(date) = chrono::NaiveDate::from_ymd_opt(
            stdin_read_int(&"Year", now.year()),
            stdin_read_int(&"Month", now.month()),
            stdin_read_int(&"Day", now.day()),
        ) {
            break date;
        }
        println!("That date is invalid, please try again.");
    };

    let time = loop {
        if let Some(time) = chrono::NaiveTime::from_hms_opt(
            stdin_read_int(&"Hour", now.hour()),
            stdin_read_int(&"Minutes", 0),
            0,
        ) {
            break time;
        }
        println!("That time is invalid, please try again.");
    };

    chrono::NaiveDateTime::new(date, time)
}
