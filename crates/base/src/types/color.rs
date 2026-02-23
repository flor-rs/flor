use std::fmt::{Display, Formatter};
use thiserror::Error;

//    Code from [`druid`] is used here
//
//    [`druid`]: https://github.com/xi-editor/druid
//
//    Copyright [2024] [name of copyright owner]
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.

#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }
}

impl Color {
    /// Attempt to create a color from a CSS-style hex string.
    ///
    /// This will accept strings in the following formats, *with or without*
    /// the leading `#`:
    ///
    /// - `rrggbb`
    /// - `rrggbbaa`
    /// - `rbg`
    /// - `rbga`
    ///
    /// This method returns a [`ColorParseError`] if the color cannot be parsed.
    pub const fn from_hex_str(hex: &str) -> Result<Color, ColorParseError> {
        // can't use `map()` in a const function
        match get_4bit_hex_channels(hex) {
            Ok(channels) => Ok(color_from_4bit_hex(channels)),
            Err(e) => Err(e),
        }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Self { r, g, b, a }
    }
    pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
        Self { r, g, b, a: 0xff }
    }

    // todo 处理掉
    pub fn rgba_gl_vec4(&self) -> String {
        format!(
            "vec4({},{},{},{});",
            self.r as f32 / 255.,
            self.g as f32 / 255.,
            self.b as f32 / 255.,
            self.a as f32 / 255.
        )
    }

    pub fn rgba_gl_vec(&self) -> Vec<f32> {
        vec![
            self.r as f32 / 255.,
            self.g as f32 / 255.,
            self.b as f32 / 255.,
            self.a as f32 / 255.,
        ]
    }

    pub fn r(mut self, r: u8) -> Self {
        self.r = r;
        self
    }
    pub fn g(mut self, g: u8) -> Self {
        self.g = g;
        self
    }
    pub fn b(mut self, b: u8) -> Self {
        self.b = b;
        self
    }
    pub fn a(mut self, a: u8) -> Self {
        self.a = a;
        self
    }
}

const fn color_from_4bit_hex(components: [u8; 8]) -> Color {
    let [r0, r1, g0, g1, b0, b1, a0, a1] = components;
    Color::rgba(r0 << 4 | r1, g0 << 4 | g1, b0 << 4 | b1, a0 << 4 | a1)
}

/// Errors that can occur when parsing a hex color.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ColorParseError {
    /// The input string has an incorrect length
    WrongSize(usize),
    /// A byte in the input string is not in one of the ranges `0..=9`,
    /// `a..=f`, or `A..=F`.
    #[allow(missing_docs)]
    NotHex { idx: usize, byte: u8 },
}

impl Display for ColorParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorParseError::WrongSize(size) => f.write_str(&format!("wrong size:{}", size))?,
            ColorParseError::NotHex { idx, byte } => {
                f.write_str(&format!("not hex, idx: {} , byte: {}", idx, byte))?
            }
        }
        Ok(())
    }
}

const fn hex_from_ascii_byte(b: u8) -> Result<u8, u8> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        _ => Err(b),
    }
}

const fn get_4bit_hex_channels(hex_str: &str) -> Result<[u8; 8], ColorParseError> {
    let mut four_bit_channels = match hex_str.as_bytes() {
        &[b'#', r, g, b] | &[r, g, b] => [r, r, g, g, b, b, b'f', b'f'],
        &[b'#', r, g, b, a] | &[r, g, b, a] => [r, r, g, g, b, b, a, a],
        &[b'#', r0, r1, g0, g1, b0, b1] | &[r0, r1, g0, g1, b0, b1] => {
            [r0, r1, g0, g1, b0, b1, b'f', b'f']
        }
        &[b'#', r0, r1, g0, g1, b0, b1, a0, a1] | &[r0, r1, g0, g1, b0, b1, a0, a1] => {
            [r0, r1, g0, g1, b0, b1, a0, a1]
        }
        other => return Err(ColorParseError::WrongSize(other.len())),
    };

    // convert to hex in-place
    // this is written without a for loop to satisfy `const`
    let mut i = 0;
    while i < four_bit_channels.len() {
        let ascii = four_bit_channels[i];
        let as_hex = match hex_from_ascii_byte(ascii) {
            Ok(hex) => hex,
            Err(byte) => return Err(ColorParseError::NotHex { idx: i, byte }),
        };
        four_bit_channels[i] = as_hex;
        i += 1;
    }
    Ok(four_bit_channels)
}

