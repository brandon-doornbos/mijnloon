mod custom_event;
mod schedule;
mod util;

fn main() {
    let username = util::stdin_read_str("Please enter your JouwLoon username:", "");
    let password = loop {
        match rpassword::prompt_password("Password: ") {
            Ok(password) => break password,
            Err(error) => println!("Failed to get password: {error}. Please try again."),
        }
    };
    let summary = util::stdin_read_str("Calendar event title", "Werken");
    let filename = util::stdin_read_str("Filename to save", "schedule.ics");

    std::thread::spawn(move || loop {
        if let Err(error) = schedule::write(&username, &password, &summary, &filename) {
            println!("Failed to write schedule: {error}. Trying again later.");
        }

        println!("Waiting 1 hour...");
        std::thread::sleep(std::time::Duration::from_secs(3600));
    });

    let stdin = std::io::stdin();
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
