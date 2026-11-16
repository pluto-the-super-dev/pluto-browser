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

    pub const fn with_alpha(self, a: u8) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }

    #[inline]
    pub const fn to_u32_opaque(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    #[inline]
    pub fn lerp(self, other: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        let inv = 1.0 - t;
        Color {
            r: (self.r as f32 * inv + other.r as f32 * t + 0.5) as u8,
            g: (self.g as f32 * inv + other.g as f32 * t + 0.5) as u8,
            b: (self.b as f32 * inv + other.b as f32 * t + 0.5) as u8,
            a: (self.a as f32 * inv + other.a as f32 * t + 0.5) as u8,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub primary: Color,
    pub primary_pressed: Color,
    pub on_primary: Color,
    pub brand_navy: Color,
    pub brand_purple_soft: Color,
    pub brand_green: Color,
    pub canvas: Color,
    pub strip: Color,
    pub surface: Color,
    pub surface_soft: Color,
    pub tab_hover: Color,
    pub btn_hover: Color,
    pub btn_active: Color,
    pub omnibox_bg: Color,
    pub hairline: Color,
    pub hairline_strong: Color,
    pub ink_deep: Color,
    pub ink: Color,
    pub charcoal: Color,
    pub slate: Color,
    pub steel: Color,
    pub muted: Color,
    pub tint_peach: Color,
    pub tint_peach_ink: Color,
    pub tint_mint: Color,
    pub tint_mint_ink: Color,
    pub tint_lavender: Color,
    pub tint_lavender_ink: Color,
    pub tint_sky: Color,
    pub tint_sky_ink: Color,
    pub tint_cream: Color,
    pub tint_gray: Color,
    pub star_active: Color,
}

impl ThemePalette {
    pub const fn light() -> Self {
        Self {
            primary: Color::rgb(0x56, 0x45, 0xd4),
            primary_pressed: Color::rgb(0x45, 0x34, 0xb3),
            on_primary: Color::rgb(0xff, 0xff, 0xff),
            brand_navy: Color::rgb(0x0a, 0x15, 0x30),
            brand_purple_soft: Color::rgb(0xd6, 0xb6, 0xf6),
            brand_green: Color::rgb(0x1a, 0xae, 0x39),
            canvas: Color::rgb(0xff, 0xff, 0xff),
            strip: Color::rgb(0xf6, 0xf5, 0xf4),
            surface: Color::rgb(0xf6, 0xf5, 0xf4),
            surface_soft: Color::rgb(0xfa, 0xfa, 0xf9),
            tab_hover: Color::rgba(0x37, 0x35, 0x2f, 16),
            btn_hover: Color::rgba(0x37, 0x35, 0x2f, 18),
            btn_active: Color::rgba(0x37, 0x35, 0x2f, 32),
            omnibox_bg: Color::rgb(0xf6, 0xf5, 0xf4),
            hairline: Color::rgb(0xe5, 0xe3, 0xdf),
            hairline_strong: Color::rgb(0xc8, 0xc4, 0xbe),
            ink_deep: Color::rgb(0x00, 0x00, 0x00),
            ink: Color::rgb(0x1a, 0x1a, 0x1a),
            charcoal: Color::rgb(0x37, 0x35, 0x2f),
            slate: Color::rgb(0x5d, 0x5b, 0x54),
            steel: Color::rgb(0x78, 0x76, 0x71),
            muted: Color::rgb(0xbb, 0xb8, 0xb1),
            tint_peach: Color::rgb(0xff, 0xe8, 0xd4),
            tint_peach_ink: Color::rgb(0x79, 0x34, 0x00),
            tint_mint: Color::rgb(0xd9, 0xf3, 0xe1),
            tint_mint_ink: Color::rgb(0x1a, 0xae, 0x39),
            tint_lavender: Color::rgb(0xe6, 0xe0, 0xf5),
            tint_lavender_ink: Color::rgb(0x39, 0x1c, 0x57),
            tint_sky: Color::rgb(0xdc, 0xec, 0xfa),
            tint_sky_ink: Color::rgb(0x00, 0x5b, 0xab),
            tint_cream: Color::rgb(0xf8, 0xf5, 0xe8),
            tint_gray: Color::rgb(0xf0, 0xee, 0xec),
            star_active: Color::rgb(0xdd, 0x5b, 0x00),
        }
    }

    pub const fn dark() -> Self {
        Self {
            primary: Color::rgb(0x72, 0x64, 0xec),
            primary_pressed: Color::rgb(0x5d, 0x4e, 0xe0),
            on_primary: Color::rgb(0xff, 0xff, 0xff),
            brand_navy: Color::rgb(0x0a, 0x15, 0x30),
            brand_purple_soft: Color::rgb(0xd6, 0xb6, 0xf6),
            brand_green: Color::rgb(0x4c, 0xd4, 0x68),
            canvas: Color::rgb(0x19, 0x19, 0x19),
            strip: Color::rgb(0x14, 0x14, 0x14),
            surface: Color::rgb(0x21, 0x21, 0x21),
            surface_soft: Color::rgb(0x26, 0x26, 0x26),
            tab_hover: Color::rgba(0xff, 0xff, 0xff, 16),
            btn_hover: Color::rgba(0xff, 0xff, 0xff, 22),
            btn_active: Color::rgba(0xff, 0xff, 0xff, 36),
            omnibox_bg: Color::rgb(0x15, 0x15, 0x15),
            hairline: Color::rgb(0x2e, 0x2e, 0x2e),
            hairline_strong: Color::rgb(0x44, 0x44, 0x44),
            ink_deep: Color::rgb(0x56, 0x45, 0xd4),
            ink: Color::rgb(0xf4, 0xf4, 0xf2),
            charcoal: Color::rgb(0xe5, 0xe3, 0xdf),
            slate: Color::rgb(0xb5, 0xb2, 0xab),
            steel: Color::rgb(0x8f, 0x8c, 0x85),
            muted: Color::rgb(0x68, 0x65, 0x5e),
            tint_peach: Color::rgb(0x38, 0x25, 0x15),
            tint_peach_ink: Color::rgb(0xff, 0xb4, 0x78),
            tint_mint: Color::rgb(0x16, 0x33, 0x22),
            tint_mint_ink: Color::rgb(0x4c, 0xd4, 0x68),
            tint_lavender: Color::rgb(0x2b, 0x23, 0x44),
            tint_lavender_ink: Color::rgb(0xd6, 0xb6, 0xf6),
            tint_sky: Color::rgb(0x17, 0x2b, 0x3d),
            tint_sky_ink: Color::rgb(0x85, 0xc3, 0xff),
            tint_cream: Color::rgb(0x26, 0x23, 0x1d),
            tint_gray: Color::rgb(0x2a, 0x2a, 0x2a),
            star_active: Color::rgb(0xf5, 0xd7, 0x5e),
        }
    }
}

