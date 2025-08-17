use core::fmt;
use std::{collections::HashSet, fmt::Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    // Standard 8 colors
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    // Bright variants (8 more colors)
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    // 256-color palette
    Indexed(u8),
    // 24-bit RGB
    Rgb(u8, u8, u8),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TextFormat {
    bg: Option<Color>,
    fg: Option<Color>,
    effects: HashSet<TextEffect>,
}

impl TextFormat {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn bg(&mut self, color: Color) -> &mut Self {
        self.bg = Some(color);
        return self;
    }
    pub fn fg(&mut self, color: Color) -> &mut Self {
        self.fg = Some(color);
        return self;
    }
    pub fn effect(&mut self, effect: TextEffect) -> &mut Self {
        self.effects.insert(effect);
        return self;
    }
    pub fn effects<I: IntoIterator<Item = TextEffect>>(&mut self, effects: I) -> &mut Self {
        self.effects.extend(effects);
        return self;
    }
    pub fn has_effect(&self, effect: &impl PartialEq<TextEffect>) -> bool {
        return self.effects.iter().find(|&e| effect == e).is_some();
    }
    pub fn len_effects(&self) -> usize {
        return self.effects.len();
    }
    pub fn get_bg(&self) -> Option<&Color> {
        return self.bg.as_ref();
    }
    pub fn get_fg(&self) -> Option<&Color> {
        return self.fg.as_ref();
    }
    pub fn take(&mut self) -> Self {
        return std::mem::take(self);
    }
}

#[derive(Debug, Clone)]
pub enum TerminalNode {
    Begin(TextFormat),
    End,
    Text(String),
    NewLine,
    Indent(usize),
}

impl From<TextFormat> for TerminalNode {
    fn from(value: TextFormat) -> Self {
        return Self::Begin(value);
    }
}

impl<T: Into<String>> From<T> for TerminalNode {
    fn from(value: T) -> Self {
        return Self::Text(value.into());
    }
}

#[derive(Debug, Clone)]
pub struct TerminalNodes {
    nodes: Vec<TerminalNode>,
    ident: usize,
}

impl Default for TerminalNodes {
    fn default() -> Self {
        return Self::new(0);
    }
}

impl TerminalNodes {
    pub fn new(ident: usize) -> Self {
        return Self {
            ident,
            nodes: Vec::from([TerminalNode::Indent(ident)]),
        };
    }
    pub fn with_format(fmt: TextFormat, node: impl Into<TerminalNode>, ident: usize) -> Self {
        return Self::new(ident)
            .begin_format(fmt)
            .append_node(node)
            .end_format()
            .clone();
    }
    pub fn append_node(&mut self, n: impl Into<TerminalNode>) -> &mut Self {
        match self.nodes.last() {
            None => self.nodes.push(n.into()),
            Some(TerminalNode::NewLine) => {
                self.nodes.push(TerminalNode::Indent(self.ident));
                self.nodes.push(n.into());
            }
            Some(_) => {
                self.nodes.push(n.into());
            }
        };
        return self;
    }
    pub fn append_sub_node(&mut self, sub_nodes: impl Into<TerminalNodes>) -> &mut Self {
        for node in sub_nodes.into() {
            self.append_node(node);
        }
        return self;
    }
    pub fn begin_format(&mut self, fmt: impl Into<TextFormat>) -> &mut Self {
        self.append_node(fmt.into());
        return self;
    }
    pub fn end_format(&mut self) -> &mut Self {
        self.nodes.push(TerminalNode::End);
        return self;
    }
    pub fn new_line(&mut self) -> &mut Self {
        return self.append_node(TerminalNode::NewLine);
    }
    pub fn to_stdout(&self) {
        std::println!("{}", self);
    }
    pub fn to_stderr(&self) {
        std::eprintln!("{}", self);
    }
    pub fn len(&self) -> usize {
        return self.nodes.len();
    }
    pub fn iter(&self) -> impl Iterator<Item = &TerminalNode> {
        return self.nodes.iter();
    }
    pub fn take(&mut self) -> Self {
        return std::mem::take(self);
    }
    pub fn indent(&self) -> usize {
        return self.ident;
    }
}

impl IntoIterator for TerminalNodes {
    type Item = TerminalNode;
    type IntoIter = <Vec<TerminalNode> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        return self.nodes.into_iter();
    }
}

impl fmt::Display for TerminalNodes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for node in self.nodes.iter() {
            if let Err(e) = write!(f, "{}", node) {
                return Err(e);
            }
        }
        return Ok(());
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::Black => write!(f, "\x1b[30m"),
            Color::Red => write!(f, "\x1b[31m"),
            Color::Green => write!(f, "\x1b[32m"),
            Color::Yellow => write!(f, "\x1b[33m"),
            Color::Blue => write!(f, "\x1b[34m"),
            Color::Magenta => write!(f, "\x1b[35m"),
            Color::Cyan => write!(f, "\x1b[36m"),
            Color::White => write!(f, "\x1b[37m"),
            Color::BrightBlack => write!(f, "\x1b[90m"),
            Color::BrightRed => write!(f, "\x1b[91m"),
            Color::BrightGreen => write!(f, "\x1b[92m"),
            Color::BrightYellow => write!(f, "\x1b[93m"),
            Color::BrightBlue => write!(f, "\x1b[94m"),
            Color::BrightMagenta => write!(f, "\x1b[95m"),
            Color::BrightCyan => write!(f, "\x1b[96m"),
            Color::BrightWhite => write!(f, "\x1b[97m"),
            Color::Indexed(n) => write!(f, "\x1b[38;5;{}m", n),
            Color::Rgb(r, g, b) => write!(f, "\x1b[38;2;{};{};{}m", r, g, b),
        }
    }
}

impl fmt::Display for TextEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextEffect::Bold => write!(f, "\x1b[1m"),
            TextEffect::Dim => write!(f, "\x1b[2m"),
            TextEffect::Italic => write!(f, "\x1b[3m"),
            TextEffect::Underline => write!(f, "\x1b[4m"),
            TextEffect::SlowBlink => write!(f, "\x1b[5m"),
            TextEffect::RapidBlink => write!(f, "\x1b[6m"),
            TextEffect::Reverse => write!(f, "\x1b[7m"),
            TextEffect::Strikethrough => write!(f, "\x1b[9m"),
            TextEffect::DoubleUnderline => write!(f, "\x1b[21m"),
        }
    }
}

impl fmt::Display for TextFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(bg) = &self.bg {
            bg.fmt(f)?;
        }
        if let Some(fg) = &self.fg {
            fg.fmt(f)?;
        }
        for effect in &self.effects {
            effect.fmt(f)?;
        }
        Ok(())
    }
}

impl fmt::Display for TerminalNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TerminalNode::Begin(format) => write!(f, "{}", format),
            TerminalNode::End => write!(f, "\x1b[0m"),
            TerminalNode::Text(text) => f.write_str(text),
            TerminalNode::Indent(ident) => write!(f, "{:1$}", "", ident),
            TerminalNode::NewLine => write!(f, "\n"),
        }
    }
}
