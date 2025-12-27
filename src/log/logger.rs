use super::emitters::StdoutEmitter;
use super::filters::NoFilter;
use super::formatters::ColorfulFormatter;
use super::prelude::{Context, Emitter, Filter, Formatter, Level};
use std::fmt;

pub struct Logger {
    filter: Box<dyn Filter>,
    formatter: Box<dyn Formatter>,
    emitter: Box<dyn Emitter>,
}

impl Logger {
    pub fn set_filter(mut self, filter: impl Filter + 'static) -> Self {
        self.filter = Box::new(filter);
        self
    }
    pub fn set_formatter(mut self, formatter: impl Formatter + 'static) -> Self {
        self.formatter = Box::new(formatter);
        self
    }
    pub fn set_emitter(mut self, emitter: impl Emitter + 'static) -> Self {
        self.emitter = Box::new(emitter);
        self
    }
    pub fn log(&self, ctx: Context<'_>) {
        if self.filter.allow(&ctx) {
            self.formatter
                .fmt(&ctx)
                .and_then(|msg| self.emitter.emit(msg))
                .or_else(|e| StdoutEmitter.emit(format!("{}", e)))
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

static ROOT_LOG: std::sync::OnceLock<Logger> = std::sync::OnceLock::new();

pub fn init_log(logger: Logger) -> Result<(), Logger> {
    ROOT_LOG.set(logger)
}

pub fn root() -> &'static Logger {
    ROOT_LOG.get_or_init(Logger::default)
}

#[track_caller]
pub fn log_with(log: &Logger, level: Level, message: fmt::Arguments<'_>) {
    log.log(Context {
        level,
        location: std::panic::Location::caller(),
        time: chrono::Utc::now(),
        message,
    });
}

#[track_caller]
pub fn log(level: Level, message: fmt::Arguments<'_>) {
    log_with(root(), level, message);
}

#[track_caller]
pub fn trace_with(log: &Logger, message: fmt::Arguments<'_>) {
    log_with(log, Level::trace(), message);
}

#[track_caller]
pub fn debug_with(log: &Logger, message: fmt::Arguments<'_>) {
    log_with(log, Level::debug(), message);
}

#[track_caller]
pub fn info_with(log: &Logger, message: fmt::Arguments<'_>) {
    log_with(log, Level::info(), message);
}

#[track_caller]
pub fn warn_with(log: &Logger, message: fmt::Arguments<'_>) {
    log_with(log, Level::warn(), message);
}

#[track_caller]
pub fn error_with(log: &Logger, message: fmt::Arguments<'_>) {
    log_with(log, Level::error(), message);
}

#[track_caller]
pub fn critical_with(log: &Logger, message: fmt::Arguments<'_>) {
    log_with(log, Level::critical(), message);
}

#[track_caller]
pub fn trace(message: fmt::Arguments<'_>) {
    log(Level::trace(), message);
}

#[track_caller]
pub fn debug(message: fmt::Arguments<'_>) {
    log(Level::debug(), message);
}

#[track_caller]
pub fn info(message: fmt::Arguments<'_>) {
    log(Level::info(), message);
}

#[track_caller]
pub fn warn(message: fmt::Arguments<'_>) {
    log(Level::warn(), message);
}

#[track_caller]
pub fn error(message: fmt::Arguments<'_>) {
    log(Level::error(), message);
}

#[track_caller]
pub fn critical(message: fmt::Arguments<'_>) {
    log(Level::critical(), message);
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {{
        $crate::log::trace(format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! trace_with {
    ($log:expr, $($arg:tt)*) => {{
        $crate::log::trace_with($log, format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        $crate::log::debug(format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! debug_with {
    ($log:expr, $($arg:tt)*) => {{
        $crate::log::debug_with($log, format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        $crate::log::info(format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! info_with {
    ($log:expr, $($arg:tt)*) => {{
        $crate::log::info_with($log, format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        $crate::log::warn(format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! warn_with {
    ($log:expr, $($arg:tt)*) => {{
        $crate::log::warn_with($log, format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        $crate::log::error(format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! error_with {
    ($log:expr, $($arg:tt)*) => {{
        $crate::log::error_with($log, format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! critical {
    ($($arg:tt)*) => {{
        $crate::log::critical(format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! critical_with {
    ($log:expr, $($arg:tt)*) => {{
        $crate::log::critical_with($log, format_args!($($arg)*))
    }};
}

pub use crate::{
    critical, critical_with, debug, debug_with, error, error_with, info, info_with, trace,
    trace_with, warn, warn_with,
};
