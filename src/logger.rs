use chrono::{Datelike, Timelike};

use crate::tui::{DomStyle, Layout, Paragraph, RgbColor};
use std::{
    error::Error,
    fmt::{self, Write},
    sync::OnceLock,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogErrorType {
    FormatError,
    IoError,
}

impl fmt::Display for LogErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FormatError => write!(f, "FORMAT_ERROR"),
            Self::IoError => write!(f, "IO_ERROR"),
        }
    }
}

#[derive(Debug)]
pub struct LogError {
    pub kind: LogErrorType,
    msg: String,
}

impl LogError {
    pub fn new(kind: LogErrorType, args: fmt::Arguments<'_>) -> Self {
        let mut msg = String::new();
        // Prefix with the error type name to mirror the C++ implementation.
        let _ = fmt::write(&mut msg, format_args!("{kind}: "));
        let _ = fmt::write(&mut msg, args);
        Self { kind, msg }
    }

    pub fn format_error(args: fmt::Arguments<'_>) -> Self {
        Self::new(LogErrorType::FormatError, args)
    }

    pub fn io_error(args: fmt::Arguments<'_>) -> Self {
        Self::new(LogErrorType::IoError, args)
    }
}

impl fmt::Display for LogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for LogError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogLevel {
    name: &'static str,
    level: u8,
}

impl LogLevel {
    pub fn trace() -> Self {
        Self {
            name: "TRACE",
            level: 0,
        }
    }

    pub fn debug() -> Self {
        Self {
            name: "DEBUG",
            level: 10,
        }
    }

    pub fn info() -> Self {
        Self {
            name: "INFO",
            level: 20,
        }
    }

    pub fn warn() -> Self {
        Self {
            name: "WARN",
            level: 30,
        }
    }

    pub fn error() -> Self {
        Self {
            name: "ERROR",
            level: 40,
        }
    }

    pub fn critical() -> Self {
        Self {
            name: "CRITICAL",
            level: 50,
        }
    }
}

impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LogLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.level.cmp(&other.level)
    }
}

pub struct LogContext<'a> {
    pub status: LogLevel,
    pub location: &'static std::panic::Location<'static>,
    pub time: chrono::DateTime<chrono::Utc>,
    pub message: fmt::Arguments<'a>,
}

pub trait LogEmitter: Send + Sync {
    fn emit(&self, v: &str) -> Result<(), LogError>;
}

