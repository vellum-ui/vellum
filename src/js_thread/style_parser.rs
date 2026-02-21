use crate::ipc::{
    BoxStyle, ColorValue, CrossAlign, FlexDirection, FontStyleValue, MainAlign, PaddingValue,
    TextAlignValue,
};

/// Parse a JSON-encoded style object into BoxStyle
pub fn parse_style_json(json_str: &str) -> Option<BoxStyle> {
    let json_str = json_str.trim();
    if json_str.is_empty() || json_str == "{}" || json_str == "null" {
        return None;
    }

    let mut style = BoxStyle::default();

    let inner = json_str
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim();
    if inner.is_empty() {
        return None;
    }

    let mut has_any = false;
    for pair in split_json_pairs(inner) {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }
        if let Some((key, value)) = parse_json_kv(pair) {
            if apply_style_property(&mut style, &key, &value) {
                has_any = true;
            }
        }
    }

    if has_any { Some(style) } else { None }
}

/// Extract a single widget-specific property value from a JSON string.
/// Returns None if the key is not found.
pub fn extract_json_value(json_str: &str, target_key: &str) -> Option<String> {
    let inner = json_str
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim();
    if inner.is_empty() {
        return None;
    }

    for pair in split_json_pairs(inner) {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }
        if let Some((key, value)) = parse_json_kv(pair) {
            if key == target_key {
                return Some(value);
            }
        }
    }
    None
}

/// Split JSON pairs at top-level commas (not inside nested braces/strings)
pub fn split_json_pairs(s: &str) -> Vec<&str> {
    let mut pairs = Vec::new();
    let mut depth = 0;
    let mut in_string = false;
    let mut escape = false;
    let mut start = 0;

    for (i, c) in s.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        match c {
            '\\' if in_string => escape = true,
            '"' => in_string = !in_string,
            '{' | '[' if !in_string => depth += 1,
            '}' | ']' if !in_string => depth -= 1,
            ',' if !in_string && depth == 0 => {
                pairs.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        pairs.push(&s[start..]);
    }
    pairs
}

/// Parse a "key": value pair
pub fn parse_json_kv(pair: &str) -> Option<(String, String)> {
    let colon_pos = pair.find(':')?;
    let key = pair[..colon_pos]
        .trim()
        .trim_matches('"')
        .trim()
        .to_string();
    let value = pair[colon_pos + 1..].trim().to_string();
    Some((key, value))
}

/// Unquote a JSON string value
pub fn unquote(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len() - 1]
            .replace("\\\"", "\"")
            .replace("\\\\", "\\")
    } else {
        s.to_string()
    }
}

