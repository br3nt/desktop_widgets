//! Native drawing with tiny-skia. The casing is drawn in separate parts (outline, end caps,
//! nameplate grip, bezel/panel, buttons) so a skin can replace any of them later.

use crate::actor::State;
use crate::casing::{Button, Parts};
use crate::geom::Rect;
use crate::glow::Side;
use crate::text::Fonts;
use tiny_skia::{
    Color, FillRule, GradientStop, LinearGradient, Paint, PathBuilder, Pixmap, Point, Shader, SpreadMode, Stroke, Transform,
};

pub struct Palette {
    pub lime: Color,
    pub cyan: Color,
    pub muted: Color,
}

pub fn palette() -> Palette {
    Palette {
        lime: Color::from_rgba8(0xbd, 0xdd, 0x55, 255),
        cyan: Color::from_rgba8(0x45, 0xed, 0xdf, 255),
        muted: Color::from_rgba8(0x82, 0x9d, 0x92, 255),
    }
}

/// Everything needed to draw one widget.
pub struct WidgetView<'a> {
    pub title: &'a str,
    pub state: Option<&'a State>,
    pub parts: &'a Parts,
    pub on_top: bool,
    pub docked: bool,
    /// HTML panels: draw the empty screen only; the webview paints the content.
    pub html: bool,
    /// Socket light brightness at the (left, right) ends, 0..1.
    pub lights: (f64, f64),
}

/// Where a socket light sits: over the end lens, about 54% of the casing's height.
pub fn socket_light_rect(p: &Parts, side: Side) -> Rect {
    let (o, (inset, lens_w)) = (p.outline, end_lens(p));
    let cx = match side {
        Side::Left => o.x + inset + lens_w / 2.0,
        Side::Right => o.right() - inset - lens_w / 2.0,
    };
    // The prototype's 12×70 light on a 130-high casing.
    let h = o.h * 70.0 / 130.0;
    Rect::new(cx - 6.0, o.y + (o.h - h) / 2.0, 12.0, h)
}

/// The end lens's inset from the outline and its width.
fn end_lens(p: &Parts) -> (f64, f64) {
    if p.shaded { (5.0, 4.0) } else { (9.0, 7.0) }
}

/// Blur sigma of the light's halo (points), the prototype's `feGaussianBlur stdDeviation="5"`.
const LIGHT_BLUR: f64 = 5.0;

/// Three box-blur passes each way approximate a gaussian of sigma about `r` (premultiplied
/// RGBA, so plain averaging is correct).
fn box_blur(data: &mut [u8], w: usize, h: usize, r: usize) {
    if r == 0 || w == 0 || h == 0 {
        return;
    }
    let mut line = Vec::new();
    for _ in 0..3 {
        for (len, count, stride, step) in [(w, h, w * 4, 4), (h, w, 4, w * 4)] {
            for k in 0..count {
                let base = k * stride;
                for c in 0..4 {
                    line.clear();
                    line.extend((0..len).map(|i| data[base + i * step + c] as u32));
                    let mut sum: u32 = line.iter().take(r + 1).sum();
                    let n = (2 * r + 1) as u32;
                    for i in 0..len {
                        data[base + i * step + c] = ((sum + n / 2) / n) as u8;
                        if i + r + 1 < len {
                            sum += line[i + r + 1];
                        }
                        if i >= r {
                            sum -= line[i - r];
                        }
                    }
                }
            }
        }
    }
}

