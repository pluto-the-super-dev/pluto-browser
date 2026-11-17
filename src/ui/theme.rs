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
    pub brand_amber: Color,
    pub tint_sky: Color,
    pub tint_mint: Color,
    pub tint_sand: Color,
    pub tint_rose: Color,
    pub canvas: Color,
    pub strip: Color,
    pub surface: Color,
    pub surface_soft: Color,
    pub tab_hover: Color,
    pub tab_active: Color,
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
    /// Clean Modern Apple + Notion Daylight (Warm Alabaster, Crisp Ink, Zero Purple).
    pub const fn light() -> Self {
        Self {
            primary: Color::rgb(0x00, 0x71, 0xe3),
            on_primary: Color::rgb(0xff, 0xff, 0xff),
            cta_bg: Color::rgb(0x1d, 0x1d, 0x1f),
            cta_hover: Color::rgb(0x3a, 0x3a, 0x3c),
            cta_fg: Color::rgb(0xff, 0xff, 0xff),
            brand_green: Color::rgb(0x1a, 0x9c, 0x46),
            brand_amber: Color::rgb(0xd9, 0x77, 0x06),
            tint_sky: Color::rgb(0xe8, 0xf3, 0xfc),
            tint_mint: Color::rgb(0xe8, 0xf6, 0xed),
            tint_sand: Color::rgb(0xfd, 0xf4, 0xdf),
            tint_rose: Color::rgb(0xfc, 0xec, 0xf1),
            canvas: Color::rgb(0xff, 0xff, 0xff),
            strip: Color::rgb(0xf5, 0xf5, 0xf4),
            surface: Color::rgb(0xfa, 0xfa, 0xf9),
            surface_soft: Color::rgb(0xef, 0xee, 0xec),
            tab_hover: Color::rgba(0x1d, 0x1d, 0x1f, 10),
            tab_active: Color::rgb(0xff, 0xff, 0xff),
            btn_hover: Color::rgba(0x1d, 0x1d, 0x1f, 14),
            omnibox_bg: Color::rgb(0xf4, 0xf4, 0xf3),
            hairline: Color::rgb(0xe6, 0xe5, 0xe3),
            hairline_strong: Color::rgb(0xc9, 0xc8, 0xc4),
            ink: Color::rgb(0x1d, 0x1d, 0x1f),
            charcoal: Color::rgb(0x37, 0x35, 0x2f),
            slate: Color::rgb(0x5d, 0x5b, 0x54),
            steel: Color::rgb(0x78, 0x76, 0x71),
            muted: Color::rgb(0xa4, 0xa0, 0x97),
            star_active: Color::rgb(0xea, 0xb3, 0x08),
        }
    }

    /// Apple Cupertino Midnight (Refined Slate Graphite Dark, Crisp White, Zero Purple).
    pub const fn dark() -> Self {
        Self {
            primary: Color::rgb(0x0a, 0x84, 0xff),
            on_primary: Color::rgb(0xff, 0xff, 0xff),
            cta_bg: Color::rgb(0xf5, 0xf5, 0xf7),
            cta_hover: Color::rgb(0xe5, 0xe5, 0xea),
            cta_fg: Color::rgb(0x16, 0x16, 0x18),
            brand_green: Color::rgb(0x30, 0xd1, 0x58),
            brand_amber: Color::rgb(0xfb, 0xbf, 0x24),
            tint_sky: Color::rgb(0x18, 0x27, 0x38),
            tint_mint: Color::rgb(0x16, 0x29, 0x20),
            tint_sand: Color::rgb(0x2b, 0x24, 0x16),
            tint_rose: Color::rgb(0x2d, 0x1d, 0x23),
            canvas: Color::rgb(0x16, 0x16, 0x18),
            strip: Color::rgb(0x11, 0x11, 0x13),
            surface: Color::rgb(0x1f, 0x1f, 0x22),
            surface_soft: Color::rgb(0x2a, 0x2a, 0x2e),
            tab_hover: Color::rgb(0x1e, 0x1e, 0x21),
            tab_active: Color::rgb(0x26, 0x26, 0x2a),
            btn_hover: Color::rgb(0x29, 0x29, 0x2d),
            omnibox_bg: Color::rgb(0x21, 0x21, 0x24),
            hairline: Color::rgb(0x2d, 0x2d, 0x31),
            hairline_strong: Color::rgb(0x48, 0x48, 0x4e),
            ink: Color::rgb(0xf5, 0xf5, 0xf7),
            charcoal: Color::rgb(0xe5, 0xe5, 0xea),
            slate: Color::rgb(0xa1, 0xa1, 0xa6),
            steel: Color::rgb(0x8e, 0x8e, 0x93),
            muted: Color::rgb(0x63, 0x63, 0x66),
            star_active: Color::rgb(0xff, 0xd6, 0x0a),
        }
    }
}
