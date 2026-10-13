//! The arrangement model: where widgets are, which group they belong to, shade state,
//! dock membership and per-group always-on-top. No windowing code here; the shell
//! reads `World` and moves real windows to match.

use crate::docks::{self, DockRegion};
use crate::geom::Rect;
use crate::magnet::{self, Body, Feel, Pairing, Proximity, Spring};
use crate::snap;
use std::collections::BTreeMap;

/// Height of a widget in shade mode.
pub const SHADE_H: f64 = 26.0;
/// Pointer travel before a press becomes a drag.
pub const DRAG_SLOP: f64 = 3.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Placed {
    pub id: String,
    pub x: f64,
    pub y: f64,
    /// Unshaded size.
    pub w: f64,
    pub h: f64,
    pub shaded: bool,
    pub group: u32,
    /// Dock region id when docked.
    pub dock: Option<String>,
    /// Shade state to restore when leaving a dock.
    pub shaded_before_dock: bool,
    /// Width granted by a dock layout (may be narrower than `w`).
    pub dock_w: Option<f64>,
    /// Has capsule end sockets (can join side by side). Checks-style cartridges don't.
    pub sockets: bool,
}

impl Placed {
    pub fn new(id: &str, x: f64, y: f64, w: f64, h: f64, group: u32) -> Self {
        Placed { id: id.into(), x, y, w, h, shaded: false, group, dock: None, shaded_before_dock: false, dock_w: None, sockets: true }
    }
    pub fn rect(&self) -> Rect {
        let w = self.dock_w.unwrap_or(self.w);
        Rect::new(self.x, self.y, w, if self.shaded { SHADE_H } else { self.h })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Drag {
    pub members: Vec<usize>,
    origins: Vec<(f64, f64)>,
    start: (f64, f64),
    pub moved: bool,
    /// Dock region index the pointer is over.
    pub hover_dock: Option<usize>,
    /// The widget that was pressed.
    pub pressed: usize,
    /// Last pointer position seen by the drag.
    pub cursor: (f64, f64),
    /// Pointer-driven offset from the origins.
    pub raw: (f64, f64),
    /// Offset the spring is heading for.
    pub goal: (f64, f64),
    /// The offset applied now (`pos`) and its velocity.
    pub spring: Spring,
    pub feel: Feel,
    /// Socket field: (other widget, pairing) and its proximity, when a socket is in range.
    pub pairing: Option<(usize, Pairing)>,
    pub proximity: Proximity,
}

#[derive(Clone, Debug, Default)]
pub struct World {
    pub widgets: Vec<Placed>,
    /// group id -> always on top
    pub groups: BTreeMap<u32, bool>,
    pub docks: Vec<DockRegion>,
    /// Usable screen areas (snap targets).
    pub screens: Vec<Rect>,
    pub drag: Option<Drag>,
}

/// What a finished drag did, so the shell can react (e.g. save, play an effect).
#[derive(Debug, PartialEq)]
pub enum DropOutcome {
    Moved,
    Joined,
    Docked(String),
    Clicked,
}

impl World {
    pub fn members(&self, group: u32) -> Vec<usize> {
        (0..self.widgets.len()).filter(|&i| self.widgets[i].group == group).collect()
    }

    pub fn next_group(&self) -> u32 {
        self.widgets.iter().map(|w| w.group).chain(self.groups.keys().copied()).max().unwrap_or(0) + 1
    }

    pub fn group_size(&self, i: usize) -> usize {
        self.members(self.widgets[i].group).len()
    }

    /// Effective always-on-top: the dock's when docked, else the group's.
    pub fn on_top(&self, i: usize) -> bool {
        let w = &self.widgets[i];
        if let Some(d) = &w.dock {
            if let Some(r) = self.docks.iter().find(|r| &r.id == d) {
                return r.on_top;
            }
        }
        *self.groups.get(&w.group).unwrap_or(&false)
    }

    pub fn toggle_on_top(&mut self, i: usize) {
        if let Some(d) = self.widgets[i].dock.clone() {
            if let Some(r) = self.docks.iter_mut().find(|r| r.id == d) {
                r.on_top = !r.on_top;
                return;
            }
        }
        let g = self.widgets[i].group;
        let v = !self.on_top(i);
        self.groups.insert(g, v);
    }

    /// Move `i` into a group of its own (keeps its on-top setting).
    pub fn detach(&mut self, i: usize) {
        if self.group_size(i) < 2 {
            return;
        }
        let on_top = self.on_top(i);
        let g = self.next_group();
        self.widgets[i].group = g;
        self.groups.insert(g, on_top);
    }

    fn merge_groups(&mut self, keep: u32, gone: u32) {
        if keep == gone {
            return;
        }
        for w in &mut self.widgets {
            if w.group == gone {
                w.group = keep;
            }
        }
        self.groups.remove(&gone);
    }

    /// Shade or unshade one widget; group members below it slide to stay flush.
    pub fn set_shaded(&mut self, i: usize, shaded: bool) {
        let w = &self.widgets[i];
        if w.shaded == shaded || w.dock.is_some() {
            return;
        }
        let old = w.rect();
        self.widgets[i].shaded = shaded;
        let new = self.widgets[i].rect();
        let delta = new.h - old.h;
        // Move what hangs below it: free group members flush under its old bottom edge
        // (overlapping horizontally), then whatever hangs under those, each once.
        let g = self.widgets[i].group;
        let mut moving = vec![false; self.widgets.len()];
        let mut frontier = vec![old];
        while let Some(above) = frontier.pop() {
            for j in 0..self.widgets.len() {
                let p = &self.widgets[j];
                if j == i || moving[j] || p.group != g || p.dock.is_some() {
                    continue;
                }
                let r = p.rect();
                if (r.y - above.bottom()).abs() < 0.75 && r.h_overlap(&above) > 0.0 {
                    moving[j] = true;
                    frontier.push(r);
                }
            }
        }
        for (j, m) in moving.into_iter().enumerate() {
            if m {
                self.widgets[j].y += delta;
            }
        }
    }

    pub fn toggle_shade(&mut self, i: usize) {
        let s = !self.widgets[i].shaded;
        self.set_shaded(i, s);
    }

    pub fn shade_all(&mut self, shaded: bool) {
        // Shade bottom-most first so slides compose.
        let mut order: Vec<usize> = (0..self.widgets.len()).collect();
        order.sort_by(|a, b| self.widgets[*b].y.total_cmp(&self.widgets[*a].y));
        if !shaded {
            order.reverse();
        }
        for i in order {
            self.set_shaded(i, shaded);
        }
    }

    // ---- docks ----

    pub fn dock_members(&self, dock: &str) -> Vec<usize> {
        (0..self.widgets.len()).filter(|&i| self.widgets[i].dock.as_deref() == Some(dock)).collect()
    }

    /// Re-run every dock's layout.
    pub fn arrange_docks(&mut self) {
        for d in 0..self.docks.len() {
            let region = self.docks[d].clone();
            let members = self.dock_members(&region.id);
            let sizes: Vec<(f64, f64)> = members.iter().map(|&i| (self.widgets[i].w, SHADE_H)).collect();
            let rects = docks::layout(&region, &sizes);
            for (k, &i) in members.iter().enumerate() {
                let w = &mut self.widgets[i];
                w.x = rects[k].x;
                w.y = rects[k].y;
                w.shaded = true;
                w.dock_w = if rects[k].w < w.w { Some(rects[k].w) } else { None };
            }
        }
        // Widgets whose dock no longer exists become free again.
        let ids: Vec<String> = self.docks.iter().map(|d| d.id.clone()).collect();
        for i in 0..self.widgets.len() {
            if let Some(d) = &self.widgets[i].dock {
                if !ids.contains(d) {
                    self.undock(i);
                }
            }
        }
    }

    pub fn dock(&mut self, i: usize, dock: &str) {
        if self.widgets[i].dock.as_deref() == Some(dock) {
            return;
        }
        if self.widgets[i].dock.is_none() {
            self.widgets[i].shaded_before_dock = self.widgets[i].shaded;
        }
        // A docked widget leaves its group: the dock owns its placement.
        self.detach(i);
        self.widgets[i].dock = Some(dock.to_string());
    }

    pub fn undock(&mut self, i: usize) {
        let w = &mut self.widgets[i];
        if w.dock.take().is_some() {
            w.shaded = w.shaded_before_dock;
            w.dock_w = None;
        }
    }

    pub fn set_docks(&mut self, docks: Vec<DockRegion>) {
        // Keep the user's on-top choice for regions that survive a refresh.
        let mut docks = docks;
        for d in &mut docks {
            if let Some(old) = self.docks.iter().find(|o| o.id == d.id) {
                d.on_top = old.on_top;
            }
        }
        self.docks = docks;
        self.arrange_docks();
    }

    // ---- dragging ----

    /// Press on widget `i`'s casing. `solo` detaches it from its group first (Option-drag).
    pub fn begin_drag(&mut self, i: usize, cursor: (f64, f64), solo: bool) {
        if solo {
            self.detach(i);
        }
        let g = self.widgets[i].group;
        let members = if self.widgets[i].dock.is_some() { vec![i] } else { self.members(g).into_iter().filter(|&m| self.widgets[m].dock.is_none()).collect() };
        let origins = members.iter().map(|&m| (self.widgets[m].x, self.widgets[m].y)).collect();
        self.drag = Some(Drag {
            members,
            origins,
            start: cursor,
            moved: false,
            hover_dock: None,
            pressed: i,
            cursor,
            raw: (0.0, 0.0),
            goal: (0.0, 0.0),
            spring: Spring::default(),
            feel: Feel::Free,
            pairing: None,
            proximity: magnet::proximity(f64::INFINITY),
        });
    }

    pub fn drag_to(&mut self, cursor: (f64, f64)) {
        let Some(mut d) = self.drag.take() else { return };
        d.cursor = cursor;
        let (dx, dy) = (cursor.0 - d.start.0, cursor.1 - d.start.1);
        if !d.moved && dx.hypot(dy) < DRAG_SLOP {
            self.drag = Some(d);
            return;
        }
        if !d.moved {
            d.moved = true;
            // Pulling a docked widget out undocks it; keep it under the pointer at its free size.
            let p = d.pressed;
            if self.widgets[p].dock.is_some() {
                let before = self.widgets[p].rect();
                self.undock(p);
                self.arrange_docks();
                let after = self.widgets[p].rect();
                let fx = ((d.start.0 - before.x) / before.w).clamp(0.0, 1.0);
                d.origins[0] = (d.start.0 - fx * after.w, d.start.1 - (d.start.1 - before.y).min(after.h - 4.0));
            }
        }
        d.raw = (dx, dy);
        // The socket field: pull toward the nearest free facing socket's aligned pose.
        let at = |w: &World, o: (f64, f64)| -> Vec<Body> {
            d.members.iter().enumerate().map(|(k, &m)| Body { rect: w.widgets[m].rect().offset(d.origins[k].0 + o.0 - w.widgets[m].x, d.origins[k].1 + o.1 - w.widgets[m].y), ..w.body(m) }).collect()
        };
        let others: Vec<usize> = (0..self.widgets.len()).filter(|j| !d.members.contains(j) && self.widgets[*j].dock.is_none()).collect();
        let bodies: Vec<Body> = others.iter().map(|&j| self.body(j)).collect();
        let pairing = magnet::nearest_pairing(&at(self, d.raw), &bodies);
        let target = pairing.map(|p| (d.raw.0 + p.aligned.0, d.raw.1 + p.aligned.1));
        let (goal, prox) = magnet::goal(d.raw, target);
        d.pairing = pairing.map(|p| (others[p.other], p));
        d.proximity = prox;
        d.feel = prox.feel;
        d.goal = goal;
        if d.feel == Feel::Free {
            // Away from a field the drag is 1:1 with the pointer (edge snapping aside).
            let moving: Vec<Rect> = at(self, d.raw).iter().map(|b| b.rect).collect();
            let targets: Vec<Rect> = others.iter().map(|&j| self.widgets[j].rect()).collect();
            let (sx, sy) = snap::snap_offset(&moving, &targets, &self.screens, snap::SNAP_THRESHOLD);
            d.goal = (d.raw.0 + sx, d.raw.1 + sy);
            d.spring = Spring { pos: d.goal, vel: (0.0, 0.0) };
        }
        self.place(&d);
        d.hover_dock = docks::region_at(&self.docks, cursor.0, cursor.1);
        self.drag = Some(d);
    }

    pub fn end_drag(&mut self) -> Option<DropOutcome> {
        let d = self.drag.take()?;
        if !d.moved {
            return Some(DropOutcome::Clicked);
        }
        if let Some(r) = d.hover_dock {
            let id = self.docks[r].id.clone();
            // Dock in left-to-right order so the group keeps its reading order.
            let mut ms = d.members.clone();
            ms.sort_by(|a, b| self.widgets[*a].x.total_cmp(&self.widgets[*b].x));
            for m in ms {
                self.dock(m, &id);
            }
            self.arrange_docks();
            return Some(DropOutcome::Docked(id));
        }
        let Some((other, p)) = d.pairing.filter(|_| d.feel == Feel::Captured) else {
            return Some(DropOutcome::Moved);
        };
        // Release while captured: seat flush at the shared socket and become one group.
        for (k, &m) in d.members.iter().enumerate() {
            self.widgets[m].x = d.origins[k].0 + d.raw.0 + p.joined.0;
            self.widgets[m].y = d.origins[k].1 + d.raw.1 + p.joined.1;
        }
        let keep = self.widgets[d.members[0]].group;
        let gone = self.widgets[other].group;
        self.merge_groups(keep, gone);
        Some(DropOutcome::Joined)
    }

    /// Put the drag's members at their origins plus the spring's offset.
    fn place(&mut self, d: &Drag) {
        for (k, &m) in d.members.iter().enumerate() {
            self.widgets[m].x = d.origins[k].0 + d.spring.pos.0;
            self.widgets[m].y = d.origins[k].1 + d.spring.pos.1;
        }
    }

    /// Widget `i` as a socket body: which ends are free to join.
    fn body(&self, i: usize) -> Body {
        let p = &self.widgets[i];
        let r = p.rect();
        let open = p.sockets && !p.shaded && p.dock.is_none();
        // An end is taken when another (undocked) widget sits flush against it.
        let taken = |left: bool| {
            (0..self.widgets.len()).any(|j| {
                let q = self.widgets[j].rect();
                j != i && self.widgets[j].dock.is_none() && q.v_overlap(&r) > 0.0 && (if left { (q.right() - r.x).abs() } else { (q.x - r.right()).abs() }) < 0.75
            })
        };
        Body { rect: r, left: open && !taken(true), right: open && !taken(false) }
    }

    /// True while the drag spring still has frames to run.
    pub fn in_flight(&self) -> bool {
        self.drag.as_ref().is_some_and(|d| !d.spring.settled(d.goal))
    }

    /// Advance the drag spring by one frame of `dt` seconds. Returns `in_flight()`.
    pub fn tick(&mut self, dt: f64) -> bool {
        let Some(mut d) = self.drag.take() else { return false };
        if !d.spring.settled(d.goal) {
            d.spring.step(d.goal, dt);
            self.place(&d);
        }
        self.drag = Some(d);
        self.in_flight()
    }

    pub fn cancel_drag(&mut self) {
        if let Some(d) = self.drag.take() {
            for (k, &m) in d.members.iter().enumerate() {
                self.widgets[m].x = d.origins[k].0;
                self.widgets[m].y = d.origins[k].1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::docks::Pack;
    use crate::magnet::{self, Feel};

    fn world() -> World {
        let mut w = World::default();
        w.widgets.push(Placed::new("a", 100.0, 100.0, 300.0, 100.0, 1));
        w.widgets.push(Placed::new("b", 600.0, 100.0, 300.0, 100.0, 2));
        w.widgets.push(Placed::new("c", 100.0, 600.0, 200.0, 80.0, 3));
        w.groups.insert(1, false);
        w.groups.insert(2, false);
        w.groups.insert(3, false);
        w
    }

    fn drag(w: &mut World, i: usize, from: (f64, f64), to: (f64, f64), solo: bool) -> Option<DropOutcome> {
        w.begin_drag(i, from, solo);
        w.drag_to(((from.0 + to.0) / 2.0, (from.1 + to.1) / 2.0));
        w.drag_to(to);
        w.end_drag()
    }

    #[test]
    fn press_without_moving_is_a_click() {
        let mut w = world();
        w.begin_drag(0, (150.0, 110.0), false);
        w.drag_to((151.0, 111.0));
        assert_eq!(w.end_drag(), Some(DropOutcome::Clicked));
        assert_eq!((w.widgets[0].x, w.widgets[0].y), (100.0, 100.0));
    }

    #[test]
    fn releasing_captured_joins_flush_at_the_socket() {
        let mut w = world();
        // b's aligned pose is a.right + 12 = 412; raw lands 25 pt beyond it (37 from flush,
        // out of edge-snap reach): captured.
        let out = drag(&mut w, 1, (610.0, 110.0), (447.0, 113.0), false);
        assert_eq!(out, Some(DropOutcome::Joined));
        assert_eq!((w.widgets[1].x, w.widgets[1].y), (400.0, 100.0));
        assert_eq!(w.widgets[0].group, w.widgets[1].group);
    }

    #[test]
    fn flush_contact_alone_no_longer_joins() {
        let mut w = world();
        // c dropped flush under a (edge snap aligns it), but sockets are at the ends only.
        let out = drag(&mut w, 2, (110.0, 610.0), (112.0, 213.0), false);
        assert_eq!(out, Some(DropOutcome::Moved));
        assert_eq!((w.widgets[2].x, w.widgets[2].y), (100.0, 200.0));
        assert_ne!(w.widgets[2].group, w.widgets[0].group);
    }

    #[test]
    fn an_end_with_any_widget_flush_against_it_is_taken() {
        let mut w = world();
        // c sits flush at a's right end without being in a's group.
        w.widgets[2] = Placed::new("c", 400.0, 100.0, 200.0, 100.0, 3);
        w.widgets[1].x = 900.0;
        // b raw 8 pt from where a's right socket would line it up: but that end is taken.
        w.begin_drag(1, (910.0, 110.0), false);
        w.drag_to((430.0, 110.0));
        let d = w.drag.as_ref().unwrap();
        assert_ne!(d.pairing.map(|(j, _)| j), Some(0));
    }

    fn settle(w: &mut World) -> usize {
        let mut frames = 0;
        while w.tick(0.016) {
            frames += 1;
            assert!(frames < 200, "spring never settled");
        }
        frames
    }

    #[test]
    fn free_drag_is_one_to_one_with_no_spring() {
        let mut w = world();
        w.begin_drag(2, (110.0, 610.0), false);
        w.drag_to((160.0, 640.0));
        assert_eq!((w.widgets[2].x, w.widgets[2].y), (150.0, 630.0));
        assert!(!w.in_flight(), "no frames needed outside the field");
        assert_eq!(w.drag.as_ref().unwrap().feel, Feel::Free);
    }

    #[test]
    fn approach_band_pulls_toward_the_socket_through_the_spring() {
        let mut w = world();
        w.begin_drag(1, (610.0, 110.0), false);
        // Raw pose x = 472: 60 pt right of the aligned pose (412, 100).
        w.drag_to((482.0, 110.0));
        let d = w.drag.as_ref().unwrap();
        assert_eq!(d.feel, Feel::Approach);
        assert!(w.in_flight(), "the spring animates toward the goal");
        assert_eq!(w.widgets[1].x, 600.0, "the spring starts from where the widget was");
        settle(&mut w);
        let pull = 60.0 * magnet::strength(&magnet::proximity(60.0));
        assert!((w.widgets[1].x - (472.0 - pull)).abs() < 1e-9, "{}", w.widgets[1].x);
        assert_eq!(w.widgets[1].y, 100.0);
        assert!(!w.in_flight());
        // Releasing in the approach band does not join and leaves it where it is.
        let x = w.widgets[1].x;
        assert_eq!(w.end_drag(), Some(DropOutcome::Moved));
        assert_ne!(w.widgets[0].group, w.widgets[1].group);
        assert_eq!(w.widgets[1].x, x);
    }

    #[test]
    fn captured_sits_almost_on_the_pose() {
        let mut w = world();
        w.begin_drag(1, (610.0, 110.0), false);
        w.drag_to((447.0, 110.0)); // raw 437: 25 pt out
        assert_eq!(w.drag.as_ref().unwrap().feel, Feel::Captured);
        settle(&mut w);
        assert!((w.widgets[1].x - (437.0 - 25.0 * 0.88)).abs() < 1e-9);
    }

    #[test]
    fn leaving_the_field_returns_to_one_to_one_and_there_is_no_hysteresis() {
        let mut w = world();
        w.begin_drag(1, (610.0, 110.0), false);
        w.drag_to((482.0, 110.0));
        settle(&mut w);
        let first = w.widgets[1].x;
        w.drag_to((447.0, 110.0));
        settle(&mut w);
        w.drag_to((700.0, 110.0));
        assert_eq!(w.widgets[1].x, 690.0, "free again: on the pointer at once");
        assert!(!w.in_flight());
        w.drag_to((482.0, 110.0));
        settle(&mut w);
        assert_eq!(w.widgets[1].x, first, "the second approach feels like the first");
    }

    #[test]
    fn widgets_without_sockets_feel_no_field_and_never_join() {
        let mut w = world();
        w.widgets[2].sockets = false;
        // c (200 wide) dragged level with a, right next to a's right end.
        let out = drag(&mut w, 2, (110.0, 610.0), (416.0, 110.0), false);
        assert_eq!(out, Some(DropOutcome::Moved));
        assert_ne!(w.widgets[2].group, w.widgets[0].group);
        // Nor does anything join to it.
        let out = drag(&mut w, 1, (610.0, 110.0), (626.0 + 200.0 - 600.0 + 400.0, 110.0), false);
        assert_ne!(out, Some(DropOutcome::Joined));
    }

    #[test]
    fn edge_snap_does_not_fight_the_field() {
        let mut w = world();
        w.screens = vec![Rect::new(0.0, 0.0, 2000.0, 1200.0)];
        w.begin_drag(1, (610.0, 110.0), false);
        // Raw pose y = 92 is within snapping range of a's top edge (100), but inside the
        // field the goal comes from the field alone.
        w.drag_to((482.0, 102.0));
        settle(&mut w);
        let (g, _) = magnet::goal((472.0, 92.0), Some((412.0, 100.0)));
        assert!((w.widgets[1].x - g.0).abs() < 1e-9 && (w.widgets[1].y - g.1).abs() < 1e-9);
    }

    #[test]
    fn a_joined_pair_offers_only_its_outer_sockets() {
        let mut w = world();
        drag(&mut w, 1, (610.0, 110.0), (428.0, 113.0), false);
        // c, level with the pair, raw 8 pt beyond b's right end's aligned pose (712).
        w.widgets[2] = Placed::new("c", 900.0, 100.0, 200.0, 100.0, 3);
        let out = drag(&mut w, 2, (910.0, 110.0), (730.0, 110.0), false);
        assert_eq!(out, Some(DropOutcome::Joined));
        assert_eq!(w.widgets[2].x, 700.0);
        assert_eq!(w.group_size(0), 3);
        // Dragging the trio finds no inner sockets to pull toward.
        w.begin_drag(0, (150.0, 110.0), false);
        w.drag_to((160.0, 110.0));
        assert_eq!(w.drag.as_ref().unwrap().feel, Feel::Free);
    }

    #[test]
    fn escape_mid_spring_restores_the_start() {
        let mut w = world();
        w.begin_drag(1, (610.0, 110.0), false);
        w.drag_to((482.0, 110.0));
        w.tick(0.016);
        w.cancel_drag();
        assert_eq!((w.widgets[1].x, w.widgets[1].y), (600.0, 100.0));
        assert!(!w.in_flight());
    }

    #[test]
    fn joined_group_moves_together_and_option_drag_separates() {
        let mut w = world();
        drag(&mut w, 1, (610.0, 110.0), (428.0, 113.0), false);
        let out = drag(&mut w, 0, (150.0, 110.0), (250.0, 310.0), false);
        assert_eq!(out, Some(DropOutcome::Moved));
        assert_eq!((w.widgets[0].x, w.widgets[0].y), (200.0, 300.0));
        assert_eq!((w.widgets[1].x, w.widgets[1].y), (500.0, 300.0));
        // Solo drag pulls b out of the group.
        drag(&mut w, 1, (510.0, 310.0), (810.0, 510.0), true);
        assert_ne!(w.widgets[0].group, w.widgets[1].group);
        assert_eq!(w.widgets[0].x, 200.0);
    }

    #[test]
    fn separating_keeps_on_top_setting() {
        let mut w = world();
        drag(&mut w, 1, (610.0, 110.0), (428.0, 113.0), false);
        w.toggle_on_top(0);
        assert!(w.on_top(1), "on-top is per group");
        w.detach(1);
        assert!(w.on_top(1) && w.on_top(0));
        w.toggle_on_top(1);
        assert!(!w.on_top(1) && w.on_top(0));
    }

    #[test]
    fn shade_slides_stacked_group_members() {
        let mut w = world();
        // c hangs directly below a in a's group (a saved layout; sockets join side by side).
        w.widgets[2].y = 200.0;
        w.widgets[2].group = 1;
        w.toggle_shade(0);
        assert_eq!(w.widgets[0].rect().h, SHADE_H);
        assert_eq!(w.widgets[2].y, 100.0 + SHADE_H, "c slides up to stay flush");
        w.toggle_shade(0);
        assert_eq!(w.widgets[2].y, 200.0);
    }

    #[test]
    fn shade_keeps_side_by_side_tops() {
        let mut w = world();
        drag(&mut w, 1, (610.0, 110.0), (428.0, 113.0), false);
        w.shade_all(true);
        assert!(w.widgets.iter().all(|p| p.shaded));
        assert_eq!(w.widgets[0].y, w.widgets[1].y);
        w.shade_all(false);
        assert_eq!(w.widgets[1].y, 100.0);
    }

    /// a | b side by side, c hanging under b, all one group.
    fn l_shape() -> World {
        let mut w = World::default();
        w.widgets.push(Placed::new("a", 100.0, 100.0, 300.0, 100.0, 1));
        w.widgets.push(Placed::new("b", 400.0, 100.0, 300.0, 100.0, 1));
        w.widgets.push(Placed::new("c", 400.0, 200.0, 300.0, 80.0, 1));
        w.groups.insert(1, false);
        w
    }

    #[test]
    fn shading_only_moves_widgets_hanging_below() {
        let mut w = l_shape();
        w.toggle_shade(0);
        assert_eq!(w.widgets[2].y, 200.0, "c hangs under b, not a");
        w.toggle_shade(1);
        assert_eq!(w.widgets[2].y, 100.0 + SHADE_H);
    }

    #[test]
    fn shade_all_round_trip_restores_positions() {
        let mut w = l_shape();
        w.shade_all(true);
        w.shade_all(false);
        let ys: Vec<f64> = w.widgets.iter().map(|p| p.y).collect();
        assert_eq!(ys, vec![100.0, 100.0, 200.0]);
    }

    #[test]
    fn shade_moves_a_chain_once() {
        let mut w = world();
        w.widgets[1] = Placed::new("b", 100.0, 200.0, 300.0, 100.0, 1);
        w.widgets[2] = Placed::new("c", 100.0, 300.0, 200.0, 80.0, 1);
        w.toggle_shade(0);
        assert_eq!(w.widgets[1].y, 100.0 + SHADE_H);
        assert_eq!(w.widgets[2].y, 200.0 + SHADE_H);
    }

    #[test]
    fn docked_widgets_never_ride_with_a_group() {
        let mut w = with_dock(world());
        w.widgets[2].dock = Some("dock-left".into());
        w.widgets[2].group = w.widgets[0].group; // e.g. a hand-edited layout
        w.begin_drag(0, (150.0, 110.0), false);
        assert_eq!(w.drag.as_ref().unwrap().members, vec![0]);
    }

    #[test]
    fn non_finite_positions_do_not_panic() {
        let mut w = world();
        w.widgets[0].y = f64::NAN;
        w.shade_all(true);
        w.begin_drag(1, (610.0, 110.0), false);
        w.drag_to((700.0, 300.0));
        w.end_drag();
    }

    fn with_dock(mut w: World) -> World {
        w.set_docks(vec![DockRegion { id: "dock-left".into(), rect: Rect::new(0.0, 1000.0, 700.0, 60.0), pack: Pack::FromRight, on_top: true }]);
        w
    }

    #[test]
    fn dropping_into_a_dock_docks_shades_and_inherits_on_top() {
        let mut w = with_dock(world());
        let out = drag(&mut w, 2, (110.0, 610.0), (300.0, 1020.0), false);
        assert_eq!(out, Some(DropOutcome::Docked("dock-left".into())));
        let c = &w.widgets[2];
        assert!(c.shaded);
        assert_eq!(c.dock.as_deref(), Some("dock-left"));
        assert_eq!(c.rect(), Rect::new(500.0, 1000.0 + 17.0, 200.0, SHADE_H));
        assert!(w.on_top(2), "inherits the dock's on-top");
    }

    #[test]
    fn dragging_out_of_a_dock_undocks_and_restores_shade() {
        let mut w = with_dock(world());
        drag(&mut w, 2, (110.0, 610.0), (300.0, 1020.0), false);
        drag(&mut w, 2, (600.0, 1020.0), (600.0, 500.0), false);
        let c = &w.widgets[2];
        assert!(c.dock.is_none());
        assert!(!c.shaded);
        assert!(!w.on_top(2));
    }

    #[test]
    fn docking_a_group_docks_each_member_in_order() {
        let mut w = with_dock(world());
        drag(&mut w, 1, (610.0, 110.0), (428.0, 113.0), false);
        drag(&mut w, 0, (150.0, 110.0), (200.0, 1010.0), false);
        assert_eq!(w.dock_members("dock-left").len(), 2);
        // a (left) fills first, against the Dock edge; b stacks below it.
        assert_eq!(w.widgets[0].rect().right(), 700.0);
        assert_eq!(w.widgets[1].rect().y, w.widgets[0].rect().bottom() + 2.0);
    }

    #[test]
    fn removing_a_dock_frees_its_widgets() {
        let mut w = with_dock(world());
        drag(&mut w, 2, (110.0, 610.0), (300.0, 1020.0), false);
        w.set_docks(vec![]);
        assert!(w.widgets[2].dock.is_none());
        assert!(!w.widgets[2].shaded);
    }
}
