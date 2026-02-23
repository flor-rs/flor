use crate::view::control_state::ControlState;
use flor_base::graphics::FontWeight;
use flor_base::types::Color;

/// Parse state prefix (hover:, focus:, active:, disabled:) from class name
pub fn parse_state_prefix(class: &str) -> (ControlState, &str) {
    if let Some(rest) = class.strip_prefix("hover:") {
        (ControlState::Hover, rest)
    } else if let Some(rest) = class.strip_prefix("focus:") {
        (ControlState::Focus, rest)
    } else if let Some(rest) = class.strip_prefix("active:") {
        (ControlState::Active, rest)
    } else if let Some(rest) = class.strip_prefix("disabled:") {
        (ControlState::Disabled, rest)
    } else {
        (ControlState::Normal, class)
    }
}

// Helper functions for parsing
pub fn extract_bracket_value(s: &str) -> Option<&str> {
    if s.starts_with('[') && s.ends_with(']') {
        Some(&s[1..s.len() - 1])
    } else {
        None
    }
}

/// 解析颜色值
///
/// 支持:
/// - 关键字: `transparent`, `black`, `white`
/// - Hex: `#fff`, `#ffffff`, `[#fff]`
/// - Tailwind: `red-500`, `blue-100`
pub fn parse_color(value: &str) -> Option<Color> {
    // 关键字
    match value {
        "transparent" => return Some(Color::rgba(0, 0, 0, 0)),
        "black" => return Some(Color::BLACK),
        "white" => return Some(Color::WHITE),
        _ => {}
    }

    // #hex 或 [#hex]
    if value.starts_with('#') {
        return Color::from_hex_str(value).ok();
    }
    if let Some(inner) = extract_bracket_value(value) {
        if inner.starts_with('#') {
            return Color::from_hex_str(inner).ok();
        }
    }

    // TW 颜色名 (color-shade)
    if let Some(idx) = value.rfind('-') {
        let color_name = &value[..idx];
        let shade = &value[idx + 1..];
        return parse_tw_color(color_name, shade);
    }

    None
}

