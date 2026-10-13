//! The magnetic field around a socket (the prototype's `proximity`, `magneticGoal` and
//! `runDragFrame`): one spatial rule for every connector, no cooldown, escape gate or
//! capture history. Pure maths; the model feeds it poses and the shell drives the frames.

use crate::geom::Rect;
use crate::glow::Side;

/// Outside this distance (points) from the aligned pose the drag is free.
pub const NEAR: f64 = 120.0;
/// Inside this distance the drag is captured: releasing joins.
pub const CAPTURE: f64 = 40.0;
/// Pull strength while captured.
pub const CAPTURED_PULL: f64 = 0.88;
/// Gap between two capsules lined up socket to socket (their inner caps still extended).
pub const SOCKET_GAP: f64 = 12.0;
/// Longest frame step the spring integrates (seconds).
pub const MAX_DT: f64 = 0.032;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Feel {
    /// Farther than `NEAR`: 1:1 with the pointer.
    Free,
    /// Between `NEAR` and `CAPTURE`: pulled, releasing does not join.
    Approach,
    /// Within `CAPTURE`: releasing joins.
    Captured,
}

impl Feel {
    pub fn name(self) -> &'static str {
        match self {
            Feel::Free => "free",
            Feel::Approach => "approach",
            Feel::Captured => "captured",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Proximity {
    pub feel: Feel,
    pub distance: f64,
    /// Socket light brightness 0..1 this distance asks for.
    pub brightness: f64,
}

pub fn proximity(distance: f64) -> Proximity {
    if !(distance <= NEAR) {
        return Proximity { feel: Feel::Free, distance, brightness: 0.0 };
    }
    if distance <= CAPTURE {
        return Proximity { feel: Feel::Captured, distance, brightness: 1.0 };
    }
    let approach = (NEAR - distance) / (NEAR - CAPTURE);
    Proximity { feel: Feel::Approach, distance, brightness: 0.18 + 0.44 * approach.powf(0.8) }
}

/// How far toward the target the goal sits (0 = at the pointer, 1 = on the target).
pub fn strength(p: &Proximity) -> f64 {
    match p.feel {
        Feel::Free => 0.0,
        Feel::Approach => (1.0 - p.distance / NEAR).powf(0.8) * 0.85,
        Feel::Captured => CAPTURED_PULL,
    }
}

/// Where the dragged thing should head for: `raw` (pointer-driven) pulled toward `target`.
pub fn goal(raw: (f64, f64), target: Option<(f64, f64)>) -> ((f64, f64), Proximity) {
    let Some(t) = target else { return (raw, proximity(f64::INFINITY)) };
    let p = proximity((t.0 - raw.0).hypot(t.1 - raw.1));
    let s = strength(&p);
    ((raw.0 + (t.0 - raw.0) * s, raw.1 + (t.1 - raw.1) * s), p)
}

/// Critically damped spring toward a goal: `v += ((goal - x) * 400 - v * 40) * dt` in three
/// sub-steps per frame, `dt` capped at 32 ms. Settles without a bounce.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Spring {
    pub pos: (f64, f64),
    pub vel: (f64, f64),
}

impl Spring {
    /// Advance one frame of `dt` seconds. Returns true once at rest on the goal.
    pub fn step(&mut self, goal: (f64, f64), dt: f64) -> bool {
        let h = dt.clamp(0.0, MAX_DT) / 3.0;
        for _ in 0..3 {
            self.vel.0 += ((goal.0 - self.pos.0) * 400.0 - self.vel.0 * 40.0) * h;
            self.vel.1 += ((goal.1 - self.pos.1) * 400.0 - self.vel.1 * 40.0) * h;
            self.pos.0 += self.vel.0 * h;
            self.pos.1 += self.vel.1 * h;
        }
        if (goal.0 - self.pos.0).abs() < 0.03 && self.vel.0.abs() < 0.1 {
            self.pos.0 = goal.0;
            self.vel.0 = 0.0;
        }
        if (goal.1 - self.pos.1).abs() < 0.03 && self.vel.1.abs() < 0.1 {
            self.pos.1 = goal.1;
            self.vel.1 = 0.0;
        }
        self.settled(goal)
    }

