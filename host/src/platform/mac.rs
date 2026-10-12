//! macOS specifics: screen and Dock geometry, global cursor, window collection behaviour,
//! and a CALayer presenter that keeps per-pixel alpha (so casings look non-rectangular).

use crate::docks::{DockSide, ScreenInfo};
use crate::geom::Rect;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{AllocAnyThread, MainThreadMarker};
use objc2_app_kit::{NSEvent, NSRunningApplication, NSScreen, NSView, NSWindowCollectionBehavior};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_core_graphics::{
    CGBitmapInfo, CGColorRenderingIntent, CGColorSpace, CGDataProvider, CGImage, CGImageAlphaInfo, CGImageByteOrderInfo,
};
use objc2_foundation::{ns_string, NSString, NSUserDefaults};
use objc2_quartz_core::{CALayer, CATransaction};
use std::ffi::c_void;
use std::ptr::NonNull;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::Window;

fn mtm() -> MainThreadMarker {
    MainThreadMarker::new().expect("main thread")
}

fn primary_height() -> f64 {
    NSScreen::screens(mtm()).firstObject().map(|s| s.frame().size.height).unwrap_or(0.0)
}

fn flip(r: CGRect, ph: f64) -> Rect {
    Rect::new(r.origin.x, ph - (r.origin.y + r.size.height), r.size.width, r.size.height)
}

/// All screens in top-left logical coordinates (winit's convention).
pub fn screens() -> Vec<ScreenInfo> {
    let ph = primary_height();
    NSScreen::screens(mtm()).iter().map(|s| ScreenInfo { frame: flip(s.frame(), ph), visible: flip(s.visibleFrame(), ph) }).collect()
}

pub fn cursor() -> Option<(f64, f64)> {
    let p = NSEvent::mouseLocation();
    Some((p.x, primary_height() - p.y))
}

/// Option key held right now (works even when no widget window is focused).
pub fn alt_held() -> Option<bool> {
    Some(NSEvent::modifierFlags_class().contains(objc2_app_kit::NSEventModifierFlags::Option))
}

fn dock_defaults() -> Option<Retained<NSUserDefaults>> {
    NSUserDefaults::initWithSuiteName(NSUserDefaults::alloc(), Some(ns_string!("com.apple.dock")))
}

pub fn dock_side_hint() -> DockSide {
    let o = dock_defaults().and_then(|d| d.stringForKey(ns_string!("orientation"))).map(|s| s.to_string());
    match o.as_deref() {
        Some("left") => DockSide::Left,
        Some("right") => DockSide::Right,
        _ => DockSide::Bottom,
    }
}

/// Rough Dock length when Accessibility isn't granted: tiles from the Dock's prefs.
pub fn dock_length_estimate() -> f64 {
    let Some(d) = dock_defaults() else { return 700.0 };
    let count = |k: &NSString| d.arrayForKey(k).map(|a| a.count()).unwrap_or(0);
    let apps = count(ns_string!("persistent-apps"));
    let others = count(ns_string!("persistent-others"));
    let recents = count(ns_string!("recent-apps"));
    let tile = d.doubleForKey(ns_string!("tilesize"));
    let tile = if tile > 8.0 { tile } else { 48.0 };
    // Finder + Trash + separators; running apps not kept in the Dock add more, so lean long.
    let items = apps + others + recents + 2 + 4;
    (items as f64 * (tile + 6.0) + 24.0).max(200.0)
}

// --- Accessibility: the Dock's exact list frame, only if the user granted access. ---

type CFTypeRef = *const c_void;
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXUIElementCreateApplication(pid: i32) -> CFTypeRef;
    fn AXUIElementCopyAttributeValue(element: CFTypeRef, attribute: CFTypeRef, value: *mut CFTypeRef) -> i32;
    fn AXValueGetValue(value: CFTypeRef, the_type: u32, value_ptr: *mut c_void) -> bool;
}
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: CFTypeRef);
    fn CFArrayGetCount(a: CFTypeRef) -> isize;
    fn CFArrayGetValueAtIndex(a: CFTypeRef, i: isize) -> CFTypeRef;
}

unsafe fn ax_attr(el: CFTypeRef, name: &NSString) -> Option<CFTypeRef> {
    let mut v: CFTypeRef = std::ptr::null();
    let key = name as *const NSString as CFTypeRef;
    (AXUIElementCopyAttributeValue(el, key, &mut v) == 0 && !v.is_null()).then_some(v)
}