/// 解析 Tailwind 颜色名
///
/// 例如: `parse_tw_color("red", "500")` -> `Color::RED_500`
pub fn parse_tw_color(color_name: &str, shade: &str) -> Option<Color> {
    let shade_num: u16 = shade.parse().ok()?;

    match (color_name, shade_num) {
        // Slate
        ("slate", 50) => Some(Color::SLATE_50),
        ("slate", 100) => Some(Color::SLATE_100),
        ("slate", 200) => Some(Color::SLATE_200),
        ("slate", 300) => Some(Color::SLATE_300),
        ("slate", 400) => Some(Color::SLATE_400),
        ("slate", 500) => Some(Color::SLATE_500),
        ("slate", 600) => Some(Color::SLATE_600),
        ("slate", 700) => Some(Color::SLATE_700),
        ("slate", 800) => Some(Color::SLATE_800),
        ("slate", 900) => Some(Color::SLATE_900),
        ("slate", 950) => Some(Color::SLATE_950),
        // Gray
        ("gray", 50) => Some(Color::GRAY_50),
        ("gray", 100) => Some(Color::GRAY_100),
        ("gray", 200) => Some(Color::GRAY_200),
        ("gray", 300) => Some(Color::GRAY_300),
        ("gray", 400) => Some(Color::GRAY_400),
        ("gray", 500) => Some(Color::GRAY_500),
        ("gray", 600) => Some(Color::GRAY_600),
        ("gray", 700) => Some(Color::GRAY_700),
        ("gray", 800) => Some(Color::GRAY_800),
        ("gray", 900) => Some(Color::GRAY_900),
        ("gray", 950) => Some(Color::GRAY_950),
        // Zinc
        ("zinc", 50) => Some(Color::ZINC_50),
        ("zinc", 100) => Some(Color::ZINC_100),
        ("zinc", 200) => Some(Color::ZINC_200),
        ("zinc", 300) => Some(Color::ZINC_300),
        ("zinc", 400) => Some(Color::ZINC_400),
        ("zinc", 500) => Some(Color::ZINC_500),
        ("zinc", 600) => Some(Color::ZINC_600),
        ("zinc", 700) => Some(Color::ZINC_700),
        ("zinc", 800) => Some(Color::ZINC_800),
        ("zinc", 900) => Some(Color::ZINC_900),
        ("zinc", 950) => Some(Color::ZINC_950),
        // Neutral
        ("neutral", 50) => Some(Color::NEUTRAL_50),
        ("neutral", 100) => Some(Color::NEUTRAL_100),
        ("neutral", 200) => Some(Color::NEUTRAL_200),
        ("neutral", 300) => Some(Color::NEUTRAL_300),
        ("neutral", 400) => Some(Color::NEUTRAL_400),
        ("neutral", 500) => Some(Color::NEUTRAL_500),
        ("neutral", 600) => Some(Color::NEUTRAL_600),
        ("neutral", 700) => Some(Color::NEUTRAL_700),
        ("neutral", 800) => Some(Color::NEUTRAL_800),
        ("neutral", 900) => Some(Color::NEUTRAL_900),
        ("neutral", 950) => Some(Color::NEUTRAL_950),
        // Red
        ("red", 50) => Some(Color::RED_50),
        ("red", 100) => Some(Color::RED_100),
        ("red", 200) => Some(Color::RED_200),
        ("red", 300) => Some(Color::RED_300),
        ("red", 400) => Some(Color::RED_400),
        ("red", 500) => Some(Color::RED_500),
        ("red", 600) => Some(Color::RED_600),
        ("red", 700) => Some(Color::RED_700),
        ("red", 800) => Some(Color::RED_800),
        ("red", 900) => Some(Color::RED_900),
        ("red", 950) => Some(Color::RED_950),
        // Orange
        ("orange", 50) => Some(Color::ORANGE_50),
        ("orange", 100) => Some(Color::ORANGE_100),
        ("orange", 200) => Some(Color::ORANGE_200),
        ("orange", 300) => Some(Color::ORANGE_300),
        ("orange", 400) => Some(Color::ORANGE_400),
        ("orange", 500) => Some(Color::ORANGE_500),
        ("orange", 600) => Some(Color::ORANGE_600),
        ("orange", 700) => Some(Color::ORANGE_700),
        ("orange", 800) => Some(Color::ORANGE_800),
        ("orange", 900) => Some(Color::ORANGE_900),
        ("orange", 950) => Some(Color::ORANGE_950),
        // Amber
        ("amber", 50) => Some(Color::AMBER_50),
        ("amber", 100) => Some(Color::AMBER_100),
        ("amber", 200) => Some(Color::AMBER_200),
        ("amber", 300) => Some(Color::AMBER_300),
        ("amber", 400) => Some(Color::AMBER_400),
        ("amber", 500) => Some(Color::AMBER_500),
        ("amber", 600) => Some(Color::AMBER_600),
        ("amber", 700) => Some(Color::AMBER_700),
        ("amber", 800) => Some(Color::AMBER_800),
        ("amber", 900) => Some(Color::AMBER_900),
        ("amber", 950) => Some(Color::AMBER_950),
        // Yellow
        ("yellow", 50) => Some(Color::YELLOW_50),
        ("yellow", 100) => Some(Color::YELLOW_100),
        ("yellow", 200) => Some(Color::YELLOW_200),
        ("yellow", 300) => Some(Color::YELLOW_300),
        ("yellow", 400) => Some(Color::YELLOW_400),
        ("yellow", 500) => Some(Color::YELLOW_500),
        ("yellow", 600) => Some(Color::YELLOW_600),
        ("yellow", 700) => Some(Color::YELLOW_700),
        ("yellow", 800) => Some(Color::YELLOW_800),
        ("yellow", 900) => Some(Color::YELLOW_900),
        ("yellow", 950) => Some(Color::YELLOW_950),
        // Lime
        ("lime", 50) => Some(Color::LIME_50),
        ("lime", 100) => Some(Color::LIME_100),
        ("lime", 200) => Some(Color::LIME_200),
        ("lime", 300) => Some(Color::LIME_300),
        ("lime", 400) => Some(Color::LIME_400),
        ("lime", 500) => Some(Color::LIME_500),
        ("lime", 600) => Some(Color::LIME_600),
        ("lime", 700) => Some(Color::LIME_700),
        ("lime", 800) => Some(Color::LIME_800),
        ("lime", 900) => Some(Color::LIME_900),
        ("lime", 950) => Some(Color::LIME_950),
        // Green
        ("green", 50) => Some(Color::GREEN_50),
        ("green", 100) => Some(Color::GREEN_100),
        ("green", 200) => Some(Color::GREEN_200),
        ("green", 300) => Some(Color::GREEN_300),
        ("green", 400) => Some(Color::GREEN_400),
        ("green", 500) => Some(Color::GREEN_500),
        ("green", 600) => Some(Color::GREEN_600),
        ("green", 700) => Some(Color::GREEN_700),
        ("green", 800) => Some(Color::GREEN_800),
        ("green", 900) => Some(Color::GREEN_900),
        ("green", 950) => Some(Color::GREEN_950),
        // Emerald
        ("emerald", 50) => Some(Color::EMERALD_50),
        ("emerald", 100) => Some(Color::EMERALD_100),
        ("emerald", 200) => Some(Color::EMERALD_200),
        ("emerald", 300) => Some(Color::EMERALD_300),
        ("emerald", 400) => Some(Color::EMERALD_400),
        ("emerald", 500) => Some(Color::EMERALD_500),
        ("emerald", 600) => Some(Color::EMERALD_600),
        ("emerald", 700) => Some(Color::EMERALD_700),
        ("emerald", 800) => Some(Color::EMERALD_800),
        ("emerald", 900) => Some(Color::EMERALD_900),
        ("emerald", 950) => Some(Color::EMERALD_950),
        // Teal
        ("teal", 50) => Some(Color::TEAL_50),
        ("teal", 100) => Some(Color::TEAL_100),
        ("teal", 200) => Some(Color::TEAL_200),
        ("teal", 300) => Some(Color::TEAL_300),
        ("teal", 400) => Some(Color::TEAL_400),
        ("teal", 500) => Some(Color::TEAL_500),
        ("teal", 600) => Some(Color::TEAL_600),
        ("teal", 700) => Some(Color::TEAL_700),
        ("teal", 800) => Some(Color::TEAL_800),
        ("teal", 900) => Some(Color::TEAL_900),
        ("teal", 950) => Some(Color::TEAL_950),
        // Cyan
        ("cyan", 50) => Some(Color::CYAN_50),
        ("cyan", 100) => Some(Color::CYAN_100),
        ("cyan", 200) => Some(Color::CYAN_200),
        ("cyan", 300) => Some(Color::CYAN_300),
        ("cyan", 400) => Some(Color::CYAN_400),
        ("cyan", 500) => Some(Color::CYAN_500),
        ("cyan", 600) => Some(Color::CYAN_600),
        ("cyan", 700) => Some(Color::CYAN_700),
        ("cyan", 800) => Some(Color::CYAN_800),
        ("cyan", 900) => Some(Color::CYAN_900),
        ("cyan", 950) => Some(Color::CYAN_950),
        // Sky
        ("sky", 50) => Some(Color::SKY_50),
        ("sky", 100) => Some(Color::SKY_100),
        ("sky", 200) => Some(Color::SKY_200),
        ("sky", 300) => Some(Color::SKY_300),
        ("sky", 400) => Some(Color::SKY_400),
        ("sky", 500) => Some(Color::SKY_500),
        ("sky", 600) => Some(Color::SKY_600),
        ("sky", 700) => Some(Color::SKY_700),
        ("sky", 800) => Some(Color::SKY_800),
        ("sky", 900) => Some(Color::SKY_900),
        ("sky", 950) => Some(Color::SKY_950),
        // Blue
        ("blue", 50) => Some(Color::BLUE_50),
        ("blue", 100) => Some(Color::BLUE_100),
        ("blue", 200) => Some(Color::BLUE_200),
        ("blue", 300) => Some(Color::BLUE_300),
        ("blue", 400) => Some(Color::BLUE_400),
        ("blue", 500) => Some(Color::BLUE_500),
        ("blue", 600) => Some(Color::BLUE_600),
        ("blue", 700) => Some(Color::BLUE_700),
        ("blue", 800) => Some(Color::BLUE_800),
        ("blue", 900) => Some(Color::BLUE_900),
        ("blue", 950) => Some(Color::BLUE_950),
        // Indigo
        ("indigo", 50) => Some(Color::INDIGO_50),
        ("indigo", 100) => Some(Color::INDIGO_100),
        ("indigo", 200) => Some(Color::INDIGO_200),
        ("indigo", 300) => Some(Color::INDIGO_300),
        ("indigo", 400) => Some(Color::INDIGO_400),
        ("indigo", 500) => Some(Color::INDIGO_500),
        ("indigo", 600) => Some(Color::INDIGO_600),
        ("indigo", 700) => Some(Color::INDIGO_700),
        ("indigo", 800) => Some(Color::INDIGO_800),
        ("indigo", 900) => Some(Color::INDIGO_900),
        ("indigo", 950) => Some(Color::INDIGO_950),
        // Violet
        ("violet", 50) => Some(Color::VIOLET_50),
        ("violet", 100) => Some(Color::VIOLET_100),
        ("violet", 200) => Some(Color::VIOLET_200),
        ("violet", 300) => Some(Color::VIOLET_300),
        ("violet", 400) => Some(Color::VIOLET_400),
        ("violet", 500) => Some(Color::VIOLET_500),
        ("violet", 600) => Some(Color::VIOLET_600),
        ("violet", 700) => Some(Color::VIOLET_700),
        ("violet", 800) => Some(Color::VIOLET_800),
        ("violet", 900) => Some(Color::VIOLET_900),
        ("violet", 950) => Some(Color::VIOLET_950),
        // Purple
        ("purple", 50) => Some(Color::PURPLE_50),
        ("purple", 100) => Some(Color::PURPLE_100),
        ("purple", 200) => Some(Color::PURPLE_200),
        ("purple", 300) => Some(Color::PURPLE_300),
        ("purple", 400) => Some(Color::PURPLE_400),
        ("purple", 500) => Some(Color::PURPLE_500),
        ("purple", 600) => Some(Color::PURPLE_600),
        ("purple", 700) => Some(Color::PURPLE_700),
        ("purple", 800) => Some(Color::PURPLE_800),
        ("purple", 900) => Some(Color::PURPLE_900),
        ("purple", 950) => Some(Color::PURPLE_950),
        // Fuchsia
        ("fuchsia", 50) => Some(Color::FUCHSIA_50),
        ("fuchsia", 100) => Some(Color::FUCHSIA_100),
        ("fuchsia", 200) => Some(Color::FUCHSIA_200),
        ("fuchsia", 300) => Some(Color::FUCHSIA_300),
        ("fuchsia", 400) => Some(Color::FUCHSIA_400),
        ("fuchsia", 500) => Some(Color::FUCHSIA_500),
        ("fuchsia", 600) => Some(Color::FUCHSIA_600),
        ("fuchsia", 700) => Some(Color::FUCHSIA_700),
        ("fuchsia", 800) => Some(Color::FUCHSIA_800),
        ("fuchsia", 900) => Some(Color::FUCHSIA_900),
        ("fuchsia", 950) => Some(Color::FUCHSIA_950),
        // Pink
        ("pink", 50) => Some(Color::PINK_50),
        ("pink", 100) => Some(Color::PINK_100),
        ("pink", 200) => Some(Color::PINK_200),
        ("pink", 300) => Some(Color::PINK_300),
        ("pink", 400) => Some(Color::PINK_400),
        ("pink", 500) => Some(Color::PINK_500),
        ("pink", 600) => Some(Color::PINK_600),
        ("pink", 700) => Some(Color::PINK_700),
        ("pink", 800) => Some(Color::PINK_800),
        ("pink", 900) => Some(Color::PINK_900),
        ("pink", 950) => Some(Color::PINK_950),
        // Rose
        ("rose", 50) => Some(Color::ROSE_50),
        ("rose", 100) => Some(Color::ROSE_100),
        ("rose", 200) => Some(Color::ROSE_200),
        ("rose", 300) => Some(Color::ROSE_300),
        ("rose", 400) => Some(Color::ROSE_400),
        ("rose", 500) => Some(Color::ROSE_500),
        ("rose", 600) => Some(Color::ROSE_600),
        ("rose", 700) => Some(Color::ROSE_700),
        ("rose", 800) => Some(Color::ROSE_800),
        ("rose", 900) => Some(Color::ROSE_900),
        ("rose", 950) => Some(Color::ROSE_950),
        _ => None,
    }
}