/// A lime socket light at `side`, `amount` 0..1: a blurred 12-wide strip with a bright core.
pub fn draw_socket_light(pm: &mut Pixmap, scale: f32, p: &Parts, side: Side, amount: f64) {
    if amount <= 0.0 || p.shaded {
        return;
    }
    let r = socket_light_rect(p, side);
    let s = scale as f64;
    let pad = 3.0 * LIGHT_BLUR;
    let area = Rect::new(r.x - pad, r.y - pad, r.w + 2.0 * pad, r.h + 2.0 * pad);
    let (bw, bh) = ((area.w * s).ceil() as u32, (area.h * s).ceil() as u32);
    if let Some(mut halo) = Pixmap::new(bw, bh) {
        if let Some(path) = rounded(Rect::new(pad, pad, r.w, r.h), 6.0, 6.0) {
            halo.fill_path(&path, &paint_solid(rgba(0xb7, 0xf6, 0x65, 255)), FillRule::Winding, Transform::from_scale(scale, scale), None);
        }
        box_blur(halo.data_mut(), bw as usize, bh as usize, (LIGHT_BLUR * s).round() as usize);
        let paint = tiny_skia::PixmapPaint { opacity: amount.clamp(0.0, 1.0) as f32, ..Default::default() };
        pm.draw_pixmap((area.x * s).round() as i32, (area.y * s).round() as i32, halo.as_ref(), &paint, Transform::identity(), None);
    }
    // The bright core: 4 wide, 63/70 of the light's height.
    let core_h = r.h * 63.0 / 70.0;
    let core = Rect::new(r.x + 4.0, r.y + (r.h - core_h) / 2.0, 4.0, core_h);
    if let Some(path) = rounded(core, 2.0, 2.0) {
        let a = (amount.clamp(0.0, 1.0) * 255.0).round() as u8;
        pm.fill_path(&path, &paint_solid(rgba(0xd9, 0xff, 0x92, a)), FillRule::Winding, Transform::from_scale(scale, scale), None);
    }
}

fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba8(r, g, b, a)
}

fn paint_solid(c: Color) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color(c);
    p.anti_alias = true;
    p
}

/// A rectangle with elliptical corners (rx across, ry down), the capsule silhouette.
fn rounded(r: Rect, rx: f64, ry: f64) -> Option<tiny_skia::Path> {
    let (x, y, w, h) = (r.x as f32, r.y as f32, r.w as f32, r.h as f32);
    let rx = (rx as f32).min(w / 2.0);
    let ry = (ry as f32).min(h / 2.0);
    let k = 0.552_284_8_f32;
    let mut pb = PathBuilder::new();
    pb.move_to(x + rx, y);
    pb.line_to(x + w - rx, y);
    pb.cubic_to(x + w - rx + rx * k, y, x + w, y + ry - ry * k, x + w, y + ry);
    pb.line_to(x + w, y + h - ry);
    pb.cubic_to(x + w, y + h - ry + ry * k, x + w - rx + rx * k, y + h, x + w - rx, y + h);
    pb.line_to(x + rx, y + h);
    pb.cubic_to(x + rx - rx * k, y + h, x, y + h - ry + ry * k, x, y + h - ry);
    pb.line_to(x, y + ry);
    pb.cubic_to(x, y + ry - ry * k, x + rx - rx * k, y, x + rx, y);
    pb.close();
    pb.finish()
}

fn vgrad(y0: f64, y1: f64, stops: &[(f32, Color)]) -> Shader<'static> {
    LinearGradient::new(
        Point::from_xy(0.0, y0 as f32),
        Point::from_xy(0.0, y1 as f32),
        stops.iter().map(|(o, c)| GradientStop::new(*o, *c)).collect(),
        SpreadMode::Pad,
        Transform::identity(),
    )
    .unwrap_or(Shader::SolidColor(stops[0].1))
}

// ---- casing parts ----