/// The Dock's icon strip, in top-left coordinates. None without Accessibility permission.
pub fn dock_rect() -> Option<Rect> {
    unsafe {
        if !AXIsProcessTrusted() {
            return None;
        }
        let apps = NSRunningApplication::runningApplicationsWithBundleIdentifier(ns_string!("com.apple.dock"));
        let pid = apps.firstObject()?.processIdentifier();
        let app = AXUIElementCreateApplication(pid);
        if app.is_null() {
            return None;
        }
        let mut out = None;
        if let Some(kids) = ax_attr(app, ns_string!("AXChildren")) {
            for i in 0..CFArrayGetCount(kids) {
                let el = CFArrayGetValueAtIndex(kids, i);
                let (Some(pos), Some(size)) = (ax_attr(el, ns_string!("AXPosition")), ax_attr(el, ns_string!("AXSize"))) else { continue };
                let mut p = CGPoint { x: 0.0, y: 0.0 };
                let mut s = CGSize { width: 0.0, height: 0.0 };
                let ok = AXValueGetValue(pos, 1, &mut p as *mut _ as *mut c_void) && AXValueGetValue(size, 2, &mut s as *mut _ as *mut c_void);
                CFRelease(pos);
                CFRelease(size);
                if ok && s.width > 0.0 && s.height > 0.0 {
                    // AX coordinates are already top-left based.
                    out = Some(Rect::new(p.x, p.y, s.width, s.height));
                    break;
                }
            }
            CFRelease(kids);
        }
        CFRelease(app);
        out
    }
}

fn ns_view(window: &Window) -> Option<&NSView> {
    match window.window_handle().ok()?.as_raw() {
        RawWindowHandle::AppKit(h) => Some(unsafe { h.ns_view.cast::<NSView>().as_ref() }),
        _ => None,
    }
}

/// Visible on every Space, out of Cmd-Tab / Exposé cycling, no window shadow.
pub fn configure_widget_window(window: &Window) {
    if let Some(view) = ns_view(window) {
        if let Some(w) = view.window() {
            w.setCollectionBehavior(
                NSWindowCollectionBehavior::CanJoinAllSpaces
                    | NSWindowCollectionBehavior::Stationary
                    | NSWindowCollectionBehavior::IgnoresCycle
                    | NSWindowCollectionBehavior::FullScreenAuxiliary,
            );
            w.setHasShadow(false);
        }
    }
}

/// Presents premultiplied RGBA pixels through a sublayer of the window's view.
pub struct Presenter {
    layer: Retained<CALayer>,
    color_space: objc2_core_foundation::CFRetained<CGColorSpace>,
}

unsafe extern "C-unwind" fn release(_info: *mut c_void, data: NonNull<c_void>, size: usize) {
    let slice = std::ptr::slice_from_raw_parts_mut(data.as_ptr().cast::<u8>(), size);
    drop(Box::from_raw(slice));
}

impl Presenter {
    pub fn new(window: &Window) -> Option<Presenter> {
        let view = ns_view(window)?;
        view.setWantsLayer(true);
        let root = view.layer()?;
        let layer = CALayer::new();
        layer.setFrame(root.bounds());
        root.addSublayer(&layer);
        Some(Presenter { layer, color_space: CGColorSpace::new_device_rgb()? })
    }

    /// `rgba` is tiny-skia's premultiplied RGBA; `w`,`h` in pixels; `lw`,`lh` the logical size.
    pub fn present(&mut self, rgba: Vec<u8>, w: u32, h: u32, lw: f64, lh: f64) {
        let data: Box<[u8]> = rgba.into_boxed_slice();
        let len = data.len();
        let ptr = Box::into_raw(data).cast::<c_void>();
        unsafe {
            let Some(provider) = CGDataProvider::with_data(std::ptr::null_mut(), ptr, len, Some(release)) else { return };
            let info = CGBitmapInfo(CGImageAlphaInfo::PremultipliedLast.0 | CGImageByteOrderInfo::OrderDefault.0);
            let Some(image) = CGImage::new(
                w as usize,
                h as usize,
                8,
                32,
                w as usize * 4,
                Some(&self.color_space),
                info,
                Some(&provider),
                std::ptr::null(),
                false,
                CGColorRenderingIntent::RenderingIntentDefault,
            ) else {
                return;
            };
            CATransaction::begin();
            CATransaction::setDisableActions(true);
            self.layer.setFrame(CGRect { origin: CGPoint { x: 0.0, y: 0.0 }, size: CGSize { width: lw, height: lh } });
            self.layer.setContents(Some(&*(image.as_ref() as *const CGImage as *const AnyObject)));
            CATransaction::commit();
        }
    }
}
