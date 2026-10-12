//! Light text rendering: ab_glyph rasterises glyphs lazily from a system font on disk
//! (only the file bytes stay resident), cached per size.

use ab_glyph::{Font, FontVec, PxScale, ScaleFont};

use std::collections::HashMap;
use tiny_skia::{Color, Pixmap, Transform};

const SANS: &[(&str, u32)] = &[
    ("/System/Library/Fonts/Helvetica.ttc", 0),
    ("C:\\Windows\\Fonts\\segoeui.ttf", 0),
    ("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 0),
    ("/usr/share/fonts/TTF/DejaVuSans.ttf", 0),
    ("/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf", 0),
];
const MONO: &[(&str, u32)] = &[
    ("/System/Library/Fonts/Menlo.ttc", 1),
    ("/System/Library/Fonts/Menlo.ttc", 0),
    ("C:\\Windows\\Fonts\\consolab.ttf", 0),
    ("C:\\Windows\\Fonts\\consola.ttf", 0),
    ("/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Bold.ttf", 0),
    ("/usr/share/fonts/TTF/DejaVuSansMono-Bold.ttf", 0),
];

/// A rasterised glyph: coverage bitmap plus placement relative to the pen and baseline.
#[derive(Clone)]
struct Glyph {
    advance: f32,
    left: i32,
    top: i32,
    w: usize,
    h: usize,
    cov: Vec<u8>,
}

fn load(cands: &[(&str, u32)]) -> Option<FontVec> {
    for (path, index) in cands {
        if let Ok(bytes) = std::fs::read(path) {
            if let Ok(f) = FontVec::try_from_vec_and_index(bytes, *index) {
                return Some(f);
            }
        }
    }
    None
}

pub struct Fonts {
    sans: Option<FontVec>,
    mono: Option<FontVec>,
    cache: HashMap<(bool, char, u32), Glyph>,
}

impl Fonts {
    pub fn load() -> Fonts {
        let sans = load(SANS);
        let mono = load(MONO).or_else(|| load(SANS));
        if sans.is_none() {
            eprintln!("desktop-widgets: no system font found; text will be blank");
        }
        Fonts { sans, mono, cache: HashMap::new() }
    }

    /// `px` is the em size in device pixels.
    fn glyph(&mut self, mono: bool, c: char, px: f32) -> Option<&Glyph> {
        let key = (mono, c, (px * 4.0) as u32);
        if !self.cache.contains_key(&key) {
            let font = if mono { self.mono.as_ref() } else { self.sans.as_ref() }?;
            let upem = font.units_per_em().unwrap_or(1000.0);
            let scale = PxScale::from(px * font.height_unscaled() / upem);
            let id = font.glyph_id(c);
            let advance = font.as_scaled(scale).h_advance(id);
            let mut g = Glyph { advance, left: 0, top: 0, w: 0, h: 0, cov: Vec::new() };
            if let Some(o) = font.outline_glyph(id.with_scale_and_position(scale, ab_glyph::point(0.0, 0.0))) {
                let b = o.px_bounds();
                g.left = b.min.x as i32;
                g.top = b.min.y as i32;
                g.w = b.width() as usize;
                g.h = b.height() as usize;
                g.cov = vec![0; g.w * g.h];
                let w = g.w;
                o.draw(|x, y, v| {
                    let i = y as usize * w + x as usize;
                    if i < g.cov.len() {
                        g.cov[i] = (v.clamp(0.0, 1.0) * 255.0) as u8;
                    }
                });
            }
            if self.cache.len() > 4000 {
                self.cache.clear();
            }
            self.cache.insert(key, g);
        }
        self.cache.get(&key)
    }

    fn advance(&mut self, mono: bool, c: char, px: f32) -> f32 {
        self.glyph(mono, c, px).map(|g| g.advance).unwrap_or(0.0)
    }

    /// Width in logical points.
    pub fn measure(&mut self, text: &str, size: f64, scale: f32, mono: bool) -> f64 {
        let px = size as f32 * scale;
        text.chars().map(|c| self.advance(mono, c, px)).sum::<f32>() as f64 / scale as f64
    }

    /// Draw `text` with its baseline at (x, y) in logical points, clipped with an ellipsis to
    /// `max_w`. Returns the drawn width in logical points.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(&mut self, pm: &mut Pixmap, scale: f32, t: Transform, text: &str, x: f64, y: f64, size: f64, max_w: f64, color: Color, mono: bool) -> f64 {
        if max_w <= 4.0 || text.is_empty() {
            return 0.0;
        }
        let mut text = text.to_string();
        if self.measure(&text, size, scale, mono) > max_w {
            while !text.is_empty() && self.measure(&format!("{text}…"), size, scale, mono) > max_w {
                text.pop();
            }
            text.push('…');
        }
        let px = size as f32 * scale;
        // Device-space origin (the transform is a pure scale here).
        let ox = (x as f32 * t.sx + t.tx).round();
        let oy = (y as f32 * t.sy + t.ty).round();
        let mut pen = 0.0f32;
        let (pw, ph) = (pm.width() as i32, pm.height() as i32);
        let (cr, cg, cb, ca) = (color.red(), color.green(), color.blue(), color.alpha());
        for c in text.chars() {
            let Some(g) = self.glyph(mono, c, px).cloned() else { continue };
            let gx = (ox + pen).round() as i32 + g.left;
            let gy = oy as i32 + g.top;
            let data = pm.data_mut();
            for row in 0..g.h as i32 {
                let py = gy + row;
                if py < 0 || py >= ph {
                    continue;
                }
                for col in 0..g.w as i32 {
                    let px_ = gx + col;
                    if px_ < 0 || px_ >= pw {
                        continue;
                    }
                    let cov = g.cov[(row * g.w as i32 + col) as usize] as f32 / 255.0 * ca;
                    if cov <= 0.0 {
                        continue;
                    }
                    let i = ((py * pw + px_) * 4) as usize;
                    // Source-over in premultiplied RGBA.
                    let inv = 1.0 - cov;
                    data[i] = (cr * cov * 255.0 + data[i] as f32 * inv) as u8;
                    data[i + 1] = (cg * cov * 255.0 + data[i + 1] as f32 * inv) as u8;
                    data[i + 2] = (cb * cov * 255.0 + data[i + 2] as f32 * inv) as u8;
                    data[i + 3] = (cov * 255.0 + data[i + 3] as f32 * inv) as u8;
                }
            }
            pen += g.advance;
        }
        pen as f64 / scale as f64
    }
}
