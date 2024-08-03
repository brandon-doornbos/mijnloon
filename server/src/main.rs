use std::thread;

use common::load_config;

mod schedule;

fn main() {
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
            let config = load_config(&path)
                .map_err(|error| format!("Failed to load config: {}, because: {error}. Please try again.", path.to_string_lossy()))
                .unwrap();

            loop {
                if let Err(error) = schedule::write(&config) {
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
    thread::park();
}
