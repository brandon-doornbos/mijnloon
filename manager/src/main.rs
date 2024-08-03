use common::{custom_event, util};

fn main() {
    let stdin = std::io::stdin();
    loop {
        let mut command = String::new();
        stdin.read_line(&mut command).ok();

        match command.trim() {
            "l" => {
                let path = util::stdin_read_str(
                    "For which configuration would you like to list the custom events?",
                    "",
                );
                println!("{:#?}", custom_event::get(&path).unwrap_or_default())
            }
            "n" => custom_event::new(),
            "r" => custom_event::remove(),
            "p" => {
                let path = util::stdin_read_str(
                    "For which configuration would you like to purge old custom events?",
                    "",
                );
                custom_event::purge(&path).unwrap_or_default();
            }
            _ => {
                println!("Unknown command, use:");
                println!("  'l' to list custom events");
                println!("  'n' to add new custom events");
                println!("  'r' to remove custom events");
                println!("  'p' to purge old custom events");
                continue;
            }
        }
    }
}
