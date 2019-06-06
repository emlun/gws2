use crate::color::palette::Palette;
use ansi_term::Colour;
use ansi_term::Style;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserConfig {
    palette: Option<PaletteConfig>,
}

impl UserConfig {
    pub fn palette(self) -> Option<Palette> {
        self.palette.map(|p| p.into())
    }
}

#[derive(Deserialize)]
pub struct PaletteConfig {
    pub branch: toml::Value,
    pub clean: toml::Value,
    pub cloning: toml::Value,
    pub dirty: toml::Value,
    pub error: toml::Value,
    pub missing: toml::Value,
    pub repo: toml::Value,
    pub repo_exists: toml::Value,
}

impl Into<Palette> for PaletteConfig {
    fn into(self) -> Palette {
        Palette {
            branch: ColourConfig::from(self.branch).into(),
            clean: ColourConfig::from(self.clean).into(),
            cloning: ColourConfig::from(self.cloning).into(),
            dirty: ColourConfig::from(self.dirty).into(),
            error: ColourConfig::from(self.error).into(),
            missing: ColourConfig::from(self.missing).into(),
            repo: ColourConfig::from(self.repo).into(),
            repo_exists: ColourConfig::from(self.repo_exists).into(),
        }
    }
}

pub enum ColourConfig {
    Named(String),
    Fixed(u8),
    RGB(u8, u8, u8),
}

impl From<toml::Value> for ColourConfig {
    fn from(v: toml::Value) -> Self {
        match v {
            toml::Value::String(name) => ColourConfig::Named(name),
            toml::Value::Integer(fixed) => {
                if (0..=255).contains(&fixed) {
                    ColourConfig::Fixed(fixed as u8)
                } else {
                    panic!("Palette value out of range [0, 255]: {}", fixed)
                }
            }
            toml::Value::Array(ref rgb) if rgb.len() == 3 => match (&rgb[0], &rgb[1], &rgb[2]) {
                (toml::Value::Integer(r), toml::Value::Integer(g), toml::Value::Integer(b)) => {
                    if (0..=255).contains(r) && (0..=255).contains(g) && (0..=255).contains(b) {
                        ColourConfig::RGB(*r as u8, *g as u8, *b as u8)
                    } else {
                        panic!("RGB value out of range [0, 255]: ({}, {}, {})", r, g, b)
                    }
                }
                _ => panic!("RGB value must be an array of 3 integers."),
            },
            _ => panic!("Colour definition must be string, u8 or array of 3 integers."),
        }
    }
}

impl Into<Style> for ColourConfig {
    fn into(self) -> Style {
        match self {
            ColourConfig::Named(name) => match name.as_str() {
                "black" => Colour::Black,
                "red" => Colour::Red,
                "green" => Colour::Green,
                "yellow" => Colour::Yellow,
                "blue" => Colour::Blue,
                "purple" => Colour::Purple,
                "cyan" => Colour::Cyan,
                "white" => Colour::White,
                _ => panic!("Unsupported colour name: {}", name),
            },
            ColourConfig::Fixed(value) => Colour::Fixed(value),
            ColourConfig::RGB(r, g, b) => Colour::RGB(r, g, b),
        }
        .normal()
    }
}