pub fn draw_outline(pm: &mut Pixmap, t: Transform, p: &Parts) {
    let o = p.outline;
    let (rx, ry) = if p.shaded { (o.h / 2.0, o.h / 2.0) } else { ((o.h * 0.45).min(52.0), o.h / 2.0) };
    let Some(path) = rounded(o, rx, ry) else { return };
    let mut paint = Paint::default();
    paint.anti_alias = true;
    paint.shader = vgrad(
        o.y,
        o.bottom(),
        &[
            (0.0, rgba(0xd5, 0xda, 0xca, 255)),
            (0.07, rgba(0x90, 0x9b, 0x89, 255)),
            (0.15, rgba(0x44, 0x51, 0x41, 255)),
            (0.45, rgba(0x25, 0x34, 0x28, 255)),
            (0.65, rgba(0x4f, 0x61, 0x4a, 255)),
            (0.82, rgba(0x37, 0x4b, 0x35, 255)),
            (0.94, rgba(0x0b, 0x1a, 0x0e, 255)),
            (1.0, rgba(0x71, 0x84, 0x6b, 255)),
        ],
    );
    pm.fill_path(&path, &paint, FillRule::Winding, t, None);
    let stroke = Stroke { width: 1.5, ..Default::default() };
    pm.stroke_path(&path, &paint_solid(rgba(0x0a, 0x16, 0x0c, 255)), &stroke, t, None);
    // Top highlight.
    if !p.shaded {
        let mut pb = PathBuilder::new();
        pb.move_to((o.x + rx * 0.55) as f32, (o.y + 14.0) as f32);
        pb.quad_to((o.x + rx * 0.7) as f32, (o.y + 4.5) as f32, (o.x + rx) as f32, (o.y + 4.5) as f32);
        pb.line_to((o.right() - rx) as f32, (o.y + 4.5) as f32);
        pb.quad_to((o.right() - rx * 0.7) as f32, (o.y + 4.5) as f32, (o.right() - rx * 0.55) as f32, (o.y + 14.0) as f32);
        if let Some(hl) = pb.finish() {
            pm.stroke_path(&hl, &paint_solid(rgba(0xe5, 0xe8, 0xd2, 150)), &Stroke { width: 1.2, ..Default::default() }, t, None);
        }
    }
}

/// The lens-like end caps (cyan left, green right) from the capsule study.
pub fn draw_end_caps(pm: &mut Pixmap, t: Transform, p: &Parts) {
    let o = p.outline;
    let (inset, cap_w) = end_lens(p);
    let h = if p.shaded { o.h - 10.0 } else { o.h * 0.55 };
    let cy = o.y + o.h / 2.0;
    for (x, c0, c1) in [
        (o.x + inset, rgba(0x0c, 0x50, 0x5b, 255), rgba(0x6a, 0xee, 0xe3, 255)),
        (o.right() - inset - cap_w, rgba(0x28, 0x49, 0x19, 255), rgba(0xb4, 0xe4, 0x41, 255)),
    ] {
        if let Some(path) = rounded(Rect::new(x, cy - h / 2.0, cap_w, h), cap_w / 2.0, cap_w / 2.0) {
            let mut paint = Paint::default();
            paint.anti_alias = true;
            paint.shader = vgrad(cy - h / 2.0, cy + h / 2.0, &[(0.0, c0), (0.5, c1), (1.0, c0)]);
            pm.fill_path(&path, &paint, FillRule::Winding, t, None);
        }
    }
}

pub fn draw_grip(pm: &mut Pixmap, t: Transform, fonts: &mut Fonts, scale: f32, p: &Parts, title: &str) {
    let g = p.grip;
    if g.w < 10.0 {
        return;
    }
    let size = if p.shaded { 11.0 } else { 11.0 };
    let text_w = g.w - 16.0;
    let baseline = g.y + g.h / 2.0 + 4.0;
    let label = if p.shaded { title.split(" / ").next().unwrap_or(title) } else { title };
    // Engraved: dark offset under a light face.
    fonts.draw(pm, scale, t, label, g.x + 2.0, baseline + 1.0, size, text_w, rgba(0x05, 0x0d, 0x07, 200), false);
    fonts.draw(pm, scale, t, label, g.x + 2.0, baseline, size, text_w, rgba(0xdc, 0xe6, 0xd2, 255), false);
    // Grip dots.
    let dx = g.right() - 8.0;
    for row in 0..2 {
        for col in 0..2 {
            let cx = dx + col as f64 * 4.0;
            let cy = g.y + g.h / 2.0 - 3.0 + row as f64 * 5.0;
            if let Some(c) = PathBuilder::from_circle(cx as f32, cy as f32, 1.1) {
                pm.fill_path(&c, &paint_solid(rgba(0xb5, 0xc9, 0xb8, 160)), FillRule::Winding, t, None);
            }
        }
    }
}

