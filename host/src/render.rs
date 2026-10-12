//! Native drawing with tiny-skia. The casing is drawn in separate parts (outline, end caps,
//! nameplate grip, bezel/panel, buttons) so a skin can replace any of them later.

use crate::actor::State;
use crate::casing::{Button, Parts};
use crate::geom::Rect;
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
    let (inset, cap_w) = if p.shaded { (5.0, 4.0) } else { (9.0, 7.0) };
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
