use clrust::{App, AppIdentity, AppVersion, Arg, ArgEmptyValidator};

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
            .help("Value to echo back to stdout")
            .validate(ArgEmptyValidator::require_value())
            .required(),
    );

    if app.parse_args(true, true).is_err() {
        return;
    }

    let value = app
        .args()
        .first_of("--echo")
        .expect("--echo is required; parser enforces this");
    println!("{value}");
}