/// The display bezel and screen glass.
pub fn draw_bezel(pm: &mut Pixmap, t: Transform, p: &Parts) {
    let s = p.panel;
    let r = if p.shaded { 4.0 } else { 7.0 };
    if let Some(path) = rounded(s.inset(-1.5), r + 1.5, r + 1.5) {
        pm.fill_path(&path, &paint_solid(rgba(0x0a, 0x16, 0x0c, 255)), FillRule::Winding, t, None);
    }
    if let Some(path) = rounded(s, r, r) {
        let mut paint = Paint::default();
        paint.anti_alias = true;
        paint.shader = vgrad(s.y, s.bottom(), &[(0.0, rgba(0x0b, 0x22, 0x20, 255)), (1.0, rgba(0x06, 0x10, 0x0e, 255))]);
        pm.fill_path(&path, &paint, FillRule::Winding, t, None);
        pm.stroke_path(&path, &paint_solid(rgba(0x45, 0xed, 0xdf, 60)), &Stroke { width: 1.0, ..Default::default() }, t, None);
    }
}

pub fn draw_buttons(pm: &mut Pixmap, t: Transform, p: &Parts, on_top: bool) {
    let pal = palette();
    for (b, r) in &p.buttons {
        let (cx, cy, rad) = ((r.x + r.w / 2.0) as f32, (r.y + r.h / 2.0) as f32, (r.w / 2.0) as f32);
        // Lens body.
        if let Some(c) = PathBuilder::from_circle(cx, cy, rad) {
            let mut paint = Paint::default();
            paint.anti_alias = true;
            paint.shader = vgrad(r.y, r.bottom(), &[(0.0, rgba(0xb0, 0xbb, 0xa0, 255)), (0.45, rgba(0x46, 0x58, 0x3c, 255)), (1.0, rgba(0x17, 0x2c, 0x19, 255))]);
            pm.fill_path(&c, &paint, FillRule::Winding, t, None);
            pm.stroke_path(&c, &paint_solid(rgba(0x07, 0x12, 0x09, 255)), &Stroke { width: 1.0, ..Default::default() }, t, None);
        }
        let ink = rgba(0x07, 0x10, 0x0e, 230);
        let s = rad * 0.45;
        let mut pb = PathBuilder::new();
        match b {
            Button::Shade => {
                // Up chevron when open (collapse), down when shaded (expand).
                let d = if p.shaded { -1.0 } else { 1.0 };
                pb.move_to(cx - s, cy + s * 0.5 * d);
                pb.line_to(cx, cy - s * 0.5 * d);
                pb.line_to(cx + s, cy + s * 0.5 * d);
            }
            Button::OnTop => {
                // A pin head; lit lime when always-on-top.
                if let Some(c) = PathBuilder::from_circle(cx, cy, s * 0.8) {
                    let col = if on_top { pal.lime } else { rgba(0x20, 0x2c, 0x22, 255) };
                    pm.fill_path(&c, &paint_solid(col), FillRule::Winding, t, None);
                }
            }
            Button::Detach => {
                pb.move_to(cx - s, cy - s);
                pb.line_to(cx - s, cy + s);
                pb.move_to(cx + s, cy - s);
                pb.line_to(cx + s, cy + s);
                pb.move_to(cx - s * 0.3, cy);
                pb.line_to(cx + s * 0.3, cy);
            }
        }
        if let Some(path) = pb.finish() {
            pm.stroke_path(&path, &paint_solid(ink), &Stroke { width: 1.6, line_cap: tiny_skia::LineCap::Round, ..Default::default() }, t, None);
        }
    }
}

fn s<'a>(st: Option<&'a State>, k: &str) -> String {
    match st.and_then(|m| m.get(k)) {
        Some(serde_json::Value::String(v)) => v.clone(),
        Some(serde_json::Value::Null) | None => String::new(),
        Some(v) => v.to_string(),
    }
}

