# ea-appointment-reminders

![lang](https://img.shields.io/badge/lang-rust-orange)
![licensing](https://img.shields.io/badge/license-MIT_or_Apache_2.0-blue)
![CI](https://github.com/Celeo/ea-appointment-reminders/actions/workflows/ci.yml/badge.svg)

Appointment reminders for [Easy!Appointments](https://easyappointments.org/).

This project is not official nor affiliated with Easy!Appointments.

## Building

### Requirements

- Git
- A recent version of [Rust](https://www.rust-lang.org/tools/install)

### Steps

```sh
git clone https://github.com/Celeo/ea-appointment-reminders
cd ea-appointment-reminders
cargo build
```

## Running

From the project root, you can run `cargo run` to start the app.

You must supply a "reminders_config.toml" file with the app's configuration. A sample file can be found in this repo at [reminders_config.example.toml](./reminders_config.example.toml).

Every 1 hour, the program will make an API call to your Easy!Appointments API, checking for appointments that are within 3 days from the current time. For each of those appointments, an email reminder will be sent to the appointment creator. A simple "reminders.txt" file is maintained so that no duplicate reminders are sent.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
* MIT license ([LICENSE-MIT](LICENSE-MIT))

## Contributing

Please feel free to contribute. Please open an issue first (or comment on an existing one) so that I know that you want to add/change something.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
