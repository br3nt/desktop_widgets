//! Casing layout: where the parts of a widget's chrome sit (outline, nameplate grip, display
//! panel, buttons) for a given size and shade state. Drawing (`render.rs`) and hit-testing
//! both use this, so a skin can replace the drawing while the parts stay consistent.

use crate::geom::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    Shade,
    OnTop,
    Detach,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parts {
    pub outline: Rect,
    /// The engraved nameplate: drag handle, double-click to shade.
    pub grip: Rect,
    /// The display slot. Native panels draw here; HTML panels put a webview here.
    pub panel: Rect,
    /// Buttons with their hit circles (centre x, centre y, radius) in a rect.
    pub buttons: Vec<(Button, Rect)>,
    pub shaded: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hit {
    Button(Button),
    Panel,
    Casing,
}

pub fn parts(w: f64, h: f64, shaded: bool, grouped: bool) -> Parts {
    let outline = Rect::new(1.0, 1.0, w - 2.0, h - 2.0);
    let mut buttons = Vec::new();
    if shaded {
        let r = 7.0;
        let cy = h / 2.0;
        let mut cx = w - 14.0;
        let mut kinds = vec![Button::Shade, Button::OnTop];
        if grouped {
            kinds.push(Button::Detach);
        }
        for b in kinds {
            buttons.push((b, Rect::new(cx - r, cy - r, 2.0 * r, 2.0 * r)));
            cx -= 2.0 * r + 3.0;
        }
        let right = cx + r - 2.0;
        let left = (w * 0.42).max(right - 150.0).min(right - 30.0);
        Parts {
            outline,
            grip: Rect::new(10.0, 2.0, (left - 14.0).max(0.0), h - 4.0),
            panel: Rect::new(left, 4.0, (right - left).max(0.0), h - 8.0),
            buttons,
            shaded,
        }
    } else {
        let r = 8.0;
        let cx = w - 30.0;
        let mut kinds = vec![Button::Shade, Button::OnTop];
        if grouped {
            kinds.push(Button::Detach);
        }
        let step = (h - 40.0) / 2.0;
        for (k, b) in kinds.into_iter().enumerate() {
            let cy = 20.0 + step * k as f64;
            buttons.push((b, Rect::new(cx - r, cy - r, 2.0 * r, 2.0 * r)));
        }
        Parts {
            outline,
            grip: Rect::new(34.0, 4.0, w - 34.0 - 50.0, 20.0),
            panel: Rect::new(36.0, 25.0, w - 36.0 - 50.0, h - 25.0 - 12.0),
            buttons,
            shaded,
        }
    }
}

impl Parts {
    pub fn hit(&self, x: f64, y: f64) -> Hit {
        for (b, r) in &self.buttons {
            // Generous hit circle around each lens button.
            let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
            if (x - cx).hypot(y - cy) <= r.w / 2.0 + 3.0 {
                return Hit::Button(*b);
            }
        }
        if self.panel.contains(x, y) {
            return Hit::Panel;
        }
        Hit::Casing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_parts_do_not_overlap() {
        let p = parts(360.0, 116.0, false, true);
        assert_eq!(p.buttons.len(), 3);
        for (_, b) in &p.buttons {
            assert!(!b.intersects(&p.panel));
            assert!(p.outline.contains(b.x, b.y));
        }
        assert!(!p.grip.intersects(&p.panel));
        assert_eq!(p.hit(p.panel.x + 5.0, p.panel.y + 5.0), Hit::Panel);
        assert_eq!(p.hit(10.0, 58.0), Hit::Casing);
        let (b, r) = p.buttons[1];
        assert_eq!(p.hit(r.x + r.w / 2.0, r.y + r.h / 2.0), Hit::Button(b));
    }

    #[test]
    fn shade_parts_fit_a_strip_and_drop_detach_when_alone() {
        let p = parts(360.0, 26.0, true, false);
        assert_eq!(p.buttons.len(), 2);
        assert!(p.panel.h > 10.0 && p.panel.right() < p.buttons[1].1.x);
        assert!(p.grip.right() <= p.panel.x);
        // Narrow dock width still leaves room for a value.
        let p = parts(200.0, 26.0, true, false);
        assert!(p.panel.w >= 30.0);
    }
}
