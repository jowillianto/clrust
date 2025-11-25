use clark::{App, AppIdentity, AppVersion, Arg};

fn main() {
    let mut app = App::new(
        AppIdentity::new(
            "Echo",
            "Echoes a provided value back to stdout.",
            AppVersion::new(0, 1, 0),
        )
        .author("Jonathan Willianto"),
    );

    app.add_argument(
        "--echo",
        Arg::new()
            .with_default("You did not provide a value")
            .help("Value to echo back to stdout")
            .require_value()
            .required(),
    );

    app.parse_args(true);

    let value = app
        .args()
        .first_of("--echo")
        .expect("--echo is required; parser enforces this");
    println!("{value}");
}
