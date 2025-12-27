use clark::{
    App, AppIdentity, AppVersion, Arg, ArgOptionValidator,
    log::{
        self, BwFormatter, ColorfulFormatter, Context, Emitter, Error, FileEmitter, Formatter,
        Logger, StderrEmitter, StdoutEmitter, ThreadedEmitter,
    },
};
use std::{
    fmt,
    time::{Duration, Instant, SystemTime},
};

#[derive(Default)]
struct EmptyEmitter;

impl Emitter for EmptyEmitter {
    fn emit(&self, _: String) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Clone, Copy, Default)]
struct PlainFormatter;

impl Formatter for PlainFormatter {
    fn fmt(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let mut buf = String::new();
        fmt::write(&mut buf, ctx.message)
            .map_err(|_| Error::format_error(format_args!("format error")))?;
        buf.push('\n');
        Ok(buf)
    }
}

#[derive(Clone, Copy, Default)]
struct EmptyFormatter;

impl Formatter for EmptyFormatter {
    fn fmt(&self, _: &Context<'_>) -> Result<String, Error> {
        Ok(String::new())
    }
}

fn invoke_bench<T>(mut f: impl FnMut() -> T) -> (T, Duration) {
    let begin = Instant::now();
    let res = f();
    let elapsed = begin.elapsed();
    (res, elapsed)
}

fn random_string(len: usize) -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
        ^ len as u64;

    let mut next = || {
        seed ^= seed >> 12;
        seed ^= seed << 25;
        seed ^= seed >> 27;
        seed = seed.wrapping_mul(0x2545F4914F6CDD1D);
        seed
    };

    let mut out = String::with_capacity(len);
    for _ in 0..len {
        let idx = (next() as usize) % ALPHABET.len();
        out.push(ALPHABET[idx] as char);
    }
    out
}

fn parse_or_default<T>(name: &str, raw: Option<&String>, default: T) -> T
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: fmt::Display,
{
    match raw {
        Some(value) => match value.parse::<T>() {
            Ok(parsed) => parsed,
            Err(err) => {
                eprintln!("Invalid {name} '{}': {err}", value);
                std::process::exit(1);
            }
        },
        None => default,
    }
}

fn create_logger(formatter: &str, emitter: &str, threaded: bool) -> Logger {
    let logger = Logger::default();
    let logger = match formatter {
        "bw" => logger.set_formatter(BwFormatter),
        "plain" => logger.set_formatter(PlainFormatter),
        "empty" => logger.set_formatter(EmptyFormatter),
        _ => logger.set_formatter(ColorfulFormatter),
    };
    match (threaded, emitter) {
        (false, "stderr") => logger.set_emitter(StderrEmitter),
        (false, "empty") => logger.set_emitter(EmptyEmitter),
        (false, "file") => logger.set_emitter(FileEmitter::open("example.log").unwrap()),
        (false, _) => logger.set_emitter(StdoutEmitter),
        (true, "stderr") => logger.set_emitter(ThreadedEmitter::new(StderrEmitter)),
        (true, "empty") => logger.set_emitter(ThreadedEmitter::new(EmptyEmitter)),
        (true, "file") => logger.set_emitter(ThreadedEmitter::new(
            FileEmitter::open("example.log").unwrap(),
        )),
        (true, _) => logger.set_emitter(ThreadedEmitter::new(StdoutEmitter)),
    }
}

fn log_messages(logger: &Logger, msg: &str, count: u64) -> u64 {
    for i in 0..count {
        log::info_with!(logger, "{i} - {msg}");
    }
    count
}

fn main() {
    let identity = AppIdentity::new(
        "Crogger Benchmarker",
        "Benchmark logger formatting and emission throughput.",
        AppVersion::new(1, 0, 0),
    );
    let mut app = App::new(identity);

    app.add_argument(
        "--count",
        Arg::new()
            .help("Number of log messages to produce")
            .require_value()
            .optional(),
    );
    app.add_argument(
        "--msg_length",
        Arg::new()
            .help("Length of each generated log message")
            .require_value()
            .optional(),
    );
    app.add_argument(
        "--emit",
        Arg::new()
            .help("Emitter to use for output")
            .validate(
                ArgOptionValidator::new()
                    .option("stdout", Some("emit logs to stdout (default)".to_string()))
                    .option("stderr", Some("emit logs to stderr".to_string()))
                    .option("file", Some("emit the logs to example.log".to_string()))
                    .option("empty", Some("discard all emitted output".to_string())),
            )
            .optional(),
    );
    app.add_argument("--threaded", Arg::new().as_flag());
    app.add_argument(
        "--format",
        Arg::new()
            .help("Formatter to use for each log message")
            .validate(
                ArgOptionValidator::new()
                    .option(
                        "color",
                        Some("colorful formatting with metadata (default)".to_string()),
                    )
                    .option(
                        "bw",
                        Some("black and white formatting with metadata".to_string()),
                    )
                    .option("plain", Some("message only".to_string()))
                    .option("empty", Some("no formatting content".to_string())),
            )
            .optional(),
    );
    app.add_help_arguments();
    app.parse_args(true);

    let args = app.args();
    let count = parse_or_default("count", args.first_of("--count"), 1_000_000u64);
    let msg_length = parse_or_default("msg_length", args.first_of("--msg_length"), 80u64);
    let formatter = args
        .first_of("--format")
        .cloned()
        .unwrap_or_else(|| "color".to_string());
    let emitter = args
        .first_of("--emit")
        .cloned()
        .unwrap_or_else(|| "stdout".to_string());

    let message = random_string(msg_length as usize);

    log::warn!("Begin: Logger Init");
    let (logger, init_time) =
        invoke_bench(|| create_logger(&formatter, &emitter, app.args().contains("--threaded")));
    log::warn!("End: Logger Init ({} ms)", init_time.as_millis());

    log::warn!("Begin: Log Message");
    let (_, log_time) = invoke_bench(|| log_messages(&logger, &message, count));
    log::warn!("End: Log Message ({} ms)", log_time.as_millis());

    std::thread::sleep(Duration::from_secs(1));
}
