#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    #[inline]
    pub const fn to_u32_opaque(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub primary: Color,
    pub on_primary: Color,
    pub cta_bg: Color,
    pub cta_hover: Color,
    pub cta_fg: Color,
    pub brand_green: Color,
    pub canvas: Color,
    pub strip: Color,
    pub surface: Color,
    pub surface_soft: Color,
    pub tab_hover: Color,
    pub btn_hover: Color,
    pub omnibox_bg: Color,
    pub hairline: Color,
    pub hairline_strong: Color,
    pub ink: Color,
    pub charcoal: Color,
    pub slate: Color,
    pub steel: Color,
    pub muted: Color,
    pub star_active: Color,
}

impl ThemePalette {
    /// Zed Paper Blueprint Grid (Crisp stark white & black 0px architectural boxes).
    pub const fn light() -> Self {
        Self {
            primary: Color::rgb(0x5b, 0x46, 0xe8),
            on_primary: Color::rgb(0xff, 0xff, 0xff),
            cta_bg: Color::rgb(0x09, 0x09, 0x0b),
            cta_hover: Color::rgb(0x27, 0x27, 0x2a),
            cta_fg: Color::rgb(0xff, 0xff, 0xff),
            brand_green: Color::rgb(0x16, 0xa3, 0x4a),
            canvas: Color::rgb(0xff, 0xff, 0xff),
            strip: Color::rgb(0xf4, 0xf4, 0xf5),
            surface: Color::rgb(0xfa, 0xfa, 0xfa),
            surface_soft: Color::rgb(0xf4, 0xf4, 0xf5),
            tab_hover: Color::rgba(0x09, 0x09, 0x0b, 14),
            btn_hover: Color::rgba(0x09, 0x09, 0x0b, 18),
            omnibox_bg: Color::rgb(0xf4, 0xf4, 0xf5),
            hairline: Color::rgb(0xd4, 0xd4, 0xd8),
            hairline_strong: Color::rgb(0x09, 0x09, 0x0b),
            ink: Color::rgb(0x09, 0x09, 0x0b),
            charcoal: Color::rgb(0x18, 0x18, 0x1b),
            slate: Color::rgb(0x52, 0x52, 0x5b),
            steel: Color::rgb(0x71, 0x71, 0x7a),
            muted: Color::rgb(0xa1, 0xa1, 0xaa),
            star_active: Color::rgb(0xd9, 0x77, 0x06),
        }
    }

    /// Zed Obsidian Grid (Default pure-black brutalist boxes with crisp white/zinc lines).
    pub const fn dark() -> Self {
        Self {
            primary: Color::rgb(0x8b, 0x6e, 0xff),
            on_primary: Color::rgb(0xff, 0xff, 0xff),
            cta_bg: Color::rgb(0xff, 0xff, 0xff),
            cta_hover: Color::rgb(0xe4, 0xe4, 0xe7),
            cta_fg: Color::rgb(0x08, 0x08, 0x0a),
            brand_green: Color::rgb(0x4a, 0xde, 0x80),
            canvas: Color::rgb(0x08, 0x08, 0x0a),
            strip: Color::rgb(0x04, 0x04, 0x05),
            surface: Color::rgb(0x0f, 0x0f, 0x13),
            surface_soft: Color::rgb(0x14, 0x14, 0x19),
            tab_hover: Color::rgb(0x16, 0x16, 0x1c),
            btn_hover: Color::rgb(0x1d, 0x1d, 0x24),
            omnibox_bg: Color::rgb(0x0c, 0x0c, 0x0f),
            hairline: Color::rgb(0x2c, 0x2c, 0x35),
            hairline_strong: Color::rgb(0xf4, 0xf4, 0xf5),
            ink: Color::rgb(0xfa, 0xfa, 0xfa),
            charcoal: Color::rgb(0xe4, 0xe4, 0xe7),
            slate: Color::rgb(0xa1, 0xa1, 0xaa),
            steel: Color::rgb(0x71, 0x71, 0x7a),
            muted: Color::rgb(0x52, 0x52, 0x5b),
            star_active: Color::rgb(0xf5, 0xd7, 0x5e),
        }
    }
}
