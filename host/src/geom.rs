//! Plain geometry in logical points, top-left origin (the primary screen's top-left is 0,0).

#[derive(Clone, Copy, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub const fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Rect { x, y, w, h }
    }
    pub fn right(&self) -> f64 {
        self.x + self.w
    }
    pub fn bottom(&self) -> f64 {
        self.y + self.h
    }
    pub fn center_x(&self) -> f64 {
        self.x + self.w / 2.0
    }
    pub fn contains(&self, px: f64, py: f64) -> bool {
        px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
    }
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn offset(&self, dx: f64, dy: f64) -> Rect {
        Rect::new(self.x + dx, self.y + dy, self.w, self.h)
    }
    pub fn inset(&self, d: f64) -> Rect {
        Rect::new(self.x + d, self.y + d, (self.w - 2.0 * d).max(0.0), (self.h - 2.0 * d).max(0.0))
    }
    /// Length of the overlap of two closed ranges (negative = gap).
    pub fn overlap(a0: f64, a1: f64, b0: f64, b1: f64) -> f64 {
        a1.min(b1) - a0.max(b0)
    }
    pub fn h_overlap(&self, o: &Rect) -> f64 {
        Rect::overlap(self.x, self.right(), o.x, o.right())
    }
    pub fn v_overlap(&self, o: &Rect) -> f64 {
        Rect::overlap(self.y, self.bottom(), o.y, o.bottom())
    }
    pub fn intersects(&self, o: &Rect) -> bool {
        self.h_overlap(o) > 0.0 && self.v_overlap(o) > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edges_and_overlap() {
        let a = Rect::new(0.0, 0.0, 100.0, 50.0);
        let b = Rect::new(90.0, 40.0, 100.0, 50.0);
        assert_eq!(a.right(), 100.0);
        assert_eq!(a.bottom(), 50.0);
        assert_eq!(a.h_overlap(&b), 10.0);
        assert_eq!(a.v_overlap(&b), 10.0);
        assert!(a.intersects(&b));
        assert!(!a.intersects(&b.offset(20.0, 0.0)));
        assert!(a.contains(0.0, 0.0) && !a.contains(100.0, 10.0));
    }
}
