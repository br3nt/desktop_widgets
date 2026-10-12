//! Snapping maths: pull a moving set of rects onto nearby widget edges and screen edges,
//! and detect which rects end up flush against each other (the join test).

use crate::geom::Rect;

/// Distance (points) inside which edges snap.
pub const SNAP_THRESHOLD: f64 = 12.0;
/// Two edges closer than this count as touching.
const TOUCH_EPS: f64 = 0.75;
/// Perpendicular overlap needed for a flush contact to count as a join.
const MIN_JOIN_OVERLAP: f64 = 8.0;

fn best(cands: impl Iterator<Item = f64>, threshold: f64) -> Option<f64> {
    cands
        .filter(|d| d.abs() <= threshold)
        .min_by(|a, b| a.abs().partial_cmp(&b.abs()).unwrap())
}

/// Returns the (dx, dy) correction to apply to every moving rect.
/// `targets` are other widgets' rects; `screens` are usable screen areas (snap to their inner edges).
pub fn snap_offset(moving: &[Rect], targets: &[Rect], screens: &[Rect], threshold: f64) -> (f64, f64) {
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for m in moving {
        for t in targets {
            // Only consider edges that are near each other in the perpendicular direction.
            let v_near = Rect::overlap(m.y, m.bottom(), t.y - threshold, t.bottom() + threshold) > 0.0;
            let h_near = Rect::overlap(m.x, m.right(), t.x - threshold, t.right() + threshold) > 0.0;
            if v_near {
                xs.extend([t.right() - m.x, t.x - m.right(), t.x - m.x, t.right() - m.right()]);
            }
            if h_near {
                ys.extend([t.bottom() - m.y, t.y - m.bottom(), t.y - m.y, t.bottom() - m.bottom()]);
            }
        }
        for s in screens {
            if Rect::overlap(m.x, m.right(), s.x, s.right()) > -threshold
                && Rect::overlap(m.y, m.bottom(), s.y, s.bottom()) > -threshold
            {
                xs.extend([s.x - m.x, s.right() - m.right()]);
                ys.extend([s.y - m.y, s.bottom() - m.bottom()]);
            }
        }
    }
    (
        best(xs.into_iter(), threshold).unwrap_or(0.0),
        best(ys.into_iter(), threshold).unwrap_or(0.0),
    )
}

/// True when `a` and `b` share an edge (flush, not overlapping) with enough perpendicular overlap.
pub fn touching(a: &Rect, b: &Rect) -> bool {
    let side = ((a.right() - b.x).abs() < TOUCH_EPS || (a.x - b.right()).abs() < TOUCH_EPS)
        && a.v_overlap(b) >= MIN_JOIN_OVERLAP;
    let stacked = ((a.bottom() - b.y).abs() < TOUCH_EPS || (a.y - b.bottom()).abs() < TOUCH_EPS)
        && a.h_overlap(b) >= MIN_JOIN_OVERLAP;
    side || stacked
}

/// Pairs (moving index, target index) that touch.
pub fn contacts(moving: &[Rect], targets: &[Rect]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for (i, m) in moving.iter().enumerate() {
        for (j, t) in targets.iter().enumerate() {
            if touching(m, t) {
                out.push((i, j));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: f64 = SNAP_THRESHOLD;

    #[test]
    fn no_snap_when_far() {
        let m = [Rect::new(500.0, 500.0, 100.0, 40.0)];
        let t = [Rect::new(0.0, 0.0, 100.0, 40.0)];
        assert_eq!(snap_offset(&m, &t, &[], T), (0.0, 0.0));
    }

    #[test]
    fn snaps_left_edge_to_right_edge_of_neighbour() {
        // Neighbour ends at x=100; moving starts at 108, roughly level.
        let m = [Rect::new(108.0, 3.0, 100.0, 40.0)];
        let t = [Rect::new(0.0, 0.0, 100.0, 40.0)];
        let (dx, dy) = snap_offset(&m, &t, &[], T);
        assert_eq!(dx, -8.0);
        assert_eq!(dy, -3.0, "tops align too");
        let snapped = m[0].offset(dx, dy);
        assert!(touching(&snapped, &t[0]));
    }

    #[test]
    fn does_not_snap_x_when_vertically_far() {
        let m = [Rect::new(104.0, 300.0, 100.0, 40.0)];
        let t = [Rect::new(0.0, 0.0, 100.0, 40.0)];
        assert_eq!(snap_offset(&m, &t, &[], T).0, 0.0);
    }

    #[test]
    fn stacks_below() {
        let m = [Rect::new(5.0, 50.0, 100.0, 26.0)];
        let t = [Rect::new(0.0, 0.0, 100.0, 40.0)];
        let (dx, dy) = snap_offset(&m, &t, &[], T);
        assert_eq!((dx, dy), (-5.0, -10.0));
        assert!(touching(&m[0].offset(dx, dy), &t[0]));
    }

    #[test]
    fn snaps_to_screen_edges() {
        let screen = [Rect::new(0.0, 25.0, 1000.0, 700.0)];
        let m = [Rect::new(7.0, 30.0, 100.0, 40.0)];
        assert_eq!(snap_offset(&m, &[], &screen, T), (-7.0, -5.0));
        let m = [Rect::new(895.0, 300.0, 100.0, 40.0)];
        assert_eq!(snap_offset(&m, &[], &screen, T), (5.0, 0.0));
    }

    #[test]
    fn group_snaps_by_its_closest_member() {
        let m = [Rect::new(300.0, 0.0, 100.0, 40.0), Rect::new(400.0, 0.0, 100.0, 40.0)];
        let t = [Rect::new(503.0, 0.0, 100.0, 40.0)];
        assert_eq!(snap_offset(&m, &t, &[], T).0, 3.0);
    }

    #[test]
    fn touching_requires_overlap_and_flush() {
        let a = Rect::new(0.0, 0.0, 100.0, 40.0);
        assert!(touching(&a, &Rect::new(100.0, 10.0, 50.0, 40.0)));
        assert!(!touching(&a, &Rect::new(101.5, 10.0, 50.0, 40.0)), "gap");
        assert!(!touching(&a, &Rect::new(100.0, 38.0, 50.0, 40.0)), "corner only");
        assert!(!touching(&a, &Rect::new(50.0, 10.0, 100.0, 40.0)), "overlapping");
        assert_eq!(contacts(&[a], &[Rect::new(0.0, 40.0, 100.0, 26.0)]), vec![(0, 0)]);
    }
}
