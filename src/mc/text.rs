use anyhow::{Context, Result};
use enum_iterator::Sequence;
use lab::Lab;
use serde::{Deserialize, Serialize};
use serde_json::Number;
use std::fmt;
use std::fmt::{Display, Formatter, Write};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Text {
    String(String),
    Bool(bool),
    Number(Number),
    Sequential(Vec<Self>),
    Full(FullText),
}

impl Text {
    pub fn push_child(self, child: impl Into<Text>) -> Self {
        self.modify_as_full(|full| full.children.push(child.into()))
    }

    pub fn push_sequential(self, other: impl Into<Text>) -> Self {
        if let Self::Sequential(mut text) = self {
            text.push(other.into());
            Self::Sequential(text)
        } else {
            Self::Sequential(vec!["".into(), self, other.into()])
        }
    }

    pub fn color(self, color: impl Into<TextColor>) -> Self {
        self.modify_as_full(|full| full.formatting.color = Some(color.into()))
    }

    pub fn font(self, font: TextFont) -> Self {
        self.modify_as_full(|full| full.formatting.font = Some(font))
    }

    pub fn bolded(self, bolded: bool) -> Self {
        self.modify_as_full(|full| full.formatting.bolded = Some(bolded))
    }

    pub fn italicized(self, italicized: bool) -> Self {
        self.modify_as_full(|full| full.formatting.italicized = Some(italicized))
    }

    pub fn underlined(self, underlined: bool) -> Self {
        self.modify_as_full(|full| full.formatting.underlined = Some(underlined))
    }

    pub fn struck_through(self, struck_through: bool) -> Self {
        self.modify_as_full(|full| full.formatting.struck_through = Some(struck_through))
    }

    pub fn obfuscated(self, obfuscated: bool) -> Self {
        self.modify_as_full(|full| full.formatting.obfuscated = Some(obfuscated))
    }

    fn modify_as_full(self, modify: impl FnOnce(&mut FullText)) -> Self {
        let mut full = match self {
            Text::Full(full) => full,
            Text::String(string) => string.into(),
            Text::Bool(bool) => bool.into(),
            Text::Number(number) => number.into(),
            Text::Sequential(children) => FullText {
                content: TextContent::default(),
                children,
                formatting: TextFormatting::default(),
            },
        };
        modify(&mut full);
        Self::Full(full)
    }

    pub fn to_plain_string(&self) -> String {
        match self {
            Self::String(string) => string.clone(),
            Self::Bool(bool) => bool.to_string(),
            Self::Number(number) => number.to_string(),
            Self::Sequential(text) => {
                let mut plain = String::new();
                for text in text {
                    plain.push_str(&text.to_plain_string());
                }
                plain
            }
            Self::Full(full) => {
                let mut plain = full.content.to_string();
                for child in &full.children {
                    plain.push_str(&child.to_plain_string());
                }
                plain
            }
        }
    }

    pub fn to_legacy_string(&self) -> String {
        match self {
            Self::Sequential(text) => {
                let mut legacy = String::new();
                for text in text {
                    legacy.push_str(&text.to_legacy_string());
                }
                legacy
            }
            Self::Full(full) => {
                let mut legacy = full.formatting.legacy_codes();
                legacy.push_str(&full.content.to_string());
                for child in &full.children {
                    legacy.push_str(&child.to_legacy_string());
                }
                legacy
            }
            _ => self.to_plain_string(),
        }
    }

    pub fn to_json_string(&self, str_type: JsonStringType) -> Result<String> {
        let string = match str_type {
            JsonStringType::Short => serde_json::to_string(self)?,
            JsonStringType::Pretty => serde_json::to_string_pretty(self)?,
        };
        Ok(string)
    }
}

impl<D: Display> From<D> for Text {
    fn from(display: D) -> Self {
        Self::String(display.to_string())
    }
}

#[derive(Serialize, Deserialize)]
pub struct FullText {
    #[serde(flatten)]
    content: TextContent,
    #[serde(rename = "extra", default, skip_serializing_if = "Vec::is_empty")]
    children: Vec<Text>,
    #[serde(flatten)]
    formatting: TextFormatting,
    // TODO: interactivity
}

