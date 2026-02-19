use serde::{Deserialize, Serialize};

/// Represents a parsed color value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorValue {
    /// RGBA color (0-255 per channel)
    Rgba { r: u8, g: u8, b: u8, a: u8 },
    /// Named color string
    Named(String),
}

impl ColorValue {
    /// Parse a color string like "#RRGGBB", "#RRGGBBAA", "rgb(r,g,b)", "rgba(r,g,b,a)",
    /// or named CSS colors.
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.starts_with('#') {
            let hex = &s[1..];
            match hex.len() {
                6 => {
                    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                    Some(ColorValue::Rgba { r, g, b, a: 255 })
                }
                8 => {
                    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                    let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                    Some(ColorValue::Rgba { r, g, b, a })
                }
                _ => None,
            }
        } else if s.starts_with("rgb(") || s.starts_with("rgba(") {
            let inner = s
                .trim_start_matches("rgba(")
                .trim_start_matches("rgb(")
                .trim_end_matches(')');
            let parts: Vec<&str> = inner.split(',').collect();
            if parts.len() >= 3 {
                let r = parts[0].trim().parse::<u8>().ok()?;
                let g = parts[1].trim().parse::<u8>().ok()?;
                let b = parts[2].trim().parse::<u8>().ok()?;
                let a = if parts.len() >= 4 {
                    let af = parts[3].trim().parse::<f32>().ok()?;
                    (af * 255.0) as u8
                } else {
                    255
                };
                Some(ColorValue::Rgba { r, g, b, a })
            } else {
                None
            }
        } else {
            // Try known named colors
            match s.to_lowercase().as_str() {
                "white" => Some(ColorValue::Rgba {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 255,
                }),
                "black" => Some(ColorValue::Rgba {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 255,
                }),
                "red" => Some(ColorValue::Rgba {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                }),
                "green" => Some(ColorValue::Rgba {
                    r: 0,
                    g: 128,
                    b: 0,
                    a: 255,
                }),
                "blue" => Some(ColorValue::Rgba {
                    r: 0,
                    g: 0,
                    b: 255,
                    a: 255,
                }),
                "yellow" => Some(ColorValue::Rgba {
                    r: 255,
                    g: 255,
                    b: 0,
                    a: 255,
                }),
                "cyan" => Some(ColorValue::Rgba {
                    r: 0,
                    g: 255,
                    b: 255,
                    a: 255,
                }),
                "magenta" => Some(ColorValue::Rgba {
                    r: 255,
                    g: 0,
                    b: 255,
                    a: 255,
                }),
                "orange" => Some(ColorValue::Rgba {
                    r: 255,
                    g: 165,
                    b: 0,
                    a: 255,
                }),
                "purple" => Some(ColorValue::Rgba {
                    r: 128,
                    g: 0,
                    b: 128,
                    a: 255,
                }),
                "gray" | "grey" => Some(ColorValue::Rgba {
                    r: 128,
                    g: 128,
                    b: 128,
                    a: 255,
                }),
                "transparent" => Some(ColorValue::Rgba {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 0,
                }),
                other => Some(ColorValue::Named(other.to_string())),
            }
        }
    }
}