/// 解析 rounded-* 类名，返回圆角值
///
/// 支持:
/// - `rounded-none` -> 0.0
/// - `rounded-sm` -> 2.0
/// - `rounded` -> 4.0
/// - `rounded-md` -> 6.0
/// - `rounded-lg` -> 8.0
/// - `rounded-xl` -> 12.0
/// - `rounded-2xl` -> 16.0
/// - `rounded-3xl` -> 24.0
/// - `rounded-full` -> 9999.0
/// - `rounded-[8px]` -> 8.0
pub fn parse_rounded(class: &str) -> Option<f32> {
    let rest = class.strip_prefix("rounded")?;

    match rest {
        "-none" => Some(0.0),
        "-sm" => Some(2.0),
        "" => Some(4.0),
        "-md" => Some(6.0),
        "-lg" => Some(8.0),
        "-xl" => Some(12.0),
        "-2xl" => Some(16.0),
        "-3xl" => Some(24.0),
        "-full" => Some(9999.0),
        _ => {
            // rounded-[8px] 或 rounded-[8]
            let inner = rest.strip_prefix("-")?;
            if let Some(val) = extract_bracket_value(inner) {
                val.trim_end_matches("px").parse::<f32>().ok()
            } else {
                None
            }
        }
    }
}

/// 解析 font-* 类名中的字重，返回 FontWeight
///
/// 支持:
/// - `font-thin` -> Thin (100)
/// - `font-extralight` -> ExtraLight (200)
/// - `font-light` -> Light (300)
/// - `font-normal` -> Normal (400)
/// - `font-medium` -> Medium (500)
/// - `font-semibold` -> SemiBold (600)
/// - `font-bold` -> Bold (700)
/// - `font-extrabold` -> ExtraBold (800)
/// - `font-black` -> Black (900)
pub fn parse_font_weight(name: &str) -> Option<FontWeight> {
    match name {
        "thin" => Some(FontWeight::Thin),
        "extralight" => Some(FontWeight::ExtraLight),
        "light" => Some(FontWeight::Light),
        "normal" => Some(FontWeight::Normal),
        "medium" => Some(FontWeight::Medium),
        "semibold" => Some(FontWeight::SemiBold),
        "bold" => Some(FontWeight::Bold),
        "extrabold" => Some(FontWeight::ExtraBold),
        "black" => Some(FontWeight::Black),
        _ => None,
    }
}