impl<D: Display> From<D> for FullText {
    fn from(display: D) -> Self {
        Self {
            content: TextContent::Plain {
                text: display.to_string(),
            },
            children: vec![],
            formatting: TextFormatting::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum TextContent {
    Plain {
        text: String,
    },
    Translated {
        #[serde(rename = "translate")]
        key: String,
        #[serde(rename = "with", default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<Text>,
    },
    KeyBinding {
        #[serde(rename = "keybind")]
        key: String,
    },
}

impl Default for TextContent {
    fn default() -> Self {
        Self::Plain {
            text: String::new(),
        }
    }
}

impl Display for TextContent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plain { text } => f.write_str(text),
            Self::Translated { key, .. } => f.write_str(key),
            Self::KeyBinding { key } => f.write_str(key),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct TextFormatting {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<TextColor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<TextFont>,
    #[serde(rename = "bold", default, skip_serializing_if = "Option::is_none")]
    pub bolded: Option<bool>,
    #[serde(rename = "italic", default, skip_serializing_if = "Option::is_none")]
    pub italicized: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub underlined: Option<bool>,
    #[serde(
        rename = "strikethrough",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub struck_through: Option<bool>,
    #[serde(
        rename = "obfuscated",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub obfuscated: Option<bool>,
}

impl TextFormatting {
    pub fn legacy_codes(&self) -> String {
        const ESCAPE_CHAR: char = '\u{00a7}';

        let mut codes = String::with_capacity(2 * 6);
        if let Some(color) = &self.color {
            let legacy = color
                .legacy_char()
                .unwrap_or_else(|_| NamedTextColor::Reset.legacy_char());
            write!(codes, "{ESCAPE_CHAR}{legacy}").unwrap();
        }
        if let Some(true) = self.bolded {
            write!(codes, "{ESCAPE_CHAR}l").unwrap();
        }
        if let Some(true) = self.italicized {
            write!(codes, "{ESCAPE_CHAR}o").unwrap();
        }
        if let Some(true) = self.underlined {
            write!(codes, "{ESCAPE_CHAR}n").unwrap();
        }
        if let Some(true) = self.struck_through {
            write!(codes, "{ESCAPE_CHAR}m").unwrap();
        }
        if let Some(true) = self.obfuscated {
            write!(codes, "{ESCAPE_CHAR}k").unwrap();
        }
        codes
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum TextColor {
    Named(NamedTextColor),
    Hex(String),
}

impl TextColor {
    pub fn legacy_char(&self) -> Result<char> {
        let result = match self {
            Self::Named(named) => named.legacy_char(),
            Self::Hex(hex) => {
                let rgb = parse_hex(hex).context("failed to parse the hex string")?;
                let lab = Lab::from_rgb(&rgb);

                let mut closest = None;
                let mut closest_distance = f32::MAX;
                for color in enum_iterator::all::<NamedTextColor>() {
                    let try_lab = Lab::from_rgb(&color.vanilla());
                    let distance = try_lab.squared_distance(&lab);
                    if distance < closest_distance {
                        closest = Some(color);
                        closest_distance = distance;
                    }
                }
                closest.unwrap().legacy_char()
            }
        };
        Ok(result)
    }
}

fn parse_hex(hex: &str) -> Result<[u8; 3]> {
    let hex = hex.trim_start_matches('#');
    let red = u8::from_str_radix(&hex[0..2], 16).context("failed to parse the red value")?;
    let green = u8::from_str_radix(&hex[2..4], 16).context("failed to parse the green value")?;
    let blue = u8::from_str_radix(&hex[4..6], 16).context("failed to parse the blue value")?;
    Ok([red, green, blue])
}

#[derive(Serialize, Deserialize, Sequence)]
#[serde(rename_all = "snake_case")]
pub enum NamedTextColor {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
    Reset,
}

impl NamedTextColor {
    pub fn legacy_char(&self) -> char {
        match self {
            Self::Black => '0',
            Self::DarkBlue => '1',
            Self::DarkGreen => '2',
            Self::DarkAqua => '3',
            Self::DarkRed => '4',
            Self::DarkPurple => '5',
            Self::Gold => '6',
            Self::Gray => '7',
            Self::DarkGray => '8',
            Self::Blue => '9',
            Self::Green => 'a',
            Self::Aqua => 'b',
            Self::Red => 'c',
            Self::LightPurple => 'd',
            Self::Yellow => 'e',
            Self::White => 'f',
            Self::Reset => 'r',
        }
    }

    pub fn vanilla(&self) -> [u8; 3] {
        match self {
            Self::Black => [0, 0, 0],
            Self::DarkBlue => [0, 0, 170],
            Self::DarkGreen => [0, 170, 0],
            Self::DarkAqua => [0, 170, 170],
            Self::DarkRed => [170, 0, 0],
            Self::DarkPurple => [170, 0, 170],
            Self::Gold => [255, 170, 0],
            Self::Gray => [170, 170, 170],
            Self::DarkGray => [85, 85, 85],
            Self::Blue => [85, 85, 255],
            Self::Green => [85, 255, 85],
            Self::Aqua => [85, 255, 255],
            Self::Red => [255, 85, 85],
            Self::LightPurple => [255, 85, 255],
            Self::Yellow => [255, 255, 85],
            Self::White => [255, 255, 255],
            Self::Reset => [255, 255, 255],
        }
    }
}

impl From<NamedTextColor> for TextColor {
    fn from(named: NamedTextColor) -> Self {
        Self::Named(named)
    }
}

pub struct HexTextColor<D: Display>(pub D);

impl<D: Display> From<HexTextColor<D>> for TextColor {
    fn from(hex: HexTextColor<D>) -> Self {
        Self::Hex(hex.0.to_string())
    }
}

#[derive(Serialize, Deserialize)]
pub enum TextFont {
    #[serde(rename = "minecraft:default")]
    Default,
    #[serde(rename = "minecraft:uniform")]
    Uniform,
    #[serde(rename = "minecraft:alt")]
    EnchantingTable,
    #[serde(rename = "minecraft:illageralt")]
    Illager,
}

pub enum JsonStringType {
    Short,
    Pretty,
}

#[macro_export]
macro_rules! text {
    ($($arg:tt)*) => {{
        let formatted = ::std::format!($($arg)*);
        <$crate::mc::text::Text as ::std::convert::From<::std::string::String>>::from(formatted)
    }};
}
