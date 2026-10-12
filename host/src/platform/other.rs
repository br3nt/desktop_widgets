//! Windows and Linux: compile-clean basics. Screens come from winit monitors (no work-area
//! data yet, so dock regions use the configurable default strip), presenting via softbuffer.

use crate::docks::{DockSide, ScreenInfo};
use crate::geom::Rect;
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::window::Window;

pub fn screens_from(monitors: impl Iterator<Item = winit::monitor::MonitorHandle>) -> Vec<ScreenInfo> {
    monitors
        .map(|m| {
            let s = m.scale_factor();
            let p = m.position().to_logical::<f64>(s);
            let z = m.size().to_logical::<f64>(s);
            let r = Rect::new(p.x, p.y, z.width, z.height);
            ScreenInfo { frame: r, visible: r }
        })
        .collect()
}

pub fn cursor() -> Option<(f64, f64)> {
    None
}

pub fn dock_side_hint() -> DockSide {
    DockSide::Bottom
}

pub fn dock_length_estimate() -> f64 {
    700.0
}

pub fn dock_rect() -> Option<Rect> {
    None
}

pub fn configure_widget_window(_window: &Window) {}

pub struct Presenter {
    surface: softbuffer::Surface<Rc<Window>, Rc<Window>>,
}

impl Presenter {
    pub fn new(window: &Rc<Window>) -> Option<Presenter> {
        let ctx = softbuffer::Context::new(window.clone()).ok()?;
        let surface = softbuffer::Surface::new(&ctx, window.clone()).ok()?;
        Some(Presenter { surface })
    }

    pub fn present(&mut self, rgba: Vec<u8>, w: u32, h: u32, _lw: f64, _lh: f64) {
        let (Some(nw), Some(nh)) = (NonZeroU32::new(w), NonZeroU32::new(h)) else { return };
        if self.surface.resize(nw, nh).is_err() {
            return;
        }
        let Ok(mut buf) = self.surface.buffer_mut() else { return };
        for (i, px) in rgba.chunks_exact(4).enumerate() {
            if i < buf.len() {
                buf[i] = (px[0] as u32) << 16 | (px[1] as u32) << 8 | px[2] as u32;
            }
        }
        let _ = buf.present();
    }
}
