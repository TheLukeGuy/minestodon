use serde::{Deserialize, Serialize};
use serde_json::Number;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Text {
    String(String),
    Bool(bool),
    Number(Number),
    Sequential(Vec<Self>),
    Full {
        #[serde(flatten)]
        content: TextContent,
        #[serde(rename = "extra", default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Self>,
        #[serde(flatten)]
        formatting: TextFormatting,
        // TODO: interactivity
    },
}

impl Display for Text {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Text::String(string) => f.write_str(string),
            Text::Bool(bool) => write!(f, "{bool}"),
            Text::Number(number) => write!(f, "{number}"),
            Text::Sequential(text) => {
                for text in text {
                    write!(f, "{text}")?;
                }
                Ok(())
            }
            Text::Full {
                content, children, ..
            } => {
                write!(f, "{content}")?;
                for child in children {
                    write!(f, "{child}")?;
                }
                Ok(())
            }
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

impl Display for TextContent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TextContent::Plain { text } => f.write_str(text),
            TextContent::Translated { key, .. } => f.write_str(key),
            TextContent::KeyBinding { key } => f.write_str(key),
        }
    }
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum TextColor {
    Named(NamedTextColor),
    Hex(String),
}

#[derive(Serialize, Deserialize)]
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

impl From<NamedTextColor> for TextColor {
    fn from(named: NamedTextColor) -> Self {
        Self::Named(named)
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
