use clrust::{App, AppIdentity, AppVersion, Arg, ArgEmptyValidator};

fn main() {
    let identity = AppIdentity::new(
        "Hello World",
        "Greets the provided name or defaults to world.",
        AppVersion::new(0, 1, 0),
    );
    let mut app = App::new(identity);
    app.add_argument(
        "--name",
        Arg::new()
            .help("Name to greet")
            .validate(ArgEmptyValidator::require_value())
            .optional(),
    );

    app.parse_args(true);

    let greeting = app
        .args()
        .first_of("--name")
        .map(|name| format!("Hello, {name}!"))
        .unwrap_or_else(|| "Hello, world!".into());

    println!("{greeting}");
}
