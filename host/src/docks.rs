//! Dock regions: user placement regions, by default either side of the OS Dock/taskbar.
//! Pure geometry: the platform layer supplies screen frames, the usable ("visible") frame
//! and, when it knows it, the Dock's exact rectangle.

use crate::geom::Rect;
use serde::{Deserialize, Serialize};

/// Which way widgets pack inside a region: toward the Dock edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Pack {
    /// Fill from the region's right edge leftwards (region sits left of a bottom Dock).
    FromRight,
    /// Fill from the region's left edge rightwards.
    FromLeft,
    /// Fill from the bottom upwards (region sits above a side Dock).
    FromBottom,
    /// Fill from the top downwards.
    FromTop,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DockRegion {
    pub id: String,
    pub rect: Rect,
    pub pack: Pack,
    pub on_top: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DockSide {
    Bottom,
    Left,
    Right,
}

/// What the platform knows about one screen.
#[derive(Clone, Debug, PartialEq)]
pub struct ScreenInfo {
    pub frame: Rect,
    pub visible: Rect,
}

/// Gap kept between the Dock and a region, and from screen edges.
const GAP: f64 = 4.0;
/// Strip thickness used when the Dock reserves no space (auto-hide).
pub const AUTOHIDE_STRIP: f64 = 48.0;

/// Work out which side the Dock is on from the reserved space; `hint` is used when nothing is reserved.
pub fn dock_side(s: &ScreenInfo, hint: DockSide) -> (DockSide, f64) {
    let bottom = s.frame.bottom() - s.visible.bottom();
    let left = s.visible.x - s.frame.x;
    let right = s.frame.right() - s.visible.right();
    if bottom > 1.0 {
        (DockSide::Bottom, bottom)
    } else if left > 1.0 {
        (DockSide::Left, left)
    } else if right > 1.0 {
        (DockSide::Right, right)
    } else {
        (hint, 0.0)
    }
}

/// Default regions either side of the Dock on the screen that holds it.
/// `dock` is the Dock's exact rect if known; otherwise `est_len` (its length along its edge) is centred.
pub fn default_regions(s: &ScreenInfo, hint: DockSide, dock: Option<Rect>, est_len: f64) -> Vec<DockRegion> {
    let (side, reserved) = dock_side(s, hint);
    let thick = if reserved > 1.0 { reserved } else { dock.map(|d| d.h.min(d.w)).unwrap_or(AUTOHIDE_STRIP) };
    let f = s.frame;
    let mk = |id: &str, rect: Rect, pack| DockRegion { id: id.to_string(), rect, pack, on_top: true };
    match side {
        DockSide::Bottom => {
            let (d0, d1) = match dock {
                Some(d) => (d.x, d.right()),
                None => (f.center_x() - est_len / 2.0, f.center_x() + est_len / 2.0),
            };
            let y = f.bottom() - thick;
            let mut out = Vec::new();
            let lw = d0 - GAP - (f.x + GAP);
            if lw > 40.0 {
                out.push(mk("dock-left", Rect::new(f.x + GAP, y, lw, thick), Pack::FromRight));
            }
            let rw = f.right() - GAP - (d1 + GAP);
            if rw > 40.0 {
                out.push(mk("dock-right", Rect::new(d1 + GAP, y, rw, thick), Pack::FromLeft));
            }
            out
        }
        DockSide::Left | DockSide::Right => {
            let (d0, d1) = match dock {
                Some(d) => (d.y, d.bottom()),
                None => {
                    let cy = f.y + f.h / 2.0;
                    (cy - est_len / 2.0, cy + est_len / 2.0)
                }
            };
            let x = if side == DockSide::Left { f.x } else { f.right() - thick };
            let top = s.visible.y + GAP;
            let mut out = Vec::new();
            if d0 - GAP - top > 40.0 {
                out.push(mk("dock-above", Rect::new(x, top, thick, d0 - GAP - top), Pack::FromBottom));
            }
            let bottom = f.bottom() - GAP;
            if bottom - (d1 + GAP) > 40.0 {
                out.push(mk("dock-below", Rect::new(x, d1 + GAP, thick, bottom - (d1 + GAP)), Pack::FromTop));
            }
            out
        }
    }
}

/// Lay out docked widgets inside a region. Each entry is a widget's preferred (width, height)
/// when shaded; returns one rect per entry. Widgets fill lanes next to the Dock first,
/// then further lanes outward. Width is clamped to the region.
pub fn layout(region: &DockRegion, sizes: &[(f64, f64)]) -> Vec<Rect> {
    const PAD: f64 = 2.0;
    let r = region.rect;
    let mut out = Vec::with_capacity(sizes.len());
    match region.pack {
        Pack::FromRight | Pack::FromLeft => {
            // Lanes (columns) of stacked strips, filled from the Dock edge outward. Lanes share
            // the region's width so nothing spills past it; strips shrink if they must.
            if sizes.is_empty() {
                return out;
            }
            let h = sizes.iter().map(|s| s.1).fold(0.0, f64::max).min(r.h);
            let rows = (((r.h + PAD) / (h + PAD)).floor() as usize).max(1);
            let cols = sizes.len().div_ceil(rows);
            let widest = sizes.iter().map(|s| s.0).fold(0.0, f64::max);
            let lane_w = widest.min(((r.w - PAD * (cols as f64 - 1.0)) / cols as f64).max(20.0));
            let used = rows.min(sizes.len()) as f64 * (h + PAD) - PAD;
            let y0 = r.y + ((r.h - used) / 2.0).max(0.0);
            for (k, &(w, _)) in sizes.iter().enumerate() {
                let (col, row) = ((k / rows) as f64, (k % rows) as f64);
                let w = w.min(lane_w);
                let edge = col * (lane_w + PAD);
                let x = match region.pack {
                    Pack::FromRight => r.right() - edge - w,
                    _ => r.x + edge,
                };
                out.push(Rect::new(x, y0 + row * (h + PAD), w, h));
            }
        }
        Pack::FromBottom | Pack::FromTop => {
            let mut pos = 0.0;
            for &(w, h) in sizes {
                let w = w.min(r.w);
                let y = match region.pack {
                    Pack::FromBottom => r.bottom() - pos - h,
                    _ => r.y + pos,
                };
                out.push(Rect::new(r.x, y, w, h));
                pos += h + PAD;
            }
        }
    }
    out
}

/// The region (index) under a point, with a little slack so a drop near it still lands.
pub fn region_at(regions: &[DockRegion], px: f64, py: f64) -> Option<usize> {
    regions.iter().position(|d| d.rect.inset(-8.0).contains(px, py))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mac_bottom() -> ScreenInfo {
        // 1728x1117 screen, 25pt menu bar, 50pt Dock at the bottom.
        ScreenInfo { frame: Rect::new(0.0, 0.0, 1728.0, 1117.0), visible: Rect::new(0.0, 38.0, 1728.0, 1029.0) }
    }

    #[test]
    fn detects_bottom_dock() {
        assert_eq!(dock_side(&mac_bottom(), DockSide::Bottom), (DockSide::Bottom, 50.0));
    }

    #[test]
    fn regions_either_side_of_known_dock() {
        let dock = Rect::new(390.0, 1063.0, 948.0, 44.0);
        let r = default_regions(&mac_bottom(), DockSide::Bottom, Some(dock), 0.0);
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].id, "dock-left");
        assert_eq!(r[0].rect, Rect::new(4.0, 1067.0, 382.0, 50.0));
        assert_eq!(r[0].pack, Pack::FromRight);
        assert_eq!(r[1].rect, Rect::new(1342.0, 1067.0, 382.0, 50.0));
        assert!(r.iter().all(|d| d.on_top));
    }

    #[test]
    fn regions_from_estimate_are_centred() {
        let r = default_regions(&mac_bottom(), DockSide::Bottom, None, 728.0);
        assert_eq!(r[0].rect.right(), 500.0 - 4.0);
        assert_eq!(r[1].rect.x, 1228.0 + 4.0);
    }

    #[test]
    fn left_dock_regions_above_and_below() {
        let s = ScreenInfo { frame: Rect::new(0.0, 0.0, 1728.0, 1117.0), visible: Rect::new(60.0, 25.0, 1668.0, 1092.0) };
        let dock = Rect::new(0.0, 300.0, 60.0, 500.0);
        let r = default_regions(&s, DockSide::Bottom, Some(dock), 0.0);
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].id, "dock-above");
        assert_eq!(r[0].rect, Rect::new(0.0, 29.0, 60.0, 267.0));
        assert_eq!(r[1].rect, Rect::new(0.0, 804.0, 60.0, 309.0));
    }

    #[test]
    fn right_dock() {
        let s = ScreenInfo { frame: Rect::new(0.0, 0.0, 1000.0, 800.0), visible: Rect::new(0.0, 25.0, 940.0, 775.0) };
        let r = default_regions(&s, DockSide::Bottom, None, 400.0);
        assert_eq!(r[0].rect.x, 940.0);
        assert_eq!(r[0].pack, Pack::FromBottom);
    }

    #[test]
    fn autohide_uses_hint_and_default_strip() {
        let s = ScreenInfo { frame: Rect::new(0.0, 0.0, 1000.0, 800.0), visible: Rect::new(0.0, 25.0, 1000.0, 775.0) };
        let r = default_regions(&s, DockSide::Bottom, None, 400.0);
        assert_eq!(r[0].rect.h, AUTOHIDE_STRIP);
        assert_eq!(r[0].rect.bottom(), 800.0);
    }

    #[test]
    fn layout_packs_toward_dock_in_columns() {
        let region = DockRegion { id: "l".into(), rect: Rect::new(0.0, 1000.0, 400.0, 60.0), pack: Pack::FromRight, on_top: true };
        let rects = layout(&region, &[(150.0, 26.0), (100.0, 26.0), (120.0, 26.0)]);
        // Two rows fit (centred: 54pt used of 60). First lane against the right edge.
        assert_eq!(rects[0], Rect::new(250.0, 1003.0, 150.0, 26.0));
        assert_eq!(rects[1], Rect::new(300.0, 1031.0, 100.0, 26.0));
        // Third goes in the next lane outward (left of the 150 wide lane).
        assert_eq!(rects[2], Rect::new(400.0 - 150.0 - 2.0 - 120.0, 1003.0, 120.0, 26.0));
    }

    #[test]
    fn layout_centres_single_lane_and_clamps_width() {
        let region = DockRegion { id: "r".into(), rect: Rect::new(1000.0, 1000.0, 300.0, 50.0), pack: Pack::FromLeft, on_top: true };
        let rects = layout(&region, &[(360.0, 26.0)]);
        assert_eq!(rects[0].x, 1000.0);
        assert_eq!(rects[0].w, 300.0);
        assert_eq!(rects[0].y, 1000.0 + 12.0);
    }

    #[test]
    fn layout_shrinks_lanes_to_stay_inside_the_region() {
        // A 50pt strip holds one row; three 360-wide strips must share 382pt.
        let region = DockRegion { id: "r".into(), rect: Rect::new(1342.0, 1067.0, 382.0, 50.0), pack: Pack::FromLeft, on_top: true };
        let rects = layout(&region, &[(360.0, 26.0), (360.0, 26.0), (260.0, 26.0)]);
        for r in &rects {
            assert!(r.x >= 1342.0 && r.right() <= 1724.0 + 0.01, "{r:?} escapes the region");
            assert_eq!(r.y, 1067.0 + 12.0);
        }
        assert!(rects[0].right() <= rects[1].x && rects[1].right() <= rects[2].x);
    }

    #[test]
    fn region_hit_test_has_slack() {
        let regions = vec![DockRegion { id: "a".into(), rect: Rect::new(0.0, 100.0, 100.0, 50.0), pack: Pack::FromLeft, on_top: true }];
        assert_eq!(region_at(&regions, 50.0, 95.0), Some(0));
        assert_eq!(region_at(&regions, 50.0, 80.0), None);
    }
}
