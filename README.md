# MijnLoon

Program to scrape schedules from JouwLoon and export it to an ICS file.

We also have custom events, i.e. a way to keep track of extra shifts. Use the manager (`cargo run -p manager`) for more info.

See the example config (`config/username.toml.example`) for how to configure a JouwLoon account for scraping. Rename the example or create a new `.toml` file, then run the server (`cargo run -p server`) to generate the `.ics` file(s) in the `ics` directory.
