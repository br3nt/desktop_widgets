//! The arrangement model: where widgets are, which group they belong to, shade state,
//! dock membership and per-group always-on-top. No windowing code here; the shell
//! reads `World` and moves real windows to match.

use crate::docks::{self, DockRegion};
use crate::geom::Rect;
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
}

impl Placed {
    pub fn new(id: &str, x: f64, y: f64, w: f64, h: f64, group: u32) -> Self {
        Placed { id: id.into(), x, y, w, h, shaded: false, group, dock: None, shaded_before_dock: false, dock_w: None }
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
    pressed: usize,
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
    pub fn index(&self, id: &str) -> Option<usize> {
        self.widgets.iter().position(|w| w.id == id)
    }

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
        let g = self.widgets[i].group;
        // Everything in the group whose top is at or below the old bottom edge moves with it.
        for j in 0..self.widgets.len() {
            if j != i && self.widgets[j].group == g && self.widgets[j].y >= old.bottom() - 0.5 {
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
        order.sort_by(|a, b| self.widgets[*b].y.partial_cmp(&self.widgets[*a].y).unwrap());
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
    pub fn relayout_docks(&mut self) {
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
        self.relayout_docks();
    }

    // ---- dragging ----

    /// Press on widget `i`'s casing. `solo` detaches it from its group first (Option-drag).
    pub fn begin_drag(&mut self, i: usize, cursor: (f64, f64), solo: bool) {
        if solo {
            self.detach(i);
        }
        let members = if self.widgets[i].dock.is_some() { vec![i] } else { self.members(self.widgets[i].group) };
        let origins = members.iter().map(|&m| (self.widgets[m].x, self.widgets[m].y)).collect();
        self.drag = Some(Drag { members, origins, start: cursor, moved: false, hover_dock: None, pressed: i });
    }

    pub fn drag_to(&mut self, cursor: (f64, f64)) {
        let Some(mut d) = self.drag.take() else { return };
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
                self.relayout_docks();
                let after = self.widgets[p].rect();
                let fx = ((d.start.0 - before.x) / before.w).clamp(0.0, 1.0);
                d.origins[0] = (d.start.0 - fx * after.w, d.start.1 - (d.start.1 - before.y).min(after.h - 4.0));
            }
        }
        for (k, &m) in d.members.iter().enumerate() {
            self.widgets[m].x = d.origins[k].0 + dx;
            self.widgets[m].y = d.origins[k].1 + dy;
        }
        // Snap against free widgets outside the moving set, and screen edges.
        let moving: Vec<Rect> = d.members.iter().map(|&m| self.widgets[m].rect()).collect();
        let targets: Vec<Rect> = (0..self.widgets.len())
            .filter(|j| !d.members.contains(j) && self.widgets[*j].dock.is_none())
            .map(|j| self.widgets[j].rect())
            .collect();
        let (sx, sy) = snap::snap_offset(&moving, &targets, &self.screens, snap::SNAP_THRESHOLD);
        for &m in &d.members {
            self.widgets[m].x += sx;
            self.widgets[m].y += sy;
        }
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
            ms.sort_by(|a, b| self.widgets[*a].x.partial_cmp(&self.widgets[*b].x).unwrap());
            for m in ms {
                self.dock(m, &id);
            }
            self.relayout_docks();
            return Some(DropOutcome::Docked(id));
        }
        let moving: Vec<Rect> = d.members.iter().map(|&m| self.widgets[m].rect()).collect();
        let others: Vec<usize> = (0..self.widgets.len())
            .filter(|j| !d.members.contains(j) && self.widgets[*j].dock.is_none())
            .collect();
        let targets: Vec<Rect> = others.iter().map(|&j| self.widgets[j].rect()).collect();
        let hits = snap::contacts(&moving, &targets);
        if hits.is_empty() {
            return Some(DropOutcome::Moved);
        }
        let keep = self.widgets[d.members[0]].group;
        for (_, t) in hits {
            let gone = self.widgets[others[t]].group;
            self.merge_groups(keep, gone);
        }
        Some(DropOutcome::Joined)
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
    fn dragging_moves_and_snaps_into_a_join() {
        let mut w = world();
        // Move b so its left edge lands 6pt right of a's right edge (400): snaps to 400 and joins.
        let out = drag(&mut w, 1, (610.0, 110.0), (416.0, 113.0), false);
        assert_eq!(out, Some(DropOutcome::Joined));
        assert_eq!((w.widgets[1].x, w.widgets[1].y), (400.0, 100.0));
        assert_eq!(w.widgets[0].group, w.widgets[1].group);
    }

    #[test]
    fn joined_group_moves_together_and_option_drag_separates() {
        let mut w = world();
        drag(&mut w, 1, (610.0, 110.0), (416.0, 113.0), false);
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
        drag(&mut w, 1, (610.0, 110.0), (416.0, 113.0), false);
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
        // Put c directly below a and join.
        let out = drag(&mut w, 2, (110.0, 610.0), (112.0, 213.0), false);
        assert_eq!(out, Some(DropOutcome::Joined));
        assert_eq!((w.widgets[2].x, w.widgets[2].y), (100.0, 200.0));
        w.toggle_shade(0);
        assert_eq!(w.widgets[0].rect().h, SHADE_H);
        assert_eq!(w.widgets[2].y, 100.0 + SHADE_H, "c slides up to stay flush");
        w.toggle_shade(0);
        assert_eq!(w.widgets[2].y, 200.0);
    }

    #[test]
    fn shade_keeps_side_by_side_tops() {
        let mut w = world();
        drag(&mut w, 1, (610.0, 110.0), (416.0, 113.0), false);
        w.shade_all(true);
        assert!(w.widgets.iter().all(|p| p.shaded));
        assert_eq!(w.widgets[0].y, w.widgets[1].y);
        w.shade_all(false);
        assert_eq!(w.widgets[1].y, 100.0);
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
        assert_eq!(c.rect(), Rect::new(500.0, 1000.0 + 2.0 + 16.0, 200.0, SHADE_H));
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
        drag(&mut w, 1, (610.0, 110.0), (416.0, 113.0), false);
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