    pub fn settled(&self, goal: (f64, f64)) -> bool {
        self.pos == goal && self.vel == (0.0, 0.0)
    }
}

/// A body that may carry capsule end sockets: `left`/`right` say whether that end is free.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Body {
    pub rect: Rect,
    pub left: bool,
    pub right: bool,
}

/// The closest socket pairing between a moving set and the bodies around it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pairing {
    /// Index into the moving bodies.
    pub member: usize,
    /// Index into the other bodies.
    pub other: usize,
    /// The moving body's end that pairs (the other body's opposite end faces it).
    pub side: Side,
    /// Add to every moving body to reach the aligned pose (`SOCKET_GAP` apart).
    pub aligned: (f64, f64),
    /// Add to every moving body to reach the joined pose (flush, sharing the socket).
    pub joined: (f64, f64),
    pub distance: f64,
}

/// For each free socket on a moving body facing a free socket on another body, the aligned
/// pose puts the two ends `SOCKET_GAP` apart with their centres level. Returns the nearest.
pub fn nearest_pairing(moving: &[Body], others: &[Body]) -> Option<Pairing> {
    let mut best: Option<Pairing> = None;
    for (k, m) in moving.iter().enumerate() {
        for (j, o) in others.iter().enumerate() {
            let dy = o.rect.y + (o.rect.h - m.rect.h) / 2.0 - m.rect.y;
            // m's right end to o's left end, and m's left end to o's right end.
            let sides = [
                (m.right && o.left, o.rect.x - m.rect.w - m.rect.x, -SOCKET_GAP, Side::Right),
                (m.left && o.right, o.rect.right() - m.rect.x, SOCKET_GAP, Side::Left),
            ];
            for (ok, flush_dx, gap, side) in sides {
                if !ok {
                    continue;
                }
                let aligned = (flush_dx + gap, dy);
                let distance = aligned.0.hypot(aligned.1);
                if best.is_none_or(|b| distance < b.distance) {
                    best = Some(Pairing { member: k, other: j, side, aligned, joined: (flush_dx, dy), distance });
                }
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn bands_and_brightness() {
        assert_eq!(proximity(121.0).feel, Feel::Free);
        assert_eq!(proximity(121.0).brightness, 0.0);
        assert_eq!(proximity(120.0).feel, Feel::Approach);
        assert_eq!(proximity(41.0).feel, Feel::Approach);
        assert_eq!(proximity(40.0).feel, Feel::Captured);
        assert_eq!(proximity(40.0).brightness, 1.0);
        assert_eq!(proximity(f64::INFINITY).feel, Feel::Free);
        assert_eq!(proximity(f64::NAN).feel, Feel::Free);
        // The spec's measured glow: 0.25 at 110 px (the formula gives 0.263), 0.53 at 60 px.
        assert!(close(proximity(110.0).brightness, 0.263, 0.001));
        assert!(close(proximity(60.0).brightness, 0.53, 0.01));
    }

    #[test]
    fn pull_strength_at_110_60_and_25() {
        // The spec measured 11 px closer at 110 px (the formula gives 12.8) and 29 at 60 px.
        let s110 = strength(&proximity(110.0));
        assert!(close(s110, (1.0f64 - 110.0 / 120.0).powf(0.8) * 0.85, 1e-12));
        assert!(close(s110 * 110.0, 12.8, 0.05), "{}", s110 * 110.0);
        let s60 = strength(&proximity(60.0));
        assert!(close(s60 * 60.0, 29.0, 0.5), "{}", s60 * 60.0);
        assert_eq!(strength(&proximity(25.0)), 0.88);
        assert_eq!(strength(&proximity(130.0)), 0.0);
    }

    #[test]
    fn goal_is_raw_outside_the_field_and_pulled_inside() {
        assert_eq!(goal((0.0, 0.0), Some((130.0, 0.0))).0, (0.0, 0.0));
        assert_eq!(goal((5.0, 5.0), None).0, (5.0, 5.0));
        let (g, p) = goal((0.0, 0.0), Some((0.0, 25.0)));
        assert_eq!(p.feel, Feel::Captured);
        assert!(close(g.1, 22.0, 1e-9));
        let (g, _) = goal((0.0, 0.0), Some((60.0, 0.0)));
        assert!(close(g.0, 29.0, 0.5));
    }

    #[test]
    fn no_hysteresis_same_distance_same_pull() {
        // Approach, retreat, approach again: the goal depends only on the current distance.
        let t = Some((100.0, 0.0));
        let first = goal((0.0, 0.0), t);
        let _ = goal((80.0, 0.0), t);
        let _ = goal((-200.0, 0.0), t);
        assert_eq!(goal((0.0, 0.0), t), first);
    }

    #[test]
    fn spring_settles_on_the_goal_without_overshoot() {
        let mut s = Spring::default();
        let g = (30.0, -12.0);
        let mut frames = 0;
        while !s.step(g, 0.016) {
            assert!(s.pos.0 <= g.0 + 1e-9 && s.pos.1 >= g.1 - 1e-9, "overshot at frame {frames}: {:?}", s.pos);
            frames += 1;
            assert!(frames < 120, "never settled");
        }
        assert_eq!(s.pos, g);
        assert_eq!(s.vel, (0.0, 0.0));
        // Critically damped (ω = 20/s): at rest within the 0.03 pt snap in about 0.6 s.
        assert!(frames < 45, "{frames} frames");
    }

    #[test]
    fn spring_caps_long_frames() {
        let mut a = Spring::default();
        let mut b = Spring::default();
        a.step((50.0, 0.0), 0.032);
        b.step((50.0, 0.0), 0.5);
        assert_eq!(a, b);
    }

    fn body(x: f64, y: f64, w: f64, h: f64) -> Body {
        Body { rect: Rect::new(x, y, w, h), left: true, right: true }
    }

    #[test]
    fn aligned_pose_is_12_beyond_the_partner_end() {
        // A at 142,215 (390 wide); B dragged to 600,230. B's aligned pose is A.x + W + 12.
        let a = body(142.0, 215.0, 390.0, 130.0);
        let b = body(600.0, 230.0, 390.0, 130.0);
        let p = nearest_pairing(&[b], &[a]).unwrap();
        assert_eq!((600.0 + p.aligned.0, 230.0 + p.aligned.1), (142.0 + 390.0 + 12.0, 215.0));
        assert_eq!((600.0 + p.joined.0, 230.0 + p.joined.1), (142.0 + 390.0, 215.0));
        assert!(close(p.distance, 56.0f64.hypot(15.0), 1e-9));
        assert_eq!(p.side, Side::Left, "B's left end meets A's right end");
        // Dragging A toward B's left end mirrors it.
        let p = nearest_pairing(&[body(100.0, 215.0, 390.0, 130.0)], &[body(700.0, 215.0, 390.0, 130.0)]).unwrap();
        assert_eq!(100.0 + p.aligned.0, 700.0 - 12.0 - 390.0);
        assert_eq!(p.side, Side::Right);
    }

    #[test]
    fn centres_line_up_for_different_heights() {
        let a = body(0.0, 100.0, 300.0, 150.0);
        let b = body(400.0, 0.0, 300.0, 100.0);
        let p = nearest_pairing(&[b], &[a]).unwrap();
        assert_eq!(p.aligned, (312.0 - 400.0, 125.0));
    }

    #[test]
    fn only_free_facing_sockets_pair() {
        let a = Body { right: false, ..body(0.0, 0.0, 300.0, 100.0) };
        let b = body(330.0, 0.0, 300.0, 100.0);
        // a's right end is taken, so b can only line up on a's left end, far away.
        let p = nearest_pairing(&[b], &[a]).unwrap();
        assert_eq!(p.aligned.0, -12.0 - 300.0 - 330.0);
        let none = Body { left: false, right: false, ..b };
        assert_eq!(nearest_pairing(&[none], &[a]), None);
    }

    #[test]
    fn picks_the_nearest_pairing_across_members() {
        let moving = [body(0.0, 0.0, 300.0, 100.0), body(300.0, 0.0, 300.0, 100.0)];
        let others = [body(-500.0, 0.0, 300.0, 100.0), body(640.0, 10.0, 300.0, 100.0)];
        let p = nearest_pairing(&moving, &others).unwrap();
        assert_eq!((p.member, p.other), (1, 1));
        assert_eq!(p.aligned, (640.0 - 12.0 - 600.0, 10.0));
    }
}
