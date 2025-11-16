use std::collections::{HashMap, HashSet};
use std::io;

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

    pub fn effects<I>(mut self, effects: I) -> Self
    where
        I: IntoIterator<Item = TextEffect>,
    {
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

#[derive(Default, Clone)]
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

    pub fn append_children<I, N>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = N>,
        N: Into<DomNode>,
    {
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

#[derive(Clone)]
pub enum DomNode {
    VStack(Layout),
    Paragraph(String),
}
pub use DomNode::Paragraph;
pub use DomNode::VStack;

impl<T: Into<String>> From<T> for DomNode {
    fn from(value: T) -> Self {
        Self::Paragraph(value.into())
    }
}

pub trait DomRenderer {
    fn render(&mut self, dom: &DomNode) -> Result<(), std::io::Error>;
    fn clear(&mut self) -> Result<(), std::io::Error>;
}

pub struct AnsiTerminal<T: io::Write> {
    out: T,
}

impl Default for AnsiTerminal<io::Stdout> {
    fn default() -> Self {
        Self { out: io::stdout() }
    }
}

impl Default for AnsiTerminal<io::Stderr> {
    fn default() -> Self {
        Self { out: io::stderr() }
    }
}

impl<T: io::Write> AnsiTerminal<T> {
    fn color_map() -> HashMap<RgbColor, (u8, u8)> {
        HashMap::from([
            (RgbColor::black(), (30, 40)),
            (RgbColor::red(), (31, 41)),
            (RgbColor::green(), (32, 42)),
            (RgbColor::yellow(), (33, 43)),
            (RgbColor::blue(), (34, 44)),
            (RgbColor::magenta(), (35, 45)),
            (RgbColor::cyan(), (36, 46)),
            (RgbColor::white(), (37, 47)),
            (RgbColor::bright_black(), (90, 100)),
            (RgbColor::bright_red(), (91, 101)),
            (RgbColor::bright_green(), (92, 102)),
            (RgbColor::bright_yellow(), (93, 103)),
            (RgbColor::bright_blue(), (94, 104)),
            (RgbColor::bright_magenta(), (95, 105)),
            (RgbColor::bright_cyan(), (96, 106)),
            (RgbColor::bright_white(), (97, 107)),
        ])
    }

    fn effect_maps() -> HashMap<TextEffect, u8> {
        HashMap::from([
            (TextEffect::Bold, 1),
            (TextEffect::Dim, 2),
            (TextEffect::Italic, 3),
            (TextEffect::Underline, 4),
            (TextEffect::SlowBlink, 5),
            (TextEffect::RapidBlink, 6),
            (TextEffect::Reverse, 7),
            (TextEffect::Strikethrough, 8),
            (TextEffect::DoubleUnderline, 9),
        ])
    }

    fn render_style(style: &DomStyle) -> Option<String> {
        let mut codes: Vec<String> = Vec::new();
        if let Some(effects) = &style.effects {
            for effect in effects.iter() {
                codes.push(Self::effect_maps()[effect].to_string());
            }
        }
        if let Some(bg) = style.bg {
            codes.push(Self::color_map()[&bg].1.to_string());
        }
        if let Some(fg) = style.fg {
            codes.push(Self::color_map()[&fg].0.to_string());
        }
        match codes.len() {
            0 => None,
            _ => Some(format!("\x1b[{}m", codes.join(";"))),
        }
    }

    fn render_paragraph(&mut self, p: &String, indent: u32) -> Result<(), std::io::Error> {
        writeln!(self.out, "{}{}", " ".repeat(indent as usize), p)
    }

    fn reset_format(&mut self) -> Result<(), std::io::Error> {
        write!(self.out, "\x1b[0m")
    }

    fn render_stack(
        &mut self,
        layout: &Layout,
        indent: u32,
        prev_style: Option<&String>,
    ) -> Result<(), std::io::Error> {
        let new_indent = indent + layout.style.indentation;
        if prev_style.is_some() {
            self.reset_format()?;
        }
        let style = Self::render_style(&layout.style);
        if let Some(s) = &style {
            write!(self.out, "{}", s)?;
        }
        for child in layout.iter() {
            self.recursive_render(child, new_indent, prev_style)?;
        }
        if style.is_some() {
            self.reset_format()?;
        }
        if let Some(s) = prev_style {
            write!(self.out, "{}", s)?;
        }
        Ok(())
    }
    fn recursive_render(
        &mut self,
        dom: &DomNode,
        indent: u32,
        prev_style: Option<&String>,
    ) -> Result<(), std::io::Error> {
        match dom {
            Paragraph(t) => self.render_paragraph(t, indent),
            VStack(layout) => self.render_stack(layout, indent, prev_style),
        }
    }
}
impl<T: io::Write> DomRenderer for AnsiTerminal<T> {
    fn render(&mut self, dom: &DomNode) -> Result<(), std::io::Error> {
        self.recursive_render(dom, 0, None)
    }
    fn clear(&mut self) -> Result<(), std::io::Error> {
        write!(self.out, "\x1b[2J")
    }
}