pub trait LogFormatter: Send + Sync {
    fn fmt(&self, ctx: &LogContext<'_>) -> Result<String, LogError>;
}

pub trait LogFilter: Send + Sync {
    fn allow(&self, ctx: &LogContext<'_>) -> bool;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NoFilter;

impl LogFilter for NoFilter {
    fn allow(&self, _: &LogContext<'_>) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOp {
    Eq,
    Lt,
    Lte,
    Gt,
    Gte,
}

#[derive(Debug, Clone, Copy)]
pub struct LevelFilter {
    op: FilterOp,
    level: u8,
}

impl LevelFilter {
    pub fn equal_to(level: u8) -> Self {
        Self {
            op: FilterOp::Eq,
            level,
        }
    }

    pub fn less_than(level: u8) -> Self {
        Self {
            op: FilterOp::Lt,
            level,
        }
    }

    pub fn less_than_or_equal_to(level: u8) -> Self {
        Self {
            op: FilterOp::Lte,
            level,
        }
    }

    pub fn greater_than(level: u8) -> Self {
        Self {
            op: FilterOp::Gt,
            level,
        }
    }

    pub fn greater_than_or_equal_to(level: u8) -> Self {
        Self {
            op: FilterOp::Gte,
            level,
        }
    }
}

impl LogFilter for LevelFilter {
    fn allow(&self, ctx: &LogContext<'_>) -> bool {
        match self.op {
            FilterOp::Eq => ctx.status.level == self.level,
            FilterOp::Lt => ctx.status.level < self.level,
            FilterOp::Lte => ctx.status.level <= self.level,
            FilterOp::Gt => ctx.status.level > self.level,
            FilterOp::Gte => ctx.status.level >= self.level,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ColorfulFormatter;

impl ColorfulFormatter {
    fn level_color(&self, level: u8) -> RgbColor {
        match level {
            0..10 => RgbColor::cyan(),
            10..20 => RgbColor::blue(),
            20..30 => RgbColor::green(),
            30..40 => RgbColor::yellow(),
            40..50 => RgbColor::magenta(),
            _ => RgbColor::red(),
        }
    }
}

impl LogFormatter for ColorfulFormatter {
    fn fmt(&self, ctx: &LogContext<'_>) -> Result<String, LogError> {
        let mut buf = String::new();
        writeln!(
            buf,
            "{} {}-{:0>2}-{:0>2}T{:0>2}:{:0>2}:{:0>2}Z {}",
            Layout::new()
                .style(DomStyle::new().fg(self.level_color(ctx.status.level)))
                .append_child(Paragraph::new(format_args!("[{}]", ctx.status.name)).no_newline()),
            ctx.time.year(),
            ctx.time.month(),
            ctx.time.day(),
            ctx.time.hour(),
            ctx.time.minute(),
            ctx.time.second(),
            ctx.message
        )
        .map_err(|_| LogError::format_error(format_args!("format error")))?;
        Ok(buf)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BwFormatter;

impl LogFormatter for BwFormatter {
    fn fmt<'a>(&'a self, ctx: &LogContext<'a>) -> Result<String, LogError> {
        let mut buf = String::new();
        writeln!(
            buf,
            "[{}] {}-{:0>2}-{:0>2}T{:0>2}:{:0>2}:{:0>2}Z {}",
            ctx.status.name,
            ctx.time.year(),
            ctx.time.month(),
            ctx.time.day(),
            ctx.time.hour(),
            ctx.time.minute(),
            ctx.time.second(),
            ctx.message
        )
        .map_err(|_| LogError::format_error(format_args!("format error")))?;
        Ok(buf)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PlainFormatter;

impl LogFormatter for PlainFormatter {
    fn fmt(&self, ctx: &LogContext<'_>) -> Result<String, LogError> {
        let mut buf = String::new();
        writeln!(buf, "{}", ctx.message)
            .map_err(|_| LogError::format_error(format_args!("format error")))?;
        Ok(buf)
    }
}

#[derive(Default)]
pub struct StdoutEmitter;
impl LogEmitter for StdoutEmitter {
    fn emit(&self, v: &str) -> Result<(), LogError> {
        print!("{}", v);
        Ok(())
    }
}

#[derive(Default)]
pub struct EmptyEmitter;
impl LogEmitter for EmptyEmitter {
    fn emit(&self, _: &str) -> Result<(), LogError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct StderrEmitter;
impl LogEmitter for StderrEmitter {
    fn emit(&self, v: &str) -> Result<(), LogError> {
        eprint!("{}", v);
        Ok(())
    }
}

pub struct Logger {
    filter: Box<dyn LogFilter>,
    formatter: Box<dyn LogFormatter>,
    emitter: Box<dyn LogEmitter>,
}

impl Logger {
    pub fn set_filter(mut self, filter: impl LogFilter + 'static) -> Self {
        self.filter = Box::new(filter);
        self
    }
    pub fn set_formatter(mut self, formatter: impl LogFormatter + 'static) -> Self {
        self.formatter = Box::new(formatter);
        self
    }
    pub fn set_emitter(mut self, emitter: impl LogEmitter + 'static) -> Self {
        self.emitter = Box::new(emitter);
        self
    }
    pub fn log(&self, ctx: LogContext) {
        if self.filter.allow(&ctx) {
            self.formatter
                .fmt(&ctx)
                .and_then(|msg| self.emitter.emit(&msg))
                .or_else(|e| StdoutEmitter.emit(&format!("{}", e)))
                .unwrap()
        }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            filter: Box::new(NoFilter),
            formatter: Box::new(ColorfulFormatter),
            emitter: Box::new(StdoutEmitter),
        }
    }
}

static ROOT_LOGGER: OnceLock<Logger> = OnceLock::new();

pub fn init_logger(logger: Logger) -> Result<(), Logger> {
    ROOT_LOGGER.set(logger)
}

pub fn root() -> &'static Logger {
    ROOT_LOGGER.get_or_init(Logger::default)
}

#[track_caller]
pub fn log_with(logger: &Logger, status: LogLevel, message: fmt::Arguments<'_>) {
    logger.log(LogContext {
        status,
        location: std::panic::Location::caller(),
        time: chrono::Utc::now(),
        message,
    });
}

#[track_caller]
pub fn log(status: LogLevel, message: fmt::Arguments<'_>) {
    log_with(root(), status, message);
}

#[track_caller]
pub fn trace_with(logger: &Logger, message: fmt::Arguments<'_>) {
    log_with(logger, LogLevel::trace(), message);
}

#[track_caller]
pub fn debug_with(logger: &Logger, message: fmt::Arguments<'_>) {
    log_with(logger, LogLevel::debug(), message);
}

#[track_caller]
pub fn info_with(logger: &Logger, message: fmt::Arguments<'_>) {
    log_with(logger, LogLevel::info(), message);
}

#[track_caller]
pub fn warn_with(logger: &Logger, message: fmt::Arguments<'_>) {
    log_with(logger, LogLevel::warn(), message);
}

#[track_caller]
pub fn error_with(logger: &Logger, message: fmt::Arguments<'_>) {
    log_with(logger, LogLevel::error(), message);
}

#[track_caller]
pub fn critical_with(logger: &Logger, message: fmt::Arguments<'_>) {
    log_with(logger, LogLevel::critical(), message);
}

#[track_caller]
pub fn trace(message: fmt::Arguments<'_>) {
    log(LogLevel::trace(), message);
}

#[track_caller]
pub fn debug(message: fmt::Arguments<'_>) {
    log(LogLevel::debug(), message);
}

#[track_caller]
pub fn info(message: fmt::Arguments<'_>) {
    log(LogLevel::info(), message);
}

#[track_caller]
pub fn warn(message: fmt::Arguments<'_>) {
    log(LogLevel::warn(), message);
}

#[track_caller]
pub fn error(message: fmt::Arguments<'_>) {
    log(LogLevel::error(), message);
}

#[track_caller]
pub fn critical(message: fmt::Arguments<'_>) {
    log(LogLevel::critical(), message);
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {{
        $crate::logger::trace(format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! trace_with {
    ($logger:expr, $($arg:tt)*) => {{
        $crate::logger::trace_with($logger, format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        $crate::logger::debug(format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! debug_with {
    ($logger:expr, $($arg:tt)*) => {{
        $crate::logger::debug_with($logger, format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        $crate::logger::info(format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! info_with {
    ($logger:expr, $($arg:tt)*) => {{
        $crate::logger::info_with($logger, format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        $crate::logger::warn(format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! warn_with {
    ($logger:expr, $($arg:tt)*) => {{
        $crate::logger::warn_with($logger, format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        $crate::logger::error(format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! error_with {
    ($logger:expr, $($arg:tt)*) => {{
        $crate::logger::error_with($logger, format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! critical {
    ($($arg:tt)*) => {{
        $crate::logger::critical(format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! critical_with {
    ($logger:expr, $($arg:tt)*) => {{
        $crate::logger::critical_with($logger, format_args!($($arg)*))
    }};
}

pub use crate::{
    critical, critical_with, debug, debug_with, error, error_with, info, info_with, trace,
    trace_with, warn, warn_with,
};