/// Native display content: declarative fields from the widget's actor state.
pub fn draw_native_panel(pm: &mut Pixmap, t: Transform, fonts: &mut Fonts, scale: f32, p: &Parts, st: Option<&State>) {
    let pal = palette();
    let r = p.panel.inset(if p.shaded { 2.0 } else { 7.0 });
    if p.shaded {
        let value = s(st, "value");
        let unit = s(st, "unit");
        let x = r.x + 4.0;
        let vw = fonts.draw(pm, scale, t, &value, x, r.y + r.h / 2.0 + 4.5, 13.0, r.w - 4.0, pal.lime, true);
        fonts.draw(pm, scale, t, &unit, x + vw + 6.0, r.y + r.h / 2.0 + 3.5, 9.5, r.w - vw - 12.0, pal.muted, false);
        return;
    }
    fonts.draw(pm, scale, t, &s(st, "caption"), r.x, r.y + 9.0, 9.5, r.w, pal.cyan, false);
    let value = s(st, "value");
    let vy = r.y + r.h * 0.62;
    let vw = fonts.draw(pm, scale, t, &value, r.x, vy, 26.0, r.w, pal.lime, true);
    fonts.draw(pm, scale, t, &s(st, "unit"), r.x + vw + 8.0, vy - 1.0, 12.0, r.w - vw - 8.0, pal.lime, false);
    fonts.draw(pm, scale, t, &s(st, "detail"), r.x, r.bottom() - 1.0, 10.5, r.w, pal.muted, false);
}

pub fn draw_widget(pm: &mut Pixmap, scale: f32, fonts: &mut Fonts, v: &WidgetView) {
    pm.fill(Color::TRANSPARENT);
    let t = Transform::from_scale(scale, scale);
    draw_outline(pm, t, v.parts);
    draw_end_caps(pm, t, v.parts);
    draw_socket_light(pm, scale, v.parts, Side::Left, v.lights.0);
    draw_socket_light(pm, scale, v.parts, Side::Right, v.lights.1);
    draw_grip(pm, t, fonts, scale, v.parts, v.title);
    draw_bezel(pm, t, v.parts);
    if !v.html {
        draw_native_panel(pm, t, fonts, scale, v.parts, v.state);
    }
    draw_buttons(pm, t, v.parts, v.on_top);
    if v.docked {
        // A small lime tick on the outline marks a docked widget.
        let o = v.parts.outline;
        if let Some(c) = PathBuilder::from_circle((o.x + o.h / 2.0) as f32, (o.y + 3.5) as f32, 1.6) {
            pm.fill_path(&c, &paint_solid(palette().lime), FillRule::Winding, t, None);
        }
    }
}

/// Faint dock region highlight shown while dragging.
pub fn draw_dock_overlay(pm: &mut Pixmap, scale: f32, fonts: &mut Fonts, w: f64, h: f64, label: &str, hot: bool) {
    pm.fill(Color::TRANSPARENT);
    let t = Transform::from_scale(scale, scale);
    let r = Rect::new(1.0, 1.0, w - 2.0, h - 2.0);
    if let Some(path) = rounded(r, 8.0, 8.0) {
        pm.fill_path(&path, &paint_solid(rgba(0xbd, 0xdd, 0x55, if hot { 70 } else { 28 })), FillRule::Winding, t, None);
        let stroke = Stroke { width: 1.5, dash: tiny_skia::StrokeDash::new(vec![5.0, 4.0], 0.0), ..Default::default() };
        pm.stroke_path(&path, &paint_solid(rgba(0xd0, 0xf0, 0x75, if hot { 230 } else { 140 })), &stroke, t, None);
    }
    fonts.draw(pm, scale, t, label, 10.0, 15.0, 10.0, w - 20.0, rgba(0xe5, 0xf5, 0xbd, if hot { 255 } else { 180 }), false);
}

/// The drag caption: lime monospace on a faint dark pill (brighter when ready to join).
pub fn draw_caption(pm: &mut Pixmap, scale: f32, fonts: &mut Fonts, w: f64, h: f64, text: &str, ready: bool) {
    pm.fill(Color::TRANSPARENT);
    let t = Transform::from_scale(scale, scale);
    let size = 11.0;
    let tw = fonts.measure(text, size, scale, true).min(w - 16.0);
    let pill = Rect::new((w - tw) / 2.0 - 8.0, 1.0, tw + 16.0, h - 2.0);
    if let Some(path) = rounded(pill, pill.h / 2.0, pill.h / 2.0) {
        pm.fill_path(&path, &paint_solid(rgba(0x08, 0x12, 0x0a, 190)), FillRule::Winding, t, None);
    }
    // The prototype's #phase colours: #b8d478, and #e4fca7 once captured.
    let ink = if ready { rgba(0xe4, 0xfc, 0xa7, 255) } else { rgba(0xb8, 0xd4, 0x78, 255) };
    fonts.draw(pm, scale, t, text, (w - tw) / 2.0, h / 2.0 + 4.0, size, tw + 1.0, ink, true);
}

