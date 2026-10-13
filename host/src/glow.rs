//! Proximity feedback: the socket light's eased brightness (the prototype's `glowTick`) and
//! the status caption shown while dragging (its `#phase` labels). The target brightness
//! comes from `magnet::proximity`; this module only eases toward it and words the feel.
//! Pure logic, no windowing.

use crate::geom::Rect;
use crate::magnet::Feel;

/// Exponential time constant of the light's easing (ms). Not a total duration: the light
/// settles over several of these, quick but visible both ways.
pub const TAU_MS: f64 = 40.0;
/// Longest frame step the easing takes (ms).
pub const MAX_STEP_MS: f64 = 64.0;
/// Closer than this to the target, the light snaps onto it and stops animating.
pub const SNAP: f64 = 0.002;

/// Which end of a widget a socket is on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    pub fn opposite(self) -> Side {
        match self {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }
}

/// One socket light: the brightness shown now and the one the field asks for.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Glow {
    pub value: f64,
    pub target: f64,
}

impl Glow {
    /// Ease toward the target over `dt_ms`: `v += (target - v) * (1 - exp(-dt / 40 ms))`.
    /// Returns true while still easing.
    pub fn step(&mut self, dt_ms: f64) -> bool {
        let dt = dt_ms.clamp(0.0, MAX_STEP_MS);
        self.value += (self.target - self.value) * (1.0 - (-dt / TAU_MS).exp());
        if (self.target - self.value).abs() < SNAP {
            self.value = self.target;
        }
        !self.settled()
    }

    pub fn settled(&self) -> bool {
        self.value == self.target
    }
}

/// The caption while dragging, by feel; `None` when there is nothing to say.
pub fn caption(feel: Feel) -> Option<&'static str> {
    match feel {
        Feel::Free => None,
        Feel::Approach => Some("Socket nearby · move closer"),
        Feel::Captured => Some("Ready · release to join"),
    }
}

/// Where a caption `w`×`h` goes for a dragged widget at `widget`: centred under it, `GAP`
/// below, or above it if that would run off the bottom of `screen`, and kept within the
/// screen sideways.
pub const CAPTION_GAP: f64 = 6.0;

pub fn caption_rect(widget: Rect, w: f64, h: f64, screen: Option<Rect>) -> Rect {
    let mut x = widget.center_x() - w / 2.0;
    if let Some(s) = screen {
        x = x.min(s.right() - w).max(s.x);
    }
    let below = widget.bottom() + CAPTION_GAP;
    let y = if screen.is_some_and(|s| below + h > s.bottom()) { widget.y - CAPTION_GAP - h } else { below };
    Rect::new(x, y, w, h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::magnet::proximity;

    fn run(g: &mut Glow, ms: f64) {
        let mut t = 0.0;
        while t < ms {
            g.step(16.0);
            t += 16.0;
        }
    }

    #[test]
    fn one_step_follows_the_exponential() {
        let mut g = Glow { value: 0.0, target: 1.0 };
        assert!(g.step(40.0));
        assert!((g.value - (1.0 - (-1.0f64).exp())).abs() < 1e-12);
        // Long frames are capped at 64 ms.
        let mut a = Glow { value: 0.0, target: 1.0 };
        let mut b = a;
        a.step(64.0);
        b.step(500.0);
        assert_eq!(a, b);
    }

    #[test]
    fn snaps_onto_the_target_and_stops() {
        let mut g = Glow { value: 0.999, target: 1.0 };
        assert!(!g.step(16.0));
        assert_eq!(g.value, 1.0);
        assert!(g.settled());
        assert!(Glow::default().settled());
        let mut frames = 0;
        let mut g = Glow { value: 0.0, target: 1.0 };
        while g.step(16.0) {
            frames += 1;
        }
        // About 6 time constants: a quarter of a second, quick but noticeable.
        assert!((12..=18).contains(&frames), "{frames} frames");
    }

    // The prototype's proximity-curve check: entry eases, brightness tracks distance below
    // 0.8, the capture crossing eases, full is steady, exit fades rather than switching off.
    #[test]
    fn glow_curve_eases_entry_tracks_distance_plateaus_and_eases_exit() {
        let mut g = Glow { target: proximity(121.0).brightness, ..Default::default() };
        run(&mut g, 250.0);
        assert_eq!(g.value, 0.0);
        g.target = proximity(119.0).brightness;
        run(&mut g, 35.0);
        let entering = g.value;
        run(&mut g, 250.0);
        let dim = g.value;
        assert!(entering > 0.0 && entering < dim, "entry must ease: {entering} vs {dim}");
        g.target = proximity(75.0).brightness;
        run(&mut g, 250.0);
        let mid = g.value;
        g.target = proximity(41.0).brightness;
        run(&mut g, 250.0);
        let high = g.value;
        assert!(dim > 0.0 && dim < mid && mid < high && high < 0.8, "{dim} {mid} {high}");
        g.target = proximity(39.0).brightness;
        run(&mut g, 35.0);
        let transition = g.value;
        run(&mut g, 250.0);
        assert!(transition > high && transition < g.value, "capture crossing must ease");
        assert_eq!(g.value, 1.0);
        g.target = proximity(5.0).brightness;
        run(&mut g, 250.0);
        assert_eq!(g.value, 1.0, "steady inside capture");
        g.target = proximity(60.0).brightness;
        run(&mut g, 250.0);
        assert!(g.value < 0.8);
        g.target = proximity(121.0).brightness;
        run(&mut g, 35.0);
        assert!(g.value > 0.0, "exit must fade, not switch off");
        run(&mut g, 250.0);
        assert_eq!(g.value, 0.0);
    }

    #[test]
    fn captions_follow_the_feel() {
        assert_eq!(caption(Feel::Free), None);
        assert_eq!(caption(Feel::Approach), Some("Socket nearby · move closer"));
        assert_eq!(caption(Feel::Captured), Some("Ready · release to join"));
    }

    #[test]
    fn caption_sits_centred_under_the_widget_or_above_near_the_bottom() {
        let w = Rect::new(100.0, 200.0, 360.0, 116.0);
        let screen = Rect::new(0.0, 25.0, 1440.0, 850.0);
        let r = caption_rect(w, 200.0, 18.0, Some(screen));
        assert_eq!(r, Rect::new(180.0, 316.0 + CAPTION_GAP, 200.0, 18.0));
        let low = Rect::new(100.0, 760.0, 360.0, 116.0);
        let r = caption_rect(low, 200.0, 18.0, Some(screen));
        assert_eq!(r, Rect::new(180.0, 760.0 - CAPTION_GAP - 18.0, 200.0, 18.0));
        assert_eq!(caption_rect(low, 200.0, 18.0, None).y, 876.0 + CAPTION_GAP);
        // Kept on screen sideways.
        let edge = Rect::new(-100.0, 200.0, 360.0, 116.0);
        assert_eq!(caption_rect(edge, 200.0, 18.0, Some(screen)).x, 0.0);
        assert_eq!(caption_rect(edge, 200.0, 18.0, None).x, -20.0);
    }
}
