use clrust::{App, AppIdentity, AppVersion, Arg, ArgEmptyValidator, ParseError};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new(
        AppIdentity::new(
            "CSV Reader",
            "Prints rows from a CSV file with optional header highlighting.",
            AppVersion::new(0, 1, 0),
        )
        .author("Jonathan Willianto"),
    );

    app.add_argument(
        "--csv",
        Arg::new()
            .help("Path to the CSV file to read")
            .validate(ArgEmptyValidator::require_value())
            .required(),
    );
    app.add_argument(
        "--headers",
        Arg::new()
            .help("Treat the first row as headers")
            .validate(ArgEmptyValidator::allow())
            .optional(),
    );

    if let Err(err) = app.parse_args(true, true) {
        return Err(Box::new(err));
    }

    let parsed = app.args();
    let csv_path = parsed
        .first_of("--csv")
        .cloned()
        .ok_or_else(|| ParseError::invalid_value("--csv is required"))?;
    let csv_path = PathBuf::from(csv_path);
    let show_headers = parsed.contains("--headers");

    let file = File::open(&csv_path)
        .map_err(|err| format!("failed to open {}: {err}", csv_path.display()))?;
    let reader = BufReader::new(file);

    for (line_idx, line) in reader.lines().enumerate() {
        let line = line?;
        if line_idx == 0 && show_headers {
            println!("== {line} ==");
        } else {
            let row = line
                .split(',')
                .map(|col| col.trim())
                .collect::<Vec<_>>()
                .join(" | ");
            println!("{row}");
        }
    }

    Ok(())
}