impl Color {
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);

    // ========================================================================
    // Slate (冷灰色 - 偏蓝)
    // ========================================================================
    pub const SLATE_50: Color = Color::rgb(248, 250, 252);
    pub const SLATE_100: Color = Color::rgb(241, 245, 249);
    pub const SLATE_200: Color = Color::rgb(226, 232, 240);
    pub const SLATE_300: Color = Color::rgb(203, 213, 225);
    pub const SLATE_400: Color = Color::rgb(148, 163, 184);
    pub const SLATE_500: Color = Color::rgb(100, 116, 139);
    pub const SLATE_600: Color = Color::rgb(71, 85, 105);
    pub const SLATE_700: Color = Color::rgb(51, 65, 85);
    pub const SLATE_800: Color = Color::rgb(30, 41, 59);
    pub const SLATE_900: Color = Color::rgb(15, 23, 42);
    pub const SLATE_950: Color = Color::rgb(2, 6, 23);

    // ========================================================================
    // Gray (中性灰 - 也就是 Cool Gray)
    // ========================================================================
    pub const GRAY_50: Color = Color::rgb(249, 250, 251);
    pub const GRAY_100: Color = Color::rgb(243, 244, 246);
    pub const GRAY_200: Color = Color::rgb(229, 231, 235);
    pub const GRAY_300: Color = Color::rgb(209, 213, 219);
    pub const GRAY_400: Color = Color::rgb(156, 163, 175);
    pub const GRAY_500: Color = Color::rgb(107, 114, 128);
    pub const GRAY_600: Color = Color::rgb(75, 85, 99);
    pub const GRAY_700: Color = Color::rgb(55, 65, 81);
    pub const GRAY_800: Color = Color::rgb(31, 41, 55);
    pub const GRAY_900: Color = Color::rgb(17, 24, 39);
    pub const GRAY_950: Color = Color::rgb(3, 7, 18);

    // ========================================================================
    // Zinc (锌灰 - 工业感)
    // ========================================================================
    pub const ZINC_50: Color = Color::rgb(250, 250, 250);
    pub const ZINC_100: Color = Color::rgb(244, 244, 245);
    pub const ZINC_200: Color = Color::rgb(228, 228, 231);
    pub const ZINC_300: Color = Color::rgb(212, 212, 216);
    pub const ZINC_400: Color = Color::rgb(161, 161, 170);
    pub const ZINC_500: Color = Color::rgb(113, 113, 122);
    pub const ZINC_600: Color = Color::rgb(82, 82, 91);
    pub const ZINC_700: Color = Color::rgb(63, 63, 70);
    pub const ZINC_800: Color = Color::rgb(39, 39, 42);
    pub const ZINC_900: Color = Color::rgb(24, 24, 27);
    pub const ZINC_950: Color = Color::rgb(9, 9, 11);

    // ========================================================================
    // Neutral (真中性色)
    // ========================================================================
    pub const NEUTRAL_50: Color = Color::rgb(250, 250, 250);
    pub const NEUTRAL_100: Color = Color::rgb(245, 245, 245);
    pub const NEUTRAL_200: Color = Color::rgb(229, 229, 229);
    pub const NEUTRAL_300: Color = Color::rgb(212, 212, 212);
    pub const NEUTRAL_400: Color = Color::rgb(163, 163, 163);
    pub const NEUTRAL_500: Color = Color::rgb(115, 115, 115);
    pub const NEUTRAL_600: Color = Color::rgb(82, 82, 82);
    pub const NEUTRAL_700: Color = Color::rgb(64, 64, 64);
    pub const NEUTRAL_800: Color = Color::rgb(38, 38, 38);
    pub const NEUTRAL_900: Color = Color::rgb(23, 23, 23);
    pub const NEUTRAL_950: Color = Color::rgb(10, 10, 10);

    // ========================================================================
    // Red (红色)
    // ========================================================================
    pub const RED_50: Color = Color::rgb(254, 242, 242);
    pub const RED_100: Color = Color::rgb(254, 226, 226);
    pub const RED_200: Color = Color::rgb(254, 202, 202);
    pub const RED_300: Color = Color::rgb(252, 165, 165);
    pub const RED_400: Color = Color::rgb(248, 113, 113);
    pub const RED_500: Color = Color::rgb(239, 68, 68);
    pub const RED_600: Color = Color::rgb(220, 38, 38);
    pub const RED_700: Color = Color::rgb(185, 28, 28);
    pub const RED_800: Color = Color::rgb(153, 27, 27);
    pub const RED_900: Color = Color::rgb(127, 29, 29);
    pub const RED_950: Color = Color::rgb(69, 10, 10);

    // ========================================================================
    // Orange (橙色)
    // ========================================================================
    pub const ORANGE_50: Color = Color::rgb(255, 247, 237);
    pub const ORANGE_100: Color = Color::rgb(255, 237, 213);
    pub const ORANGE_200: Color = Color::rgb(254, 215, 170);
    pub const ORANGE_300: Color = Color::rgb(253, 186, 116);
    pub const ORANGE_400: Color = Color::rgb(251, 146, 60);
    pub const ORANGE_500: Color = Color::rgb(249, 115, 22);
    pub const ORANGE_600: Color = Color::rgb(234, 88, 12);
    pub const ORANGE_700: Color = Color::rgb(194, 65, 12);
    pub const ORANGE_800: Color = Color::rgb(154, 52, 18);
    pub const ORANGE_900: Color = Color::rgb(124, 45, 18);
    pub const ORANGE_950: Color = Color::rgb(67, 20, 7);

    // ========================================================================
    // Amber (琥珀色 - 暖黄)
    // ========================================================================
    pub const AMBER_50: Color = Color::rgb(255, 251, 235);
    pub const AMBER_100: Color = Color::rgb(254, 243, 199);
    pub const AMBER_200: Color = Color::rgb(253, 230, 138);
    pub const AMBER_300: Color = Color::rgb(252, 211, 77);
    pub const AMBER_400: Color = Color::rgb(251, 191, 36);
    pub const AMBER_500: Color = Color::rgb(245, 158, 11);
    pub const AMBER_600: Color = Color::rgb(217, 119, 6);
    pub const AMBER_700: Color = Color::rgb(180, 83, 9);
    pub const AMBER_800: Color = Color::rgb(146, 64, 14);
    pub const AMBER_900: Color = Color::rgb(120, 53, 15);
    pub const AMBER_950: Color = Color::rgb(69, 26, 3);

    // ========================================================================
    // Yellow (黄色)
    // ========================================================================
    pub const YELLOW_50: Color = Color::rgb(254, 252, 232);
    pub const YELLOW_100: Color = Color::rgb(254, 249, 195);
    pub const YELLOW_200: Color = Color::rgb(254, 240, 138);
    pub const YELLOW_300: Color = Color::rgb(253, 224, 71);
    pub const YELLOW_400: Color = Color::rgb(250, 204, 21);
    pub const YELLOW_500: Color = Color::rgb(234, 179, 8);
    pub const YELLOW_600: Color = Color::rgb(202, 138, 4);
    pub const YELLOW_700: Color = Color::rgb(161, 98, 7);
    pub const YELLOW_800: Color = Color::rgb(133, 77, 14);
    pub const YELLOW_900: Color = Color::rgb(113, 63, 18);
    pub const YELLOW_950: Color = Color::rgb(66, 32, 6);

    // ========================================================================
    // Lime (酸橙绿)
    // ========================================================================
    pub const LIME_50: Color = Color::rgb(247, 254, 231);
    pub const LIME_100: Color = Color::rgb(236, 252, 203);
    pub const LIME_200: Color = Color::rgb(217, 249, 157);
    pub const LIME_300: Color = Color::rgb(190, 242, 100);
    pub const LIME_400: Color = Color::rgb(163, 230, 53);
    pub const LIME_500: Color = Color::rgb(132, 204, 22);
    pub const LIME_600: Color = Color::rgb(101, 163, 13);
    pub const LIME_700: Color = Color::rgb(77, 124, 15);
    pub const LIME_800: Color = Color::rgb(63, 98, 18);
    pub const LIME_900: Color = Color::rgb(54, 83, 20);
    pub const LIME_950: Color = Color::rgb(26, 46, 5);

    // ========================================================================
    // Green (绿色)
    // ========================================================================
    pub const GREEN_50: Color = Color::rgb(240, 253, 244);
    pub const GREEN_100: Color = Color::rgb(220, 252, 231);
    pub const GREEN_200: Color = Color::rgb(187, 247, 208);
    pub const GREEN_300: Color = Color::rgb(134, 239, 172);
    pub const GREEN_400: Color = Color::rgb(74, 222, 128);
    pub const GREEN_500: Color = Color::rgb(34, 197, 94);
    pub const GREEN_600: Color = Color::rgb(22, 163, 74);
    pub const GREEN_700: Color = Color::rgb(21, 128, 61);
    pub const GREEN_800: Color = Color::rgb(22, 101, 52);
    pub const GREEN_900: Color = Color::rgb(20, 83, 45);
    pub const GREEN_950: Color = Color::rgb(5, 46, 22);

    // ========================================================================
    // Emerald (祖母绿)
    // ========================================================================
    pub const EMERALD_50: Color = Color::rgb(236, 253, 245);
    pub const EMERALD_100: Color = Color::rgb(209, 250, 229);
    pub const EMERALD_200: Color = Color::rgb(167, 243, 208);
    pub const EMERALD_300: Color = Color::rgb(110, 231, 183);
    pub const EMERALD_400: Color = Color::rgb(52, 211, 153);
    pub const EMERALD_500: Color = Color::rgb(16, 185, 129);
    pub const EMERALD_600: Color = Color::rgb(5, 150, 105);
    pub const EMERALD_700: Color = Color::rgb(4, 120, 87);
    pub const EMERALD_800: Color = Color::rgb(6, 95, 70);
    pub const EMERALD_900: Color = Color::rgb(6, 78, 59);
    pub const EMERALD_950: Color = Color::rgb(2, 44, 34);

    // ========================================================================
    // Teal (蓝绿色)
    // ========================================================================
    pub const TEAL_50: Color = Color::rgb(240, 253, 250);
    pub const TEAL_100: Color = Color::rgb(204, 251, 241);
    pub const TEAL_200: Color = Color::rgb(153, 246, 228);
    pub const TEAL_300: Color = Color::rgb(94, 234, 212);
    pub const TEAL_400: Color = Color::rgb(45, 212, 191);
    pub const TEAL_500: Color = Color::rgb(20, 184, 166);
    pub const TEAL_600: Color = Color::rgb(13, 148, 136);
    pub const TEAL_700: Color = Color::rgb(15, 118, 110);
    pub const TEAL_800: Color = Color::rgb(17, 94, 89);
    pub const TEAL_900: Color = Color::rgb(19, 78, 74);
    pub const TEAL_950: Color = Color::rgb(4, 47, 46);

    // ========================================================================
    // Cyan (青色)
    // ========================================================================
    pub const CYAN_50: Color = Color::rgb(236, 254, 255);
    pub const CYAN_100: Color = Color::rgb(207, 250, 254);
    pub const CYAN_200: Color = Color::rgb(165, 243, 252);
    pub const CYAN_300: Color = Color::rgb(103, 232, 249);
    pub const CYAN_400: Color = Color::rgb(34, 211, 238);
    pub const CYAN_500: Color = Color::rgb(6, 182, 212);
    pub const CYAN_600: Color = Color::rgb(8, 145, 178);
    pub const CYAN_700: Color = Color::rgb(14, 116, 144);
    pub const CYAN_800: Color = Color::rgb(21, 94, 117);
    pub const CYAN_900: Color = Color::rgb(22, 78, 99);
    pub const CYAN_950: Color = Color::rgb(8, 51, 68);

    // ========================================================================
    // Sky (天蓝)
    // ========================================================================
    pub const SKY_50: Color = Color::rgb(240, 249, 255);
    pub const SKY_100: Color = Color::rgb(224, 242, 254);
    pub const SKY_200: Color = Color::rgb(186, 230, 253);
    pub const SKY_300: Color = Color::rgb(125, 211, 252);
    pub const SKY_400: Color = Color::rgb(56, 189, 248);
    pub const SKY_500: Color = Color::rgb(14, 165, 233);
    pub const SKY_600: Color = Color::rgb(2, 132, 199);
    pub const SKY_700: Color = Color::rgb(3, 105, 161);
    pub const SKY_800: Color = Color::rgb(7, 89, 133);
    pub const SKY_900: Color = Color::rgb(12, 74, 110);
    pub const SKY_950: Color = Color::rgb(8, 47, 73);

    // ========================================================================
    // Blue (蓝色)
    // ========================================================================
    pub const BLUE_50: Color = Color::rgb(239, 246, 255);
    pub const BLUE_100: Color = Color::rgb(219, 234, 254);
    pub const BLUE_200: Color = Color::rgb(191, 219, 254);
    pub const BLUE_300: Color = Color::rgb(147, 197, 253);
    pub const BLUE_400: Color = Color::rgb(96, 165, 250);
    pub const BLUE_500: Color = Color::rgb(59, 130, 246);
    pub const BLUE_600: Color = Color::rgb(37, 99, 235);
    pub const BLUE_700: Color = Color::rgb(29, 78, 216);
    pub const BLUE_800: Color = Color::rgb(30, 64, 175);
    pub const BLUE_900: Color = Color::rgb(30, 58, 138);
    pub const BLUE_950: Color = Color::rgb(23, 37, 84);

    // ========================================================================
    // Indigo (靛青)
    // ========================================================================
    pub const INDIGO_50: Color = Color::rgb(238, 242, 255);
    pub const INDIGO_100: Color = Color::rgb(224, 231, 255);
    pub const INDIGO_200: Color = Color::rgb(199, 210, 254);
    pub const INDIGO_300: Color = Color::rgb(165, 180, 252);
    pub const INDIGO_400: Color = Color::rgb(129, 140, 248);
    pub const INDIGO_500: Color = Color::rgb(99, 102, 241);
    pub const INDIGO_600: Color = Color::rgb(79, 70, 229);
    pub const INDIGO_700: Color = Color::rgb(67, 56, 202);
    pub const INDIGO_800: Color = Color::rgb(55, 48, 163);
    pub const INDIGO_900: Color = Color::rgb(49, 46, 129);
    pub const INDIGO_950: Color = Color::rgb(30, 27, 75);

    // ========================================================================
    // Violet (紫罗兰)
    // ========================================================================
    pub const VIOLET_50: Color = Color::rgb(245, 243, 255);
    pub const VIOLET_100: Color = Color::rgb(237, 233, 254);
    pub const VIOLET_200: Color = Color::rgb(221, 214, 254);
    pub const VIOLET_300: Color = Color::rgb(196, 181, 253);
    pub const VIOLET_400: Color = Color::rgb(167, 139, 250);
    pub const VIOLET_500: Color = Color::rgb(139, 92, 246);
    pub const VIOLET_600: Color = Color::rgb(124, 58, 237);
    pub const VIOLET_700: Color = Color::rgb(109, 40, 217);
    pub const VIOLET_800: Color = Color::rgb(91, 33, 182);
    pub const VIOLET_900: Color = Color::rgb(76, 29, 149);
    pub const VIOLET_950: Color = Color::rgb(46, 16, 101);

    // ========================================================================
    // Purple (紫色)
    // ========================================================================
    pub const PURPLE_50: Color = Color::rgb(250, 245, 255);
    pub const PURPLE_100: Color = Color::rgb(243, 232, 255);
    pub const PURPLE_200: Color = Color::rgb(233, 213, 255);
    pub const PURPLE_300: Color = Color::rgb(216, 180, 254);
    pub const PURPLE_400: Color = Color::rgb(192, 132, 252);
    pub const PURPLE_500: Color = Color::rgb(168, 85, 247);
    pub const PURPLE_600: Color = Color::rgb(147, 51, 234);
    pub const PURPLE_700: Color = Color::rgb(126, 34, 206);
    pub const PURPLE_800: Color = Color::rgb(107, 33, 168);
    pub const PURPLE_900: Color = Color::rgb(88, 28, 135);
    pub const PURPLE_950: Color = Color::rgb(59, 7, 100);

    // ========================================================================
    // Fuchsia (紫红色/洋红)
    // ========================================================================
    pub const FUCHSIA_50: Color = Color::rgb(253, 244, 255);
    pub const FUCHSIA_100: Color = Color::rgb(250, 232, 255);
    pub const FUCHSIA_200: Color = Color::rgb(245, 208, 254);
    pub const FUCHSIA_300: Color = Color::rgb(240, 171, 252);
    pub const FUCHSIA_400: Color = Color::rgb(232, 121, 249);
    pub const FUCHSIA_500: Color = Color::rgb(217, 70, 239);
    pub const FUCHSIA_600: Color = Color::rgb(192, 38, 211);
    pub const FUCHSIA_700: Color = Color::rgb(162, 28, 175);
    pub const FUCHSIA_800: Color = Color::rgb(134, 25, 143);
    pub const FUCHSIA_900: Color = Color::rgb(112, 26, 117);
    pub const FUCHSIA_950: Color = Color::rgb(74, 4, 78);

    // ========================================================================
    // Pink (粉色)
    // ========================================================================
    pub const PINK_50: Color = Color::rgb(253, 242, 248);
    pub const PINK_100: Color = Color::rgb(252, 231, 243);
    pub const PINK_200: Color = Color::rgb(251, 204, 231);
    pub const PINK_300: Color = Color::rgb(249, 168, 212);
    pub const PINK_400: Color = Color::rgb(244, 114, 182);
    pub const PINK_500: Color = Color::rgb(236, 72, 153);
    pub const PINK_600: Color = Color::rgb(219, 39, 119);
    pub const PINK_700: Color = Color::rgb(190, 24, 93);
    pub const PINK_800: Color = Color::rgb(157, 23, 77);
    pub const PINK_900: Color = Color::rgb(131, 24, 67);
    pub const PINK_950: Color = Color::rgb(80, 7, 36);

    // ========================================================================
    // Rose (玫瑰红)
    // ========================================================================
    pub const ROSE_50: Color = Color::rgb(255, 241, 242);
    pub const ROSE_100: Color = Color::rgb(255, 228, 230);
    pub const ROSE_200: Color = Color::rgb(254, 205, 211);
    pub const ROSE_300: Color = Color::rgb(253, 164, 175);
    pub const ROSE_400: Color = Color::rgb(251, 113, 133);
    pub const ROSE_500: Color = Color::rgb(244, 63, 94);
    pub const ROSE_600: Color = Color::rgb(225, 29, 72);
    pub const ROSE_700: Color = Color::rgb(190, 18, 60);
    pub const ROSE_800: Color = Color::rgb(159, 18, 57);
    pub const ROSE_900: Color = Color::rgb(136, 19, 55);
    pub const ROSE_950: Color = Color::rgb(76, 5, 25);
}
