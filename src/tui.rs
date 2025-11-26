use std::collections::HashSet;
use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Default for RgbColor {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

impl From<(u8, u8, u8)> for RgbColor {
    fn from(value: (u8, u8, u8)) -> Self {
        Self {
            r: value.0,
            g: value.1,
            b: value.2,
        }
    }
}

impl RgbColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const fn black() -> Self {
        Self::new(0, 0, 0)
    }

    pub const fn red() -> Self {
        Self::new(205, 0, 0)
    }

    pub const fn green() -> Self {
        Self::new(0, 205, 0)
    }

    pub const fn yellow() -> Self {
        Self::new(205, 205, 0)
    }

    pub const fn blue() -> Self {
        Self::new(0, 0, 205)
    }

    pub const fn magenta() -> Self {
        Self::new(205, 0, 205)
    }

    pub const fn cyan() -> Self {
        Self::new(0, 205, 205)
    }

    pub const fn white() -> Self {
        Self::new(229, 229, 229)
    }

    pub const fn bright_black() -> Self {
        Self::new(127, 127, 127)
    }

    pub const fn bright_red() -> Self {
        Self::new(255, 0, 0)
    }

    pub const fn bright_green() -> Self {
        Self::new(0, 255, 0)
    }

    pub const fn bright_yellow() -> Self {
        Self::new(255, 255, 0)
    }

    pub const fn bright_blue() -> Self {
        Self::new(92, 92, 255)
    }

    pub const fn bright_magenta() -> Self {
        Self::new(255, 0, 255)
    }

    pub const fn bright_cyan() -> Self {
        Self::new(0, 255, 255)
    }

    pub const fn bright_white() -> Self {
        Self::new(255, 255, 255)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextEffect {
    Bold,
    Dim,
    Italic,
    Underline,
    SlowBlink,
    RapidBlink,
    Reverse,
    Strikethrough,
    DoubleUnderline,
}

#[derive(Debug, Default, Clone)]
pub struct DomStyle {
    indentation: u32,
    effects: Option<HashSet<TextEffect>>,
    bg: Option<RgbColor>,
    fg: Option<RgbColor>,
}

impl DomStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn indent(mut self, v: u32) -> Self {
        self.indentation = v;
        self
    }

    pub fn effects<I: IntoIterator<Item = TextEffect>>(mut self, effects: I) -> Self {
        for effect in effects {
            self.effects.get_or_insert_with(HashSet::new).insert(effect);
        }
        self
    }

    pub fn effect(mut self, effect: TextEffect) -> Self {
        self.effects.get_or_insert_with(HashSet::new).insert(effect);
        self
    }

    pub fn bg(mut self, color: RgbColor) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn fg(mut self, color: RgbColor) -> Self {
        self.fg = Some(color);
        self
    }
}

#[derive(Debug, Default, Clone)]
pub struct Layout {
    children: Vec<DomNode>,
    style: DomStyle,
}

impl Layout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn style(mut self, style: DomStyle) -> Self {
        self.style = style;
        self
    }

    pub fn append_child<N: Into<DomNode>>(mut self, child: N) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn append_children<N: Into<DomNode>, I: IntoIterator<Item = N>>(
        mut self,
        children: I,
    ) -> Self {
        for child in children {
            self.children.push(child.into());
        }
        self
    }

    pub fn len(&self) -> usize {
        self.children.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &DomNode> {
        self.children.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct Paragraph {
    text: String,
    newline: bool,
}

impl Paragraph {
    pub fn new<'a>(args: fmt::Arguments<'a>) -> Self {
        Self {
            text: fmt::format(args),
            newline: true,
        }
    }
    pub fn no_newline(mut self) -> Self {
        self.newline = false;
        self
    }
}

#[derive(Debug, Clone)]
pub enum DomNode {
    VStack(Layout),
    Text(Paragraph),
}

pub use DomNode::VStack;

impl From<Paragraph> for DomNode {
    fn from(value: Paragraph) -> Self {
        Self::Text(value)
    }
}

impl From<Layout> for DomNode {
    fn from(value: Layout) -> Self {
        Self::VStack(value)
    }
}

#[macro_export]
macro_rules! paragraph {
    ($($args: expr), *) => {
        tui::DomNode::Text(tui::Paragraph::new(format_args!($($args), *)))
    };
}
mod ansi {
    use std::fmt;

    use crate::tui::{DomNode, DomStyle, Layout, Paragraph, RgbColor, TextEffect};

    static ANSI_BG_MAP: [(RgbColor, u32); 16] = [
        (RgbColor::black(), 40),
        (RgbColor::red(), 41),
        (RgbColor::green(), 42),
        (RgbColor::yellow(), 43),
        (RgbColor::blue(), 44),
        (RgbColor::magenta(), 45),
        (RgbColor::cyan(), 46),
        (RgbColor::white(), 47),
        (RgbColor::bright_black(), 100),
        (RgbColor::bright_red(), 101),
        (RgbColor::bright_green(), 102),
        (RgbColor::bright_yellow(), 103),
        (RgbColor::bright_blue(), 104),
        (RgbColor::bright_magenta(), 105),
        (RgbColor::bright_cyan(), 106),
        (RgbColor::bright_white(), 107),
    ];

    static ANSI_FG_MAP: [(RgbColor, u32); 16] = [
        (RgbColor::black(), 30),
        (RgbColor::red(), 31),
        (RgbColor::green(), 32),
        (RgbColor::yellow(), 33),
        (RgbColor::blue(), 34),
        (RgbColor::magenta(), 35),
        (RgbColor::cyan(), 36),
        (RgbColor::white(), 37),
        (RgbColor::bright_black(), 90),
        (RgbColor::bright_red(), 91),
        (RgbColor::bright_green(), 92),
        (RgbColor::bright_yellow(), 93),
        (RgbColor::bright_blue(), 94),
        (RgbColor::bright_magenta(), 95),
        (RgbColor::bright_cyan(), 96),
        (RgbColor::bright_white(), 97),
    ];

    static ANSI_EFFECT_MAP: [(TextEffect, u32); 9] = [
        (TextEffect::Bold, 1),
        (TextEffect::Dim, 2),
        (TextEffect::Italic, 3),
        (TextEffect::Underline, 4),
        (TextEffect::SlowBlink, 5),
        (TextEffect::RapidBlink, 6),
        (TextEffect::Reverse, 7),
        (TextEffect::Strikethrough, 8),
        (TextEffect::DoubleUnderline, 9),
    ];

    fn render_style(style: &DomStyle) -> Option<String> {
        let mut codes: Vec<String> = Vec::new();
        if let Some(effects) = &style.effects {
            for effect in effects.iter() {
                if let Some(code) = ANSI_EFFECT_MAP.iter().find_map(|(key, code)| {
                    if key == effect {
                        return Some(code.to_string());
                    }
                    None
                }) {
                    codes.push(code);
                }
            }
        }
        if let Some(bg) = style.bg
            && let Some(code) = ANSI_BG_MAP.iter().find_map(|(key, code)| {
                if key == &bg {
                    return Some(code.to_string());
                }
                None
            })
        {
            codes.push(code);
        }
        if let Some(fg) = style.fg
            && let Some(code) = ANSI_FG_MAP.iter().find_map(|(key, code)| {
                if key == &fg {
                    return Some(code.to_string());
                }
                None
            })
        {
            codes.push(code);
        }
        match codes.len() {
            0 => None,
            _ => Some(format!("\x1b[{}m", codes.join(";"))),
        }
    }

    pub fn render_dom(dom: &DomNode, buf: &mut impl fmt::Write) -> Result<(), fmt::Error> {
        recursive_render_dom(dom, buf, 0, None)
    }

    fn recursive_render_dom(
        dom: &DomNode,
        buf: &mut impl fmt::Write,
        indent: usize,
        prev_style: Option<&String>,
    ) -> Result<(), fmt::Error> {
        match dom {
            DomNode::VStack(layout) => recursive_render_vstack(layout, buf, indent, prev_style),
            DomNode::Text(paragraph) => recursive_render_text(paragraph, buf, indent),
        }
    }

    fn reset_format(buf: &mut impl fmt::Write) -> Result<(), fmt::Error> {
        write!(buf, "\x1b[0m")
    }

    pub fn recursive_render_vstack(
        dom: &Layout,
        buf: &mut impl fmt::Write,
        indent: usize,
        prev_style: Option<&String>,
    ) -> Result<(), fmt::Error> {
        let cur_codes = render_style(&dom.style);
        if let Some(code_str) = &cur_codes {
            reset_format(buf)?;
            write!(buf, "{}", code_str)?;
        }
        for child in dom.iter() {
            recursive_render_dom(
                child,
                buf,
                indent + dom.style.indentation as usize,
                cur_codes.as_ref(),
            )?;
        }
        if cur_codes.is_some() {
            reset_format(buf)?;
        }
        if let Some(s) = prev_style {
            write!(buf, "{}", s)?;
        }
        Ok(())
    }

    pub fn recursive_render_text(
        dom: &Paragraph,
        buf: &mut impl fmt::Write,
        indent: usize,
    ) -> Result<(), fmt::Error> {
        write!(buf, "{:indent$}", "")?;
        if dom.newline {
            writeln!(buf, "{}", dom.text)
        } else {
            write!(buf, "{}", dom.text)
        }
    }
}

impl Display for DomNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ansi::render_dom(self, f)
    }
}

impl Display for Paragraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ansi::recursive_render_text(self, f, 0)
    }
}

impl Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ansi::recursive_render_vstack(self, f, 0, None)
    }
}