/// Apply a single style property to a BoxStyle.
/// Returns true if the property was recognized and applied.
pub fn apply_style_property(style: &mut BoxStyle, key: &str, value: &str) -> bool {
    let value = value.trim();
    match key {
        // Text styles
        "fontSize" | "font_size" => {
            style.font_size = value.trim_matches('"').parse::<f32>().ok();
        }
        "fontWeight" | "font_weight" => {
            style.font_weight = value.trim_matches('"').parse::<f32>().ok().or_else(|| {
                match unquote(value).to_lowercase().as_str() {
                    "thin" => Some(100.0),
                    "extralight" | "extra-light" => Some(200.0),
                    "light" => Some(300.0),
                    "normal" | "regular" => Some(400.0),
                    "medium" => Some(500.0),
                    "semibold" | "semi-bold" => Some(600.0),
                    "bold" => Some(700.0),
                    "extrabold" | "extra-bold" => Some(800.0),
                    "black" => Some(900.0),
                    _ => None,
                }
            });
        }
        "fontStyle" | "font_style" => {
            style.font_style = match unquote(value).to_lowercase().as_str() {
                "italic" => Some(FontStyleValue::Italic),
                "normal" => Some(FontStyleValue::Normal),
                _ => None,
            };
        }
        "fontFamily" | "font_family" => {
            style.font_family = Some(unquote(value));
        }
        "color" => {
            style.color = ColorValue::parse(&unquote(value));
        }
        "letterSpacing" | "letter_spacing" => {
            style.letter_spacing = value.trim_matches('"').parse::<f32>().ok();
        }
        "lineHeight" | "line_height" => {
            style.line_height = value.trim_matches('"').parse::<f32>().ok();
        }
        "wordSpacing" | "word_spacing" => {
            style.word_spacing = value.trim_matches('"').parse::<f32>().ok();
        }
        "underline" => {
            style.underline = parse_json_bool(value);
        }
        "strikethrough" => {
            style.strikethrough = parse_json_bool(value);
        }
        "textAlign" | "text_align" => {
            style.text_align = match unquote(value).to_lowercase().as_str() {
                "start" | "left" => Some(TextAlignValue::Start),
                "center" => Some(TextAlignValue::Center),
                "end" | "right" => Some(TextAlignValue::End),
                "justify" => Some(TextAlignValue::Justify),
                _ => None,
            };
        }

        // Box styles
        "background" | "backgroundColor" | "background_color" | "bg" => {
            style.background = ColorValue::parse(&unquote(value));
        }
        "borderColor" | "border_color" => {
            style.border_color = ColorValue::parse(&unquote(value));
        }
        "borderWidth" | "border_width" => {
            style.border_width = value.trim_matches('"').parse::<f64>().ok();
        }
        "cornerRadius" | "corner_radius" | "borderRadius" | "border_radius" => {
            style.corner_radius = value.trim_matches('"').parse::<f64>().ok();
        }
        "padding" => {
            if let Ok(v) = value.trim_matches('"').parse::<f64>() {
                style.padding = Some(PaddingValue::Uniform(v));
            } else {
                let inner = unquote(value);
                if inner.contains(',') {
                    let parts: Vec<f64> = inner
                        .split(',')
                        .filter_map(|p| p.trim().parse::<f64>().ok())
                        .collect();
                    match parts.len() {
                        1 => style.padding = Some(PaddingValue::Uniform(parts[0])),
                        2 => {
                            style.padding = Some(PaddingValue::Sides {
                                top: parts[0],
                                right: parts[1],
                                bottom: parts[0],
                                left: parts[1],
                            })
                        }
                        4 => {
                            style.padding = Some(PaddingValue::Sides {
                                top: parts[0],
                                right: parts[1],
                                bottom: parts[2],
                                left: parts[3],
                            })
                        }
                        _ => {}
                    }
                }
            }
        }
        "width" => {
            style.width = value.trim_matches('"').parse::<f64>().ok();
        }
        "height" => {
            style.height = value.trim_matches('"').parse::<f64>().ok();
        }

        // Flex-child property
        "flex" | "flexGrow" | "flex_grow" => {
            style.flex = value.trim_matches('"').parse::<f64>().ok();
        }

        // Flex container styles (also applied via BoxStyle for runtime updates)
        "direction" | "flexDirection" | "flex_direction" => {
            style.direction = match unquote(value).to_lowercase().as_str() {
                "row" | "horizontal" => Some(FlexDirection::Row),
                "column" | "vertical" => Some(FlexDirection::Column),
                _ => None,
            };
        }
        "crossAxisAlignment" | "cross_axis_alignment" | "alignItems" | "align_items" => {
            style.cross_axis_alignment = match unquote(value).to_lowercase().as_str() {
                "start" | "flex-start" => Some(CrossAlign::Start),
                "center" => Some(CrossAlign::Center),
                "end" | "flex-end" => Some(CrossAlign::End),
                "fill" | "stretch" => Some(CrossAlign::Fill),
                "baseline" => Some(CrossAlign::Baseline),
                _ => None,
            };
        }
        "mainAxisAlignment" | "main_axis_alignment" | "justifyContent" | "justify_content" => {
            style.main_axis_alignment = match unquote(value).to_lowercase().as_str() {
                "start" | "flex-start" => Some(MainAlign::Start),
                "center" => Some(MainAlign::Center),
                "end" | "flex-end" => Some(MainAlign::End),
                "space-between" | "spaceBetween" => Some(MainAlign::SpaceBetween),
                "space-around" | "spaceAround" => Some(MainAlign::SpaceAround),
                "space-evenly" | "spaceEvenly" => Some(MainAlign::SpaceEvenly),
                _ => None,
            };
        }
        "gap" => {
            style.gap = value.trim_matches('"').parse::<f64>().ok();
        }
        "mustFillMainAxis" | "must_fill_main_axis" => {
            style.must_fill_main_axis = parse_json_bool(value);
        }

        // Widget-specific properties are NOT handled here anymore.
        // They are parsed per-widget in build_widget_data().
        // We still return false so callers know this wasn't a recognized style prop.
        _ => {
            return false;
        }
    }
    true
}

pub fn parse_json_bool(s: &str) -> Option<bool> {
    let s = s.trim().trim_matches('"');
    match s {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

pub fn parse_json_f64(s: &str) -> Option<f64> {
    s.trim().trim_matches('"').parse::<f64>().ok()
}
