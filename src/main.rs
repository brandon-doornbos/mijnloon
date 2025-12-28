use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::thread;

use chrono::DateTime;
use rocket::fs::NamedFile;
//use rocket::response::status::NotFound;
use rocket::serde::{json::Json, Deserialize};

mod config;
mod custom_event;
mod schedule;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Config<'r> {
    username: &'r str,
    password: &'r str,
    summaries: Vec<&'r str>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Event<'r> {
    username: &'r str,
    start: &'r str,
    end: &'r str,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct UpdateEvent<'r> {
    username: &'r str,
    orig_start: &'r str,
    orig_end: &'r str,
    start: &'r str,
    end: &'r str,
}

fn make_ics(username: &str) {
    let path = Path::new("config").join(username).with_extension("toml");

    let config = config::load(&path)
        .map_err(|error| {
            format!(
                "Failed to load config: {}, because: {error}. Please try again.",
                username
            )
        })
        .unwrap();

    if let Err(error) = schedule::write_cached(&config) {
        println!(
            "{}: Failed to write schedule: {error}. Trying again later.",
            config.username
        );
    }
}

#[rocket::get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open("public/index.html").await.ok()
}

#[rocket::get("/<file..>")]
async fn file(file: PathBuf) -> Option<NamedFile> {
    match file
        .extension()
        .unwrap_or(OsStr::new(""))
        .to_str()
        .unwrap_or("")
    {
        "ics" => NamedFile::open(Path::new("ics").join(file)).await.ok(),
        "" => None,
        _ => NamedFile::open(Path::new("public").join(file)).await.ok(),
    }
}

#[rocket::post("/login", data = "<data>")]
fn login(data: String) -> Option<()> {
    println!("{}", data);
    let path = Path::new("config").join(&data).with_extension("toml");
    println!("{:?}", path);
    path.exists().then(|| ())
    //custom_event::get(&data)
    //    .map_err(|e| NotFound(e.to_string()))
    //    .map(|v| format!("{:#?}", v))
}

#[rocket::post("/register", data = "<data>")]
fn register(data: Json<Config<'_>>) -> String {
    println!("{}: {}, {:?}", data.username, data.password, data.summaries);
    todo!("Registering a new account");
    //let custom_events = custom_event::get(data.username).unwrap();
    //format!("{:#?}", custom_events)
}

#[rocket::post("/new", data = "<data>")]
fn new(data: Json<Event<'_>>) -> () {
    let start = DateTime::parse_from_rfc3339(data.start).unwrap();
    let end = DateTime::parse_from_rfc3339(data.end).unwrap();
    custom_event::new(&data.username, &start.naive_local(), &end.naive_local());

    make_ics(data.username);
}

#[rocket::post("/update", data = "<data>")]
fn update(data: Json<UpdateEvent<'_>>) -> () {
    let orig_start = DateTime::parse_from_rfc3339(data.orig_start).unwrap();
    let orig_end = DateTime::parse_from_rfc3339(data.orig_end).unwrap();
    let start = DateTime::parse_from_rfc3339(data.start).unwrap();
    let end = DateTime::parse_from_rfc3339(data.end).unwrap();
    custom_event::update(
        &data.username,
        &orig_start.naive_local().to_string(),
        &orig_end.naive_local().to_string(),
        &start.naive_local(),
        &end.naive_local(),
    );

    make_ics(data.username);
}

#[rocket::post("/remove", data = "<data>")]
fn remove(data: Json<Event<'_>>) -> () {
    let start = DateTime::parse_from_rfc3339(data.start).unwrap();
    let end = DateTime::parse_from_rfc3339(data.end).unwrap();
    custom_event::remove(data.username, &start.naive_local(), &end.naive_local());

    make_ics(data.username);
}

#[rocket::launch]
fn rocket() -> _ {
    for entry in std::fs::read_dir("config")
        .expect("directory `config` should exist, have correct permissions and be a folder since that is the default")
    {
        thread::spawn(move || {
            let path = entry
                .expect("file should exist as it was found by `read_dir`")
                .path();
            if !path.to_string_lossy().ends_with(".toml") {
                return;
            }
            let mut config = config::load(&path)
                .map_err(|error| format!("Failed to load config: {}, because: {error}. Please try again.", path.to_string_lossy()))
                .unwrap();

            loop {
                if let Err(error) = schedule::write(&mut config) {
                    println!(
                        "{}: Failed to write schedule: {error}. Trying again later.",
                        config.username
                    );
                }

                println!(
                    "{}: Waiting {} seconds...",
                    config.username, config.frequency
                );
                thread::sleep(std::time::Duration::from_secs(config.frequency));
            }
        });
    }

    rocket::build().mount(
        "/",
        rocket::routes![index, file, login, register, new, update, remove],
    )
}
