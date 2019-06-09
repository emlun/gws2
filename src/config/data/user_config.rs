use crate::color::palette::Palette;
use ansi_term::Colour;
use ansi_term::Style;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserConfig {
    palette: Option<PaletteConfig>,
}

impl UserConfig {
    pub fn palette(&self) -> Option<Palette> {
        match &self.palette {
            Some(p) => Some(p.make()),
            None => None,
        }
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

impl PaletteConfig {
    fn make(&self) -> Palette {
        Palette {
            branch: ColourConfig::from(&self.branch).make_style(),
            clean: ColourConfig::from(&self.clean).make_style(),
            cloning: ColourConfig::from(&self.cloning).make_style(),
            dirty: ColourConfig::from(&self.dirty).make_style(),
            error: ColourConfig::from(&self.error).make_style(),
            missing: ColourConfig::from(&self.missing).make_style(),
            repo: ColourConfig::from(&self.repo).make_style(),
            repo_exists: ColourConfig::from(&self.repo_exists).make_style(),
        }
    }
}

pub enum ColourConfig<'conf> {
    Named(&'conf str),
    Fixed(u8),
    RGB(u8, u8, u8),
}

impl<'conf> From<&'conf toml::Value> for ColourConfig<'conf> {
    fn from(v: &'conf toml::Value) -> Self {
        match v {
            toml::Value::String(name) => ColourConfig::Named(name),
            toml::Value::Integer(fixed) => {
                if (0..=255).contains(fixed) {
                    ColourConfig::Fixed(*fixed as u8)
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

impl<'conf> ColourConfig<'conf> {
    fn make_style(&self) -> Style {
        match self {
            ColourConfig::Named(name) => match *name {
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
            ColourConfig::Fixed(value) => Colour::Fixed(*value),
            ColourConfig::RGB(r, g, b) => Colour::RGB(*r, *g, *b),
        }
        .normal()
    }
}