/// A tiny capsule glyph for the menu bar (template image: black + alpha).
pub fn tray_icon_rgba(size: u32) -> Vec<u8> {
    let Some(mut pm) = Pixmap::new(size, size) else { return vec![0; (size * size * 4) as usize] };
    let s = size as f64;
    let r = Rect::new(s * 0.08, s * 0.28, s * 0.84, s * 0.44);
    if let Some(path) = rounded(r, r.h / 2.0, r.h / 2.0) {
        pm.stroke_path(&path, &paint_solid(Color::BLACK), &Stroke { width: (s * 0.09) as f32, ..Default::default() }, Transform::identity(), None);
    }
    if let Some(path) = rounded(Rect::new(s * 0.3, s * 0.42, s * 0.4, s * 0.16), s * 0.04, s * 0.04) {
        pm.fill_path(&path, &paint_solid(Color::BLACK), FillRule::Winding, Transform::identity(), None);
    }
    pm.data().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::casing;
    use crate::magnet::proximity;

    /// Mean luminance of a strip (points) of a widget drawn at 2x, as the prototype's pixel
    /// check measures it.
    fn luminance(lights: (f64, f64), strip: Rect) -> f64 {
        let (w, h, scale) = (360.0, 116.0, 2.0f32);
        let parts = casing::parts(w, h, false, false);
        let mut pm = Pixmap::new((w * 2.0) as u32, (h * 2.0) as u32).unwrap();
        let mut fonts = Fonts::load();
        let view = WidgetView { title: "Jarvis / agent control", state: None, parts: &parts, on_top: false, docked: false, html: false, lights };
        draw_widget(&mut pm, scale, &mut fonts, &view);
        let (mut sum, mut n) = (0.0, 0.0);
        for y in (strip.y * 2.0) as u32..(strip.bottom() * 2.0) as u32 {
            for x in (strip.x * 2.0) as u32..(strip.right() * 2.0) as u32 {
                let p = pm.pixel(x, y).unwrap().demultiply();
                sum += 0.2126 * p.red() as f64 + 0.7152 * p.green() as f64 + 0.0722 * p.blue() as f64;
                n += 1.0;
            }
        }
        sum / n
    }

    #[test]
    fn both_sockets_glow_dim_when_near_and_bright_when_ready() {
        let parts = casing::parts(360.0, 116.0, false, false);
        let near = proximity(110.0).brightness;
        for (side, lights) in [(socket_light_rect(&parts, Side::Left), [(0.0, 0.0), (near, 0.0), (1.0, 0.0)]), (socket_light_rect(&parts, Side::Right), [(0.0, 0.0), (0.0, near), (0.0, 1.0)])] {
            // The prototype samples a 3×22 strip on the socket.
            let c = (side.x + side.w / 2.0, side.y + side.h / 2.0);
            let strip = Rect::new(c.0 - 1.5, c.1 - 11.0, 3.0, 22.0);
            let [idle, dim, ready] = lights.map(|l| luminance(l, strip));
            assert!(dim > idle + 5.0, "nearby must visibly glow: idle {idle:.1}, near {dim:.1}");
            assert!(ready > dim + 10.0, "ready must be brighter: near {dim:.1}, ready {ready:.1}");
        }
    }

    #[test]
    fn a_light_only_touches_its_own_end() {
        let parts = casing::parts(360.0, 116.0, false, false);
        let l = socket_light_rect(&parts, Side::Left);
        let strip = Rect::new(l.x + l.w / 2.0 - 1.5, l.y + l.h / 2.0 - 11.0, 3.0, 22.0);
        assert_eq!(luminance((0.0, 0.0), strip), luminance((0.0, 1.0), strip));
        // The light sits over the end lens, about the prototype's 70/130 of the height.
        assert!((l.h / 116.0 - 70.0 / 130.0).abs() < 0.02);
        assert!(l.x < 30.0 && socket_light_rect(&parts, Side::Right).right() > 330.0);
    }
}
