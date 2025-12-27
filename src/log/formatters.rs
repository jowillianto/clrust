use super::prelude::{Context, Error, Formatter};
use crate::tui::{DomStyle, Layout, Paragraph, RgbColor};
use chrono::{Datelike, Timelike};
use std::fmt::Write;

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

impl Formatter for ColorfulFormatter {
    fn fmt(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let mut buf = String::new();
        writeln!(
            buf,
            "{} {}-{:0>2}-{:0>2}T{:0>2}:{:0>2}:{:0>2}Z {}",
            Layout::new()
                .style(DomStyle::new().fg(self.level_color(ctx.level.value)))
                .append_child(Paragraph::new(format_args!("[{}]", ctx.level.name)).no_newline()),
            ctx.time.year(),
            ctx.time.month(),
            ctx.time.day(),
            ctx.time.hour(),
            ctx.time.minute(),
            ctx.time.second(),
            ctx.message
        )
        .map_err(|_| Error::format_error(format_args!("format error")))?;
        Ok(buf)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BwFormatter;

impl Formatter for BwFormatter {
    fn fmt<'a>(&'a self, ctx: &Context<'a>) -> Result<String, Error> {
        let mut buf = String::new();
        writeln!(
            buf,
            "[{}] {}-{:0>2}-{:0>2}T{:0>2}:{:0>2}:{:0>2}Z {}",
            ctx.level.name,
            ctx.time.year(),
            ctx.time.month(),
            ctx.time.day(),
            ctx.time.hour(),
            ctx.time.minute(),
            ctx.time.second(),
            ctx.message
        )
        .map_err(|_| Error::format_error(format_args!("format error")))?;
        Ok(buf)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PlainFormatter;

impl Formatter for PlainFormatter {
    fn fmt(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let mut buf = String::new();
        writeln!(buf, "{}", ctx.message)
            .map_err(|_| Error::format_error(format_args!("format error")))?;
        Ok(buf)
    }
}
