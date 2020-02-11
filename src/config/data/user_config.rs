use super::super::error::ConfigError;
use crate::color::palette::Palette;
use ansi_term::Colour;
use ansi_term::Style;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserConfig {
    palette: Option<PaletteConfig>,
}

impl UserConfig {
    pub fn palette(&self) -> Result<Option<Palette>, ConfigError> {
        match &self.palette {
            Some(p) => Ok(Some(p.make()?)),
            None => Ok(None),
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
    fn make(&self) -> Result<Palette, ConfigError> {
        Ok(Palette {
            branch: parse_style(&self.branch)?,
            clean: parse_style(&self.clean)?,
            cloning: parse_style(&self.cloning)?,
            dirty: parse_style(&self.dirty)?,
            error: parse_style(&self.error)?,
            missing: parse_style(&self.missing)?,
            repo: parse_style(&self.repo)?,
            repo_exists: parse_style(&self.repo_exists)?,
        })
    }
}

pub enum ColourConfig<'conf> {
    Fixed(u8),
    Hex(&'conf str),
    Named(&'conf str),
    RGB(u8, u8, u8),
}

fn parse_style(v: &toml::Value) -> Result<Style, ConfigError> {
    ColourConfig::from(v)?.make_style()
}

fn in_range_inclusive(value: i64, min: i64, max: i64) -> bool {
    value >= min && value <= max
}

fn is_u8(value: i64) -> bool {
    in_range_inclusive(value, 0, 255)
}

impl<'conf> ColourConfig<'conf> {
    fn from(v: &'conf toml::Value) -> Result<Self, ConfigError> {
        match v {
            toml::Value::String(name) => {
                if !name.is_empty() && name.chars().nth(0) == Some('#') {
                    Ok(ColourConfig::Hex(name))
                } else {
                    Ok(ColourConfig::Named(name))
                }
            }
            toml::Value::Integer(fixed) => {
                if is_u8(*fixed) {
                    Ok(ColourConfig::Fixed(*fixed as u8))
                } else {
                    Err(ConfigError::InvalidConfig(format!(
                        "Palette value out of range [0, 255]: {}",
                        fixed
                    )))
                }
            }
            toml::Value::Array(ref rgb) if rgb.len() == 3 => match (&rgb[0], &rgb[1], &rgb[2]) {
                (toml::Value::Integer(r), toml::Value::Integer(g), toml::Value::Integer(b)) => {
                    if is_u8(*r) && is_u8(*g) && is_u8(*b) {
                        Ok(ColourConfig::RGB(*r as u8, *g as u8, *b as u8))
                    } else {
                        Err(ConfigError::InvalidConfig(format!(
                            "RGB value out of range [0, 255]: ({}, {}, {})",
                            r, g, b
                        )))
                    }
                }
                _ => Err(ConfigError::InvalidConfig(
                    "RGB value must be an array of 3 integers.".to_string(),
                )),
            },
            _ => Err(ConfigError::InvalidConfig(
                "Colour definition must be string, u8 or array of 3 integers.".to_string(),
            )),
        }
    }
}

fn u8_hex(hex: &str) -> Result<u8, ConfigError> {
    u8::from_str_radix(hex, 16)
        .map_err(|_| ConfigError::InvalidConfig(format!("Invalid hex value: {}", hex)))
}

impl<'conf> ColourConfig<'conf> {
    fn make_style(&self) -> Result<Style, ConfigError> {
        match self {
            ColourConfig::Fixed(value) => Ok(Colour::Fixed(*value)),
            ColourConfig::Hex(hex) => {
                if hex.len() == 7 {
                    Ok(Colour::RGB(
                        u8_hex(&hex[1..3])?,
                        u8_hex(&hex[3..5])?,
                        u8_hex(&hex[5..7])?,
                    ))
                } else {
                    Err(ConfigError::InvalidConfig(format!(
                        "Invalid hex colour code: {}",
                        hex
                    )))
                }
            }
            ColourConfig::Named(name) => match *name {
                "black" => Ok(Colour::Black),
                "red" => Ok(Colour::Red),
                "green" => Ok(Colour::Green),
                "yellow" => Ok(Colour::Yellow),
                "blue" => Ok(Colour::Blue),
                "purple" => Ok(Colour::Purple),
                "cyan" => Ok(Colour::Cyan),
                "white" => Ok(Colour::White),
                _ => Err(ConfigError::InvalidConfig(format!(
                    "Unsupported colour name: {}",
                    name
                ))),
            },
            ColourConfig::RGB(r, g, b) => Ok(Colour::RGB(*r, *g, *b)),
        }
        .map(|colour| colour.normal())
    }
}
