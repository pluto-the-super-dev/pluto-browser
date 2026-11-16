use crate::ui::theme::Color;
use ab_glyph::{point, Font, FontArc, PxScale, ScaleFont};
use std::collections::HashMap;
use std::fs;

const FONT_SCALE: f32 = 1.38;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontStyle {
    Regular,
    SemiBold,
    Bold,
    Mono,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GlyphKey {
    style: FontStyle,
    ch: char,
    px_tenths: u16,
}

struct CachedGlyph {
    min_x: i32,
    min_y: i32,
    width: usize,
    height: usize,
    alpha: Vec<u8>,
    advance: f32,
}

pub struct FontSystem {
    regular: Option<FontArc>,
    semibold: Option<FontArc>,
    bold: Option<FontArc>,
    mono: Option<FontArc>,
    cache: HashMap<GlyphKey, CachedGlyph>,
}

impl FontSystem {
    pub fn load_system() -> Self {
        let regular = load_first_font(&[
            r"C:\Windows\Fonts\segoeui.ttf",
            r"C:\Windows\Fonts\arial.ttf",
            "/System/Library/Fonts/SFNS.ttf",
            "/System/Library/Fonts/Supplemental/Arial.ttf",
            "/usr/share/fonts/truetype/inter/Inter-Regular.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        ]);

        let semibold = load_first_font(&[
            r"C:\Windows\Fonts\seguisb.ttf",
            r"C:\Windows\Fonts\segoeuib.ttf",
            r"C:\Windows\Fonts\arialbd.ttf",
            "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
            "/usr/share/fonts/truetype/inter/Inter-SemiBold.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
            "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
        ]);

        let bold = load_first_font(&[
            r"C:\Windows\Fonts\segoeuib.ttf",
            r"C:\Windows\Fonts\arialbd.ttf",
            "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
            "/usr/share/fonts/truetype/inter/Inter-Bold.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
            "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
        ]);

        let mono = load_first_font(&[
            r"C:\Windows\Fonts\consola.ttf",
            r"C:\Windows\Fonts\lucon.ttf",
            "/System/Library/Fonts/Menlo.ttc",
            "/System/Library/Fonts/Supplemental/Courier New.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
            "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
        ]);

        Self {
            regular,
            semibold,
            bold,
            mono,
            cache: HashMap::with_capacity(512),
        }
    }

    fn font_for(&self, style: FontStyle) -> Option<&FontArc> {
        match style {
            FontStyle::Regular => self.regular.as_ref(),
            FontStyle::SemiBold => self.semibold.as_ref().or(self.bold.as_ref()).or(self.regular.as_ref()),
            FontStyle::Bold => self.bold.as_ref().or(self.semibold.as_ref()).or(self.regular.as_ref()),
            FontStyle::Mono => self.mono.as_ref().or(self.regular.as_ref()),
        }
    }

    pub fn measure_text(&self, style: FontStyle, px_size: f32, text: &str) -> f32 {
        let eff_px = px_size * FONT_SCALE;
        let Some(font) = self.font_for(style) else {
            return text.chars().count() as f32 * (eff_px * 0.54);
        };
        let scaled = font.as_scaled(PxScale::from(eff_px));
        let mut width = 0.0_f32;
        let mut prev_id = None;
        for ch in text.chars() {
            let gid = font.glyph_id(ch);
            if let Some(prev) = prev_id {
                width += scaled.kern(prev, gid);
            }
            width += scaled.h_advance(gid);
            prev_id = Some(gid);
        }
        width
    }

    fn get_or_rasterize(&mut self, style: FontStyle, ch: char, phys_px: f32) -> Option<&CachedGlyph> {
        let px_tenths = (phys_px.clamp(6.0, 280.0) * 10.0).round() as u16;
        let key = GlyphKey {
            style,
            ch,
            px_tenths,
        };
        if !self.cache.contains_key(&key) {
            let font = self.font_for(style)?;
            let scale = PxScale::from(px_tenths as f32 / 10.0);
            let scaled = font.as_scaled(scale);
            let gid = font.glyph_id(ch);
            let advance = scaled.h_advance(gid);
            let glyph = gid.with_scale_and_position(scale, point(0.0, scaled.ascent()));
            let built = if let Some(outlined) = font.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                let w = (bounds.width().ceil() as i32).max(0) as usize;
                let h = (bounds.height().ceil() as i32).max(0) as usize;
                let mut alpha = vec![0u8; w * h];
                if w > 0 && h > 0 {
                    outlined.draw(|gx, gy, c| {
                        let idx = (gy as usize) * w + (gx as usize);
                        if idx < alpha.len() {
                            alpha[idx] = (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                        }
                    });
                }
                CachedGlyph {
                    min_x: bounds.min.x.floor() as i32,
                    min_y: bounds.min.y.floor() as i32,
                    width: w,
                    height: h,
                    alpha,
                    advance,
                }
            } else {
                CachedGlyph {
                    min_x: 0,
                    min_y: 0,
                    width: 0,
                    height: 0,
                    alpha: Vec::new(),
                    advance,
                }
            };
            if self.cache.len() > 2400 {
                self.cache.clear();
            }
            self.cache.insert(key, built);
        }
        self.cache.get(&key)
    }
}

fn load_first_font(paths: &[&str]) -> Option<FontArc> {
    for path in paths {
        if let Ok(bytes) = fs::read(path) {
            if let Ok(font) = FontArc::try_from_vec(bytes) {
                return Some(font);
            }
        }
    }
    None
}

#[derive(Debug, Clone, Copy)]
struct ClipRect {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
}

pub struct Canvas<'a> {
    buf: &'a mut [u32],
    phys_w: usize,
    phys_h: usize,
    scale: f32,
    clips: Vec<ClipRect>,
}

impl<'a> Canvas<'a> {
    pub fn new(buf: &'a mut [u32], phys_w: usize, phys_h: usize, scale: f64) -> Self {
        let initial_clip = ClipRect {
            x0: 0,
            y0: 0,
            x1: phys_w as i32,
            y1: phys_h as i32,
        };
        Self {
            buf,
            phys_w,
            phys_h,
            scale: (scale as f32).max(1.0),
            clips: vec![initial_clip],
        }
    }

    pub fn set_scale(&mut self, scale: f64) {
        self.scale = (scale as f32).max(0.4);
    }

    pub fn logical_width(&self) -> f32 {
        self.phys_w as f32 / self.scale
    }

    pub fn logical_height(&self) -> f32 {
        self.phys_h as f32 / self.scale
    }

    #[inline]
    fn clip(&self) -> ClipRect {
        *self.clips.last().unwrap_or(&ClipRect {
            x0: 0,
            y0: 0,
            x1: self.phys_w as i32,
            y1: self.phys_h as i32,
        })
    }

    pub fn push_clip(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let prev = self.clip();
        let x0 = ((x * self.scale).floor() as i32).max(prev.x0);
        let y0 = ((y * self.scale).floor() as i32).max(prev.y0);
        let x1 = (((x + w) * self.scale).ceil() as i32).min(prev.x1);
        let y1 = (((y + h) * self.scale).ceil() as i32).min(prev.y1);
        self.clips.push(ClipRect { x0, y0, x1, y1 });
    }

    pub fn pop_clip(&mut self) {
        if self.clips.len() > 1 {
            self.clips.pop();
        }
    }

    pub fn clear(&mut self, color: Color) {
        self.buf.fill(color.to_u32_opaque());
    }

    #[inline]
    fn blend_phys_px(&mut self, px: i32, py: i32, color: Color, cov: f32) {
        let alpha = (color.a as f32 / 255.0) * cov;
        if alpha <= 0.002 {
            return;
        }
        let idx = (py as usize) * self.phys_w + (px as usize);
        if idx >= self.buf.len() {
            return;
        }
        if alpha >= 0.996 {
            self.buf[idx] = color.to_u32_opaque();
            return;
        }
        let bg = self.buf[idx];
        let br = ((bg >> 16) & 0xff) as f32;
        let bg_g = ((bg >> 8) & 0xff) as f32;
        let bb = (bg & 0xff) as f32;
        let inv = 1.0 - alpha;
        let r = (color.r as f32 * alpha + br * inv + 0.5) as u32;
        let g = (color.g as f32 * alpha + bg_g * inv + 0.5) as u32;
        let b = (color.b as f32 * alpha + bb * inv + 0.5) as u32;
        self.buf[idx] = (r.min(255) << 16) | (g.min(255) << 8) | b.min(255);
    }

    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        if w <= 0.0 || h <= 0.0 || color.a == 0 {
            return;
        }
        let clip = self.clip();
        let x0 = ((x * self.scale).round() as i32).max(clip.x0);
        let y0 = ((y * self.scale).round() as i32).max(clip.y0);
        let x1 = (((x + w) * self.scale).round() as i32).min(clip.x1);
        let y1 = (((y + h) * self.scale).round() as i32).min(clip.y1);
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        if color.a == 255 {
            let val = color.to_u32_opaque();
            for py in y0..y1 {
                let row_start = (py as usize) * self.phys_w + (x0 as usize);
                let row_end = row_start + (x1 - x0) as usize;
                self.buf[row_start..row_end].fill(val);
            }
        } else {
            for py in y0..y1 {
                for px in x0..x1 {
                    self.blend_phys_px(px, py, color, 1.0);
                }
            }
        }
    }

    pub fn fill_radial_glow(&mut self, cx: f32, cy: f32, radius: f32, color: Color) {
        if radius <= 0.0 || color.a == 0 {
            return;
        }
        let clip = self.clip();
        let pcx = cx * self.scale;
        let pcy = cy * self.scale;
        let pr = radius * self.scale;
        let x0 = ((pcx - pr).floor() as i32).max(clip.x0);
        let y0 = ((pcy - pr).floor() as i32).max(clip.y0);
        let x1 = ((pcx + pr).ceil() as i32).min(clip.x1);
        let y1 = ((pcy + pr).ceil() as i32).min(clip.y1);
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        for py in y0..y1 {
            let dy = (py as f32 + 0.5) - pcy;
            for px in x0..x1 {
                let dx = (px as f32 + 0.5) - pcx;
                let dist = (dx * dx + dy * dy).sqrt() / pr;
                if dist < 1.0 {
                    let falloff = (1.0 - dist) * (1.0 - dist);
                    self.blend_phys_px(px, py, color, falloff);
                }
            }
        }
    }

    /// Zero-rounded sharp rectangular fill matching the Zed brutalist box grid.
    pub fn fill_rounded_rect(&mut self, x: f32, y: f32, w: f32, h: f32, _radius: f32, color: Color) {
        self.fill_rect(x, y, w, h, color);
    }

    /// Crisp 0px sharp architectural rectangular border (4 SIMD horizontal/vertical edges).
    pub fn stroke_rounded_rect(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        _radius: f32,
        thickness: f32,
        color: Color,
    ) {
        if w <= 0.0 || h <= 0.0 || thickness <= 0.0 || color.a == 0 {
            return;
        }
        let t = thickness.max(1.0);
        self.fill_rect(x, y, w, t, color);
        self.fill_rect(x, (y + h - t).max(y), w, t, color);
        if h > t * 2.0 {
            self.fill_rect(x, y + t, t, h - t * 2.0, color);
            self.fill_rect((x + w - t).max(x), y + t, t, h - t * 2.0, color);
        }
    }

    pub fn fill_circle(&mut self, cx: f32, cy: f32, r: f32, color: Color) {
        if r <= 0.0 || color.a == 0 {
            return;
        }
        let clip = self.clip();
        let pcx = cx * self.scale;
        let pcy = cy * self.scale;
        let pr = r * self.scale;
        let x0 = ((pcx - pr - 1.0).floor() as i32).max(clip.x0);
        let y0 = ((pcy - pr - 1.0).floor() as i32).max(clip.y0);
        let x1 = ((pcx + pr + 1.0).ceil() as i32).min(clip.x1);
        let y1 = ((pcy + pr + 1.0).ceil() as i32).min(clip.y1);
        for py in y0..y1 {
            let dy = (py as f32 + 0.5) - pcy;
            for px in x0..x1 {
                let dx = (px as f32 + 0.5) - pcx;
                let dist = (dx * dx + dy * dy).sqrt() - pr;
                let cov = (0.5 - dist).clamp(0.0, 1.0);
                if cov > 0.0 {
                    self.blend_phys_px(px, py, color, cov);
                }
            }
        }
    }

    pub fn stroke_circle(&mut self, cx: f32, cy: f32, r: f32, thickness: f32, color: Color) {
        if r <= 0.0 || thickness <= 0.0 || color.a == 0 {
            return;
        }
        let clip = self.clip();
        let pcx = cx * self.scale;
        let pcy = cy * self.scale;
        let pr = r * self.scale;
        let half_t = (thickness * self.scale * 0.5).max(0.5);
        let outer = pr + half_t + 1.0;
        let x0 = ((pcx - outer).floor() as i32).max(clip.x0);
        let y0 = ((pcy - outer).floor() as i32).max(clip.y0);
        let x1 = ((pcx + outer).ceil() as i32).min(clip.x1);
        let y1 = ((pcy + outer).ceil() as i32).min(clip.y1);
        for py in y0..y1 {
            let dy = (py as f32 + 0.5) - pcy;
            for px in x0..x1 {
                let dx = (px as f32 + 0.5) - pcx;
                let dist = ((dx * dx + dy * dy).sqrt() - pr).abs() - half_t;
                let cov = (0.5 - dist).clamp(0.0, 1.0);
                if cov > 0.0 {
                    self.blend_phys_px(px, py, color, cov);
                }
            }
        }
    }

    pub fn draw_line_aa(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        thickness: f32,
        color: Color,
    ) {
        let clip = self.clip();
        let ax = x0 * self.scale;
        let ay = y0 * self.scale;
        let bx = x1 * self.scale;
        let by = y1 * self.scale;
        let half_t = (thickness * self.scale * 0.5).max(0.55);
        let pad = half_t + 1.5;

        let min_x = (ax.min(bx) - pad).floor() as i32;
        let max_x = (ax.max(bx) + pad).ceil() as i32;
        let min_y = (ay.min(by) - pad).floor() as i32;
        let max_y = (ay.max(by) + pad).ceil() as i32;

        let sx0 = min_x.max(clip.x0);
        let sy0 = min_y.max(clip.y0);
        let sx1 = max_x.min(clip.x1);
        let sy1 = max_y.min(clip.y1);

        let abx = bx - ax;
        let aby = by - ay;
        let len_sq = abx * abx + aby * aby;

        for py in sy0..sy1 {
            let fy = py as f32 + 0.5;
            for px in sx0..sx1 {
                let fx = px as f32 + 0.5;
                let dist = if len_sq <= f32::EPSILON {
                    ((fx - ax).powi(2) + (fy - ay).powi(2)).sqrt()
                } else {
                    let t = (((fx - ax) * abx + (fy - ay) * aby) / len_sq).clamp(0.0, 1.0);
                    let cx = ax + t * abx;
                    let cy = ay + t * aby;
                    ((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt()
                };
                let cov = (0.5 - (dist - half_t)).clamp(0.0, 1.0);
                if cov > 0.0 {
                    self.blend_phys_px(px, py, color, cov);
                }
            }
        }
    }

    pub fn draw_text(
        &mut self,
        fonts: &mut FontSystem,
        style: FontStyle,
        px_size: f32,
        x: f32,
        y: f32,
        color: Color,
        text: &str,
    ) {
        if text.is_empty() || color.a == 0 {
            return;
        }
        let phys_px = px_size * FONT_SCALE * self.scale;
        let Some(font_clone) = fonts.font_for(style).cloned() else {
            self.draw_fallback_text(x, y, px_size, color, text);
            return;
        };
        let scaled = font_clone.as_scaled(PxScale::from(phys_px));
        let clip = self.clip();

        let mut cursor_x = x * self.scale;
        let origin_y = (y * self.scale).round() as i32;
        let mut prev_id = None;

        for ch in text.chars() {
            let gid = font_clone.glyph_id(ch);
            if let Some(prev) = prev_id {
                cursor_x += scaled.kern(prev, gid);
            }
            prev_id = Some(gid);

            let base_x = cursor_x.round() as i32;
            let Some(cached) = fonts.get_or_rasterize(style, ch, phys_px) else {
                continue;
            };
            let gx0 = base_x + cached.min_x;
            let gy0 = origin_y + cached.min_y;
            let w = cached.width;
            let h = cached.height;
            let adv = cached.advance;

            if w > 0 && h > 0 {
                let start_ry = (clip.y0 - gy0).max(0) as usize;
                let end_ry = ((clip.y1 - gy0).max(0) as usize).min(h);
                let start_rx = (clip.x0 - gx0).max(0) as usize;
                let end_rx = ((clip.x1 - gx0).max(0) as usize).min(w);

                let base_a = color.a as f32 / 255.0;
                for ry in start_ry..end_ry {
                    let py = gy0 + ry as i32;
                    let row_offset = ry * w;
                    for rx in start_rx..end_rx {
                        let a_byte = cached.alpha[row_offset + rx];
                        if a_byte == 0 {
                            continue;
                        }
                        let cov = (a_byte as f32 / 255.0) * base_a;
                        let px = gx0 + rx as i32;
                        let idx = (py as usize) * self.phys_w + (px as usize);
                        let bg = self.buf[idx];
                        let br = ((bg >> 16) & 0xff) as f32;
                        let bg_g = ((bg >> 8) & 0xff) as f32;
                        let bb = (bg & 0xff) as f32;
                        let inv = 1.0 - cov;
                        let r = (color.r as f32 * cov + br * inv + 0.5) as u32;
                        let g = (color.g as f32 * cov + bg_g * inv + 0.5) as u32;
                        let b = (color.b as f32 * cov + bb * inv + 0.5) as u32;
                        self.buf[idx] = (r.min(255) << 16) | (g.min(255) << 8) | b.min(255);
                    }
                }
            }
            cursor_x += adv;
        }
    }

    pub fn draw_text_ellipsis(
        &mut self,
        fonts: &mut FontSystem,
        style: FontStyle,
        px_size: f32,
        x: f32,
        y: f32,
        max_w: f32,
        color: Color,
        text: &str,
    ) {
        if max_w <= 6.0 || text.is_empty() {
            return;
        }
        if fonts.measure_text(style, px_size, text) <= max_w {
            self.draw_text(fonts, style, px_size, x, y, color, text);
            return;
        }
        let ell = "...";
        let ell_w = fonts.measure_text(style, px_size, ell);
        let mut out = String::new();
        for ch in text.chars() {
            out.push(ch);
            if fonts.measure_text(style, px_size, &out) + ell_w > max_w {
                out.pop();
                break;
            }
        }
        out.push_str(ell);
        self.draw_text(fonts, style, px_size, x, y, color, &out);
    }

    fn draw_fallback_text(&mut self, x: f32, y: f32, px_size: f32, color: Color, text: &str) {
        let cw = (px_size * 0.52).max(5.0);
        let ch = px_size * 0.72;
        for (i, c) in text.chars().enumerate() {
            if c.is_whitespace() {
                continue;
            }
            let cx = x + i as f32 * cw;
            self.fill_rounded_rect(cx, y + px_size * 0.16, cw - 1.5, ch, 1.5, color);
        }
    }

    // -------------------------------------------------------------------------
    // Vector Icons & Planet Pluto Brand Artwork (100% Pure Rust Math)
    // -------------------------------------------------------------------------

    pub fn draw_pluto_logo(&mut self, x: f32, y: f32, size: f32, hover_t: f32) {
        self.fill_rect(x, y, size, size, Color::rgb(0x04, 0x04, 0x05));
        self.stroke_rounded_rect(
            x,
            y,
            size,
            size,
            0.0,
            1.0,
            if hover_t > 0.5 {
                Color::rgb(0xff, 0xff, 0xff)
            } else {
                Color::rgba(0xe4, 0xe4, 0xe7, 150)
            },
        );

        let cx = x + size * 0.5;
        let cy = y + size * 0.5;
        let pr = size * 0.22;
        let tilt = -0.44 + hover_t * 0.18;
        let rx = size * 0.40;
        let ry = size * 0.13;

        self.draw_tilted_ellipse_half(cx, cy, rx, ry, tilt, 1.35, Color::rgba(0x9d, 0x8d, 0xf7, 175), true);

        // Shaded Planet Pluto sphere
        let clip = self.clip();
        let pcx = cx * self.scale;
        let pcy = cy * self.scale;
        let p_rad = pr * self.scale;
        let x0 = ((pcx - p_rad - 1.5).floor() as i32).max(clip.x0);
        let y0 = ((pcy - p_rad - 1.5).floor() as i32).max(clip.y0);
        let x1 = ((pcx + p_rad + 1.5).ceil() as i32).min(clip.x1);
        let y1 = ((pcy + p_rad + 1.5).ceil() as i32).min(clip.y1);

        for py in y0..y1 {
            let dy = (py as f32 + 0.5) - pcy;
            for px in x0..x1 {
                let dx = (px as f32 + 0.5) - pcx;
                let dist = (dx * dx + dy * dy).sqrt() - p_rad;
                let cov = (0.5 - dist).clamp(0.0, 1.0);
                if cov <= 0.0 {
                    continue;
                }
                let nx = (dx / p_rad).clamp(-1.0, 1.0);
                let ny = (dy / p_rad).clamp(-1.0, 1.0);
                let nz = (1.0 - nx * nx - ny * ny).max(0.0).sqrt();
                let diffuse = (nx * -0.52 + ny * -0.48 + nz * 0.70).clamp(0.0, 1.0);
                let rim = (1.0 - nz).powi(3);
                let ice = (1.0 - ((nx - 0.16).powi(2) + (ny + 0.04).powi(2)).sqrt() * 1.9).max(0.0) * 0.5;
                let mut r = 68.0 + diffuse * 155.0 + rim * 60.0;
                let mut g = 55.0 + diffuse * 126.0 + rim * 52.0;
                let mut b = 114.0 + diffuse * 110.0 + rim * 104.0;
                r = r * (1.0 - ice) + 250.0 * ice;
                g = g * (1.0 - ice) + 240.0 * ice;
                b = b * (1.0 - ice) + 246.0 * ice;
                let col = Color::rgb(
                    r.clamp(0.0, 255.0) as u8,
                    g.clamp(0.0, 255.0) as u8,
                    b.clamp(0.0, 255.0) as u8,
                );
                self.blend_phys_px(px, py, col, cov);
            }
        }

        self.draw_tilted_ellipse_half(cx, cy, rx, ry, tilt, 1.55, Color::rgba(0xe8, 0xdb, 0xff, 240), false);

        let moon_a = -0.62 + hover_t * 0.35;
        let mx = cx + (rx * 0.92) * moon_a.cos();
        let my = cy + (rx * 0.62) * moon_a.sin();
        self.fill_circle(mx, my, size * 0.068, Color::rgb(0xeb, 0xde, 0xff));
    }

    pub fn draw_tilted_ellipse_half(
        &mut self,
        cx: f32,
        cy: f32,
        rx: f32,
        ry: f32,
        angle: f32,
        thickness: f32,
        color: Color,
        back_half: bool,
    ) {
        let clip = self.clip();
        let pcx = cx * self.scale;
        let pcy = cy * self.scale;
        let prx = (rx * self.scale).max(1.0);
        let pry = (ry * self.scale).max(1.0);
        let half_t = (thickness * self.scale * 0.5).max(0.5);
        let bound = prx.max(pry) + half_t + 2.0;

        let x0 = ((pcx - bound).floor() as i32).max(clip.x0);
        let y0 = ((pcy - bound).floor() as i32).max(clip.y0);
        let x1 = ((pcx + bound).ceil() as i32).min(clip.x1);
        let y1 = ((pcy + bound).ceil() as i32).min(clip.y1);

        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let inv_rx2 = 1.0 / (prx * prx);
        let inv_ry2 = 1.0 / (pry * pry);

        for py in y0..y1 {
            let dy = (py as f32 + 0.5) - pcy;
            for px in x0..x1 {
                let dx = (px as f32 + 0.5) - pcx;
                let u = dx * cos_a - dy * sin_a;
                let v = dx * sin_a + dy * cos_a;
                if back_half && v >= 0.0 {
                    continue;
                }
                if !back_half && v < 0.0 {
                    continue;
                }
                let f = u * u * inv_rx2 + v * v * inv_ry2;
                let grad = 2.0 * ((u * inv_rx2).powi(2) + (v * inv_ry2).powi(2)).sqrt().max(1e-4);
                let dist = ((f - 1.0) / grad).abs() - half_t;
                let cov = (0.5 - dist).clamp(0.0, 1.0);
                if cov > 0.0 {
                    self.blend_phys_px(px, py, color, cov);
                }
            }
        }
    }

    pub fn draw_pluto_hero_system(&mut self, cx: f32, cy: f32, time_sec: f32, px_off: f32, py_off: f32) {
        let bob = (time_sec * 1.45).sin() * 4.5;
        let center_x = cx + px_off;
        let center_y = cy + py_off + bob;

        self.fill_radial_glow(center_x, center_y, 76.0, Color::rgba(0x7b, 0x3f, 0xf2, 82));
        self.fill_radial_glow(center_x - 16.0, center_y - 12.0, 48.0, Color::rgba(0x56, 0x45, 0xd4, 70));

        let tilt = -0.32 + (time_sec * 0.45).sin() * 0.04;
        // Back orbital rings
        self.draw_tilted_ellipse_half(
            center_x,
            center_y,
            84.0,
            24.0,
            tilt,
            1.4,
            Color::rgba(0x7b, 0x3f, 0xf2, 130),
            true,
        );
        self.draw_tilted_ellipse_half(
            center_x,
            center_y,
            68.0,
            19.0,
            tilt,
            2.1,
            Color::rgba(0x9d, 0x8d, 0xf7, 175),
            true,
        );

        // Shaded Planet Pluto sphere + Tombaugh Regio heart basin
        let clip = self.clip();
        let pcx = center_x * self.scale;
        let pcy = center_y * self.scale;
        let p_rad = 35.0 * self.scale;
        let x0 = ((pcx - p_rad - 2.0).floor() as i32).max(clip.x0);
        let y0 = ((pcy - p_rad - 2.0).floor() as i32).max(clip.y0);
        let x1 = ((pcx + p_rad + 2.0).ceil() as i32).min(clip.x1);
        let y1 = ((pcy + p_rad + 2.0).ceil() as i32).min(clip.y1);

        let spin = (time_sec * 0.25).sin() * 0.12;
        for py in y0..y1 {
            let dy = (py as f32 + 0.5) - pcy;
            for px in x0..x1 {
                let dx = (px as f32 + 0.5) - pcx;
                let dist = (dx * dx + dy * dy).sqrt() - p_rad;
                let cov = (0.5 - dist).clamp(0.0, 1.0);
                if cov <= 0.0 {
                    continue;
                }
                let nx = (dx / p_rad).clamp(-1.0, 1.0);
                let ny = (dy / p_rad).clamp(-1.0, 1.0);
                let nz = (1.0 - nx * nx - ny * ny).max(0.0).sqrt();

                let diffuse = (nx * -0.54 + ny * -0.46 + nz * 0.70).clamp(0.0, 1.0);
                let rim = (1.0 - nz).powi(3);
                let band = ((ny * 7.0 + nx * 1.4).sin() * 0.5 + 0.5) * 14.0;

                // Subtle nitrogen-ice heart-basin Tombaugh Regio
                let hx = nx - (0.14 + spin);
                let hy = ny + 0.02;
                let heart_l = ((hx + 0.12).powi(2) + (hy + 0.06).powi(2)).sqrt();
                let heart_r = ((hx - 0.12).powi(2) + (hy + 0.06).powi(2)).sqrt();
                let heart_b = (hx.powi(2) * 0.9 + (hy - 0.12).powi(2)).sqrt();
                let tombaugh = (1.0 - (heart_l.min(heart_r).min(heart_b)) * 3.8).max(0.0) * 0.42;

                let mut r = 24.0 + diffuse * 212.0 + rim * 70.0 + band;
                let mut g = 20.0 + diffuse * 150.0 + rim * 55.0 + band * 0.6;
                let mut b = 58.0 + diffuse * 165.0 + rim * 125.0;

                r = r * (1.0 - tombaugh) + 255.0 * tombaugh;
                g = g * (1.0 - tombaugh) + 245.0 * tombaugh;
                b = b * (1.0 - tombaugh) + 236.0 * tombaugh;

                let col = Color::rgb(
                    r.clamp(0.0, 255.0) as u8,
                    g.clamp(0.0, 255.0) as u8,
                    b.clamp(0.0, 255.0) as u8,
                );
                self.blend_phys_px(px, py, col, cov);
            }
        }

        // Front orbital rings crossing over Pluto
        self.draw_tilted_ellipse_half(
            center_x,
            center_y,
            68.0,
            19.0,
            tilt,
            2.3,
            Color::rgba(0xf0, 0xe4, 0xff, 240),
            false,
        );
        self.draw_tilted_ellipse_half(
            center_x,
            center_y,
            84.0,
            24.0,
            tilt,
            1.2,
            Color::rgba(0xd6, 0xb6, 0xf6, 150),
            false,
        );

        // Orbiting Charon moon along upper-right/front path
        let orb_t = time_sec * 0.72 - 0.45;
        let ox = orb_t.cos() * 75.0;
        let oy = orb_t.sin() * 21.0;
        let cos_t = tilt.cos();
        let sin_t = tilt.sin();
        let charon_x = center_x + ox * cos_t + oy * sin_t;
        let charon_y = center_y - ox * sin_t + oy * cos_t;

        self.fill_radial_glow(charon_x, charon_y, 12.0, Color::rgba(0xd6, 0xb6, 0xf6, 95));
        self.fill_circle(charon_x, charon_y, 5.2, Color::rgb(0xf2, 0xe9, 0xff));
    }

    pub fn draw_icon_back(&mut self, cx: f32, cy: f32, color: Color) {
        self.draw_line_aa(cx + 2.5, cy - 4.5, cx - 2.5, cy, 1.7, color);
        self.draw_line_aa(cx - 2.5, cy, cx + 2.5, cy + 4.5, 1.7, color);
    }

    pub fn draw_icon_forward(&mut self, cx: f32, cy: f32, color: Color) {
        self.draw_line_aa(cx - 2.5, cy - 4.5, cx + 2.5, cy, 1.7, color);
        self.draw_line_aa(cx + 2.5, cy, cx - 2.5, cy + 4.5, 1.7, color);
    }

    pub fn draw_icon_reload(&mut self, cx: f32, cy: f32, color: Color) {
        let r = 5.2_f32;
        let mut prev = None;
        for step in 0..=16 {
            let a = 0.45 + (step as f32 / 16.0) * 5.15;
            let px = cx + a.cos() * r;
            let py = cy + a.sin() * r;
            if let Some((lx, ly)) = prev {
                self.draw_line_aa(lx, ly, px, py, 1.55, color);
            }
            prev = Some((px, py));
        }
        let end_a = 0.45_f32;
        let ex = cx + end_a.cos() * r;
        let ey = cy + end_a.sin() * r;
        self.draw_line_aa(ex, ey, ex - 3.2, ey - 0.5, 1.5, color);
        self.draw_line_aa(ex, ey, ex - 0.5, ey - 3.2, 1.5, color);
    }

    pub fn draw_icon_stop(&mut self, cx: f32, cy: f32, color: Color) {
        self.draw_line_aa(cx - 4.0, cy - 4.0, cx + 4.0, cy + 4.0, 1.7, color);
        self.draw_line_aa(cx + 4.0, cy - 4.0, cx - 4.0, cy + 4.0, 1.7, color);
    }

    pub fn draw_icon_home(&mut self, cx: f32, cy: f32, color: Color) {
        self.draw_line_aa(cx - 5.2, cy - 0.4, cx, cy - 5.0, 1.6, color);
        self.draw_line_aa(cx, cy - 5.0, cx + 5.2, cy - 0.4, 1.6, color);
        self.draw_line_aa(cx - 3.8, cy - 1.4, cx - 3.8, cy + 5.0, 1.55, color);
        self.draw_line_aa(cx + 3.8, cy - 1.4, cx + 3.8, cy + 5.0, 1.55, color);
        self.draw_line_aa(cx - 3.8, cy + 5.0, cx + 3.8, cy + 5.0, 1.55, color);
    }

    pub fn draw_icon_reopen(&mut self, cx: f32, cy: f32, color: Color) {
        let r = 5.0_f32;
        let mut prev = None;
        for step in 0..=15 {
            let a = 3.55 + (step as f32 / 15.0) * 5.0;
            let px = cx + a.cos() * r;
            let py = cy + a.sin() * r;
            if let Some((lx, ly)) = prev {
                self.draw_line_aa(lx, ly, px, py, 1.5, color);
            }
            prev = Some((px, py));
        }
        let ex = cx + 3.55_f32.cos() * r;
        let ey = cy + 3.55_f32.sin() * r;
        self.draw_line_aa(ex, ey, ex + 3.2, ey - 0.2, 1.5, color);
        self.draw_line_aa(ex, ey, ex + 0.2, ey - 3.2, 1.5, color);
    }

    pub fn draw_icon_search(&mut self, cx: f32, cy: f32, color: Color) {
        self.stroke_circle(cx - 1.0, cy - 1.0, 4.0, 1.55, color);
        self.draw_line_aa(cx + 2.0, cy + 2.0, cx + 5.0, cy + 5.0, 1.7, color);
    }

    pub fn draw_icon_star(&mut self, cx: f32, cy: f32, filled: bool, color: Color) {
        let outer = 5.8_f32;
        let inner = 2.6_f32;
        let mut pts = [(0.0_f32, 0.0_f32); 10];
        for (i, slot) in pts.iter_mut().enumerate() {
            let angle = -std::f32::consts::FRAC_PI_2 + i as f32 * (std::f32::consts::PI / 5.0);
            let r = if i % 2 == 0 { outer } else { inner };
            *slot = (cx + angle.cos() * r, cy + angle.sin() * r);
        }
        if filled {
            self.fill_circle(cx, cy, 3.0, color);
            for i in 0..5 {
                let tip = pts[i * 2];
                self.draw_line_aa(cx, cy, tip.0, tip.1, 2.2, color);
            }
        }
        for i in 0..10 {
            let a = pts[i];
            let b = pts[(i + 1) % 10];
            self.draw_line_aa(a.0, a.1, b.0, b.1, 1.35, color);
        }
    }

    pub fn draw_icon_plus(&mut self, cx: f32, cy: f32, color: Color) {
        self.draw_line_aa(cx, cy - 4.5, cx, cy + 4.5, 1.7, color);
        self.draw_line_aa(cx - 4.5, cy, cx + 4.5, cy, 1.7, color);
    }

    pub fn draw_icon_minus(&mut self, cx: f32, cy: f32, color: Color) {
        self.draw_line_aa(cx - 4.5, cy, cx + 4.5, cy, 1.7, color);
    }

    pub fn draw_icon_copy(&mut self, cx: f32, cy: f32, color: Color) {
        self.stroke_rounded_rect(cx - 5.0, cy - 2.0, 7.2, 8.2, 1.6, 1.35, color);
        self.draw_line_aa(cx - 2.0, cy - 5.2, cx + 4.8, cy - 5.2, 1.35, color);
        self.draw_line_aa(cx + 4.8, cy - 5.2, cx + 4.8, cy + 2.4, 1.35, color);
    }

    pub fn draw_icon_history(&mut self, cx: f32, cy: f32, color: Color) {
        self.stroke_circle(cx, cy, 5.2, 1.4, color);
        self.draw_line_aa(cx, cy, cx, cy - 3.0, 1.4, color);
        self.draw_line_aa(cx, cy, cx + 2.4, cy + 1.4, 1.4, color);
    }

    pub fn draw_icon_palette(&mut self, cx: f32, cy: f32, color: Color) {
        self.draw_line_aa(cx - 4.0, cy - 3.5, cx - 0.5, cy, 1.6, color);
        self.draw_line_aa(cx - 0.5, cy, cx - 4.0, cy + 3.5, 1.6, color);
        self.draw_line_aa(cx + 0.5, cy + 3.5, cx + 4.8, cy + 3.5, 1.6, color);
    }

    pub fn draw_icon_sidebar(&mut self, cx: f32, cy: f32, color: Color) {
        self.stroke_rounded_rect(cx - 6.0, cy - 5.0, 12.0, 10.0, 0.0, 1.3, color);
        self.draw_line_aa(cx - 2.0, cy - 5.0, cx - 2.0, cy + 5.0, 1.3, color);
    }

    pub fn draw_icon_settings(&mut self, cx: f32, cy: f32, color: Color) {
        self.draw_line_aa(cx - 5.5, cy - 3.5, cx + 5.5, cy - 3.5, 1.35, color);
        self.fill_rect(cx - 3.0, cy - 5.2, 3.2, 3.4, color);
        self.draw_line_aa(cx - 5.5, cy, cx + 5.5, cy, 1.35, color);
        self.fill_rect(cx + 1.0, cy - 1.7, 3.2, 3.4, color);
        self.draw_line_aa(cx - 5.5, cy + 3.5, cx + 5.5, cy + 3.5, 1.35, color);
        self.fill_rect(cx - 2.0, cy + 1.8, 3.2, 3.4, color);
    }

    pub fn draw_icon_shield(&mut self, cx: f32, cy: f32, active: bool, color: Color) {
        self.draw_line_aa(cx, cy - 5.5, cx + 4.8, cy - 3.5, 1.4, color);
        self.draw_line_aa(cx + 4.8, cy - 3.5, cx + 3.6, cy + 2.2, 1.4, color);
        self.draw_line_aa(cx + 3.6, cy + 2.2, cx, cy + 5.5, 1.4, color);
        self.draw_line_aa(cx, cy + 5.5, cx - 3.6, cy + 2.2, 1.4, color);
        self.draw_line_aa(cx - 3.6, cy + 2.2, cx - 4.8, cy - 3.5, 1.4, color);
        self.draw_line_aa(cx - 4.8, cy - 3.5, cx, cy - 5.5, 1.4, color);
        if active {
            self.draw_line_aa(cx - 2.0, cy, cx - 0.4, cy + 1.8, 1.4, color);
            self.draw_line_aa(cx - 0.4, cy + 1.8, cx + 2.4, cy - 1.6, 1.4, color);
        }
    }

    pub fn draw_icon_reader(&mut self, cx: f32, cy: f32, color: Color) {
        self.draw_line_aa(cx - 5.0, cy - 3.5, cx + 5.0, cy - 3.5, 1.45, color);
        self.draw_line_aa(cx - 5.0, cy, cx + 5.0, cy, 1.45, color);
        self.draw_line_aa(cx - 5.0, cy + 3.5, cx + 2.5, cy + 3.5, 1.45, color);
    }

    pub fn draw_icon_volume(&mut self, cx: f32, cy: f32, muted: bool, color: Color) {
        self.fill_rect(cx - 5.0, cy - 2.0, 2.5, 4.0, color);
        self.draw_line_aa(cx - 2.5, cy - 2.0, cx + 1.2, cy - 4.8, 1.4, color);
        self.draw_line_aa(cx + 1.2, cy - 4.8, cx + 1.2, cy + 4.8, 1.4, color);
        self.draw_line_aa(cx + 1.2, cy + 4.8, cx - 2.5, cy + 2.0, 1.4, color);
        if muted {
            self.draw_line_aa(cx + 2.8, cy - 3.0, cx + 5.6, cy + 3.0, 1.4, color);
            self.draw_line_aa(cx + 5.6, cy - 3.0, cx + 2.8, cy + 3.0, 1.4, color);
        } else {
            self.draw_line_aa(cx + 3.5, cy - 2.5, cx + 4.5, cy, 1.3, color);
            self.draw_line_aa(cx + 4.5, cy, cx + 3.5, cy + 2.5, 1.3, color);
        }
    }

    pub fn draw_icon_close(&mut self, cx: f32, cy: f32, color: Color) {
        self.draw_line_aa(cx - 3.2, cy - 3.2, cx + 3.2, cy + 3.2, 1.5, color);
        self.draw_line_aa(cx + 3.2, cy - 3.2, cx - 3.2, cy + 3.2, 1.5, color);
    }

    pub fn draw_icon_theme(&mut self, cx: f32, cy: f32, dark: bool, color: Color) {
        if dark {
            self.fill_circle(cx, cy, 3.3, color);
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::FRAC_PI_4;
                self.draw_line_aa(
                    cx + a.cos() * 4.8,
                    cy + a.sin() * 4.8,
                    cx + a.cos() * 6.0,
                    cy + a.sin() * 6.0,
                    1.3,
                    color,
                );
            }
        } else {
            self.stroke_circle(cx, cy, 4.6, 1.45, color);
            self.fill_circle(cx - 1.5, cy + 0.5, 2.2, color);
        }
    }
}

