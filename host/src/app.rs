//! The windowing shell: one borderless transparent window per widget, plus faint overlay
//! windows for dock regions. Input becomes model operations (drag, shade, join, dock) or
//! actor messages; windows are then synced to the model and redrawn on demand only.

use crate::actor::{self, Message, Stage};
use crate::casing::{self, Button, Hit, Parts};
use crate::config::{self, Layout, PanelKind, WidgetDef, WidgetsFile};
use crate::docks::{self, DockRegion, ScreenInfo};
use crate::geom::Rect;
use crate::model::{DropOutcome, World};
use crate::platform::{self, Presenter};
use crate::render::{self, WidgetView};
use crate::text::Fonts;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};
use tiny_skia::Pixmap;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId, WindowLevel};

pub enum UserEvent {
    Menu(tray_icon::menu::MenuId),
    /// A JSON message from the local endpoint.
    Endpoint(serde_json::Value),
    /// A message posted by an HTML panel: (widget id, JSON body).
    Ipc(String, String),
}

struct WidgetWin {
    window: Rc<Window>,
    presenter: Option<Presenter>,
    webview: Option<wry::WebView>,
    pos: (f64, f64),
    size: (f64, f64),
    top: bool,
    /// What the last draw depended on besides actor state.
    look: (bool, bool, bool, bool),
    local: (f64, f64),
    last_click: Option<Instant>,
}

struct Overlay {
    id: String,
    window: Rc<Window>,
    presenter: Option<Presenter>,
    rect: Rect,
    shown: bool,
    hot: bool,
}

struct MenuIds {
    shade: tray_icon::menu::MenuId,
    unshade: tray_icon::menu::MenuId,
    refresh: tray_icon::menu::MenuId,
    reset: tray_icon::menu::MenuId,
    quit: tray_icon::menu::MenuId,
}

pub struct App {
    proxy: EventLoopProxy<UserEvent>,
    dir: PathBuf,
    file: WidgetsFile,
    defs: Vec<WidgetDef>,
    world: World,
    stage: Stage,
    fonts: Fonts,
    wins: Vec<WidgetWin>,
    overlays: Vec<Overlay>,
    tray: Option<tray_icon::TrayIcon>,
    menu: Option<MenuIds>,
    alt: bool,
    cmd: bool,
    dragging: Option<usize>,
    started: bool,
}

impl App {
    pub fn new(proxy: EventLoopProxy<UserEvent>, dir: PathBuf, file: WidgetsFile) -> App {
        let defs = file.widgets.clone();
        let mut stage = Stage::new();
        for d in &defs {
            stage.spawn(&d.id, actor::behaviour(&d.behaviour), d.state.clone());
        }
        App {
            proxy,
            dir,
            file,
            defs,
            world: World::default(),
            stage,
            fonts: Fonts::load(),
            wins: Vec::new(),
            overlays: Vec::new(),
            tray: None,
            menu: None,
            alt: false,
            cmd: false,
            dragging: None,
            started: false,
        }
    }

    // ---- geometry from the OS ----

    fn screens(&self, el: &ActiveEventLoop) -> Vec<ScreenInfo> {
        #[cfg(target_os = "macos")]
        {
            let _ = el;
            platform::screens()
        }
        #[cfg(not(target_os = "macos"))]
        {
            platform::screens_from(el.available_monitors())
        }
    }

    fn compute_docks(&self, screens: &[ScreenInfo]) -> Vec<DockRegion> {
        let dock = platform::dock_rect();
        let est = self.file.dock_length.unwrap_or_else(platform::dock_length_estimate);
        let screen = dock
            .and_then(|d| screens.iter().find(|s| s.frame.contains(d.center_x(), d.y + d.h / 2.0)))
            .or_else(|| screens.iter().find(|s| s.frame.bottom() - s.visible.bottom() > 1.0 || s.visible.x > s.frame.x || s.visible.right() < s.frame.right()))
            .or(screens.first());
        match screen {
            Some(s) => docks::default_regions(s, platform::dock_side_hint(), dock, est),
            None => Vec::new(),
        }
    }

    /// Re-read screens and the Dock; move docked widgets if the regions changed.
    fn refresh_docks(&mut self, el: &ActiveEventLoop) {
        let screens = self.screens(el);
        let regions = self.compute_docks(&screens);
        self.world.screens = screens.iter().map(|s| s.visible).collect();
        let same = regions.len() == self.world.docks.len() && regions.iter().zip(&self.world.docks).all(|(a, b)| a.id == b.id && a.rect == b.rect);
        if !same {
            self.world.set_docks(regions);
            self.build_overlays(el);
            self.sync();
        }
    }

    // ---- windows ----

    fn base_attrs(title: &str, r: Rect, top: bool) -> winit::window::WindowAttributes {
        #[allow(unused_mut)]
        let mut a = Window::default_attributes()
            .with_title(title)
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(false)
            // Shown after the Spaces/collection behaviour is set, so it applies on first order-in.
            .with_visible(false)
            .with_inner_size(LogicalSize::new(r.w, r.h))
            .with_position(LogicalPosition::new(r.x, r.y))
            .with_window_level(if top { WindowLevel::AlwaysOnTop } else { WindowLevel::Normal });
        #[cfg(target_os = "macos")]
        {
            use winit::platform::macos::WindowAttributesExtMacOS;
            a = a.with_has_shadow(false).with_accepts_first_mouse(true);
        }
        #[cfg(target_os = "windows")]
        {
            use winit::platform::windows::WindowAttributesExtWindows;
            a = a.with_skip_taskbar(true);
        }
        a
    }

    fn create_widget_windows(&mut self, el: &ActiveEventLoop) {
        for i in 0..self.defs.len() {
            let def = self.defs[i].clone();
            let r = self.world.widgets[i].rect();
            let top = self.world.on_top(i);
            let window = match el.create_window(Self::base_attrs(&def.title, r, top)) {
                Ok(w) => Rc::new(w),
                Err(e) => {
                    eprintln!("desktop-widgets: window for {} failed: {e}", def.id);
                    std::process::exit(1);
                }
            };
            platform::configure_widget_window(&window);
            let presenter = Presenter::new(&window);
            let mut webview = None;
            if def.panel.kind == PanelKind::Html {
                let url = def.panel.url.clone().unwrap_or_else(|| "about:blank".into());
                let parts = self.parts(i);
                let proxy = self.proxy.clone();
                let id = def.id.clone();
                let built = wry::WebViewBuilder::new()
                    .with_url(&url)
                    .with_transparent(true)
                    .with_bounds(panel_bounds(&parts))
                    .with_ipc_handler(move |req| {
                        let _ = proxy.send_event(UserEvent::Ipc(id.clone(), req.body().clone()));
                    })
                    .build_as_child(&*window);
                match built {
                    Ok(wv) => webview = Some(wv),
                    Err(e) => eprintln!("desktop-widgets: webview for {} failed: {e}", def.id),
                }
            }
            window.set_visible(true);
            window.request_redraw();
            self.wins.push(WidgetWin {
                window,
                presenter,
                webview,
                pos: (r.x, r.y),
                size: (r.w, r.h),
                top,
                look: (false, false, false, false),
                local: (0.0, 0.0),
                last_click: None,
            });
        }
    }

    fn build_overlays(&mut self, el: &ActiveEventLoop) {
        self.overlays.clear();
        for d in self.world.docks.clone() {
            let Ok(w) = el.create_window(Self::base_attrs("dock region", d.rect, true)) else { continue };
            let window = Rc::new(w);
            let _ = window.set_cursor_hittest(false);
            platform::configure_widget_window(&window);
            let presenter = Presenter::new(&window);
            let mut o = Overlay { id: d.id.clone(), window, presenter, rect: d.rect, shown: false, hot: false };
            draw_overlay(&mut self.fonts, &mut o);
            o.window.set_visible(true);
            self.overlays.push(o);
        }
    }

    fn draw_overlay_at(&mut self, k: usize) {
        draw_overlay(&mut self.fonts, &mut self.overlays[k]);
    }

    fn set_overlays(&mut self, shown: bool, hot: Option<usize>) {
        for k in 0..self.overlays.len() {
            let hot_k = hot.map(|h| self.world.docks.get(h).map(|d| d.id == self.overlays[k].id).unwrap_or(false)).unwrap_or(false);
            if self.overlays[k].shown != shown || self.overlays[k].hot != hot_k {
                self.overlays[k].shown = shown;
                self.overlays[k].hot = hot_k;
                self.draw_overlay_at(k);
            }
        }
    }

    fn parts(&self, i: usize) -> Parts {
        let r = self.world.widgets[i].rect();
        casing::parts(r.w, r.h, self.world.widgets[i].shaded, self.world.group_size(i) > 1)
    }

    /// Push model geometry to the real windows.
    fn sync(&mut self) {
        for i in 0..self.wins.len() {
            let r = self.world.widgets[i].rect();
            let top = self.world.on_top(i);
            let look = (self.world.widgets[i].shaded, self.world.group_size(i) > 1, top, self.world.widgets[i].dock.is_some());
            let parts = self.parts(i);
            let w = &mut self.wins[i];
            let resized = (r.w, r.h) != w.size;
            if resized {
                let _ = w.window.request_inner_size(LogicalSize::new(r.w, r.h));
                w.size = (r.w, r.h);
                if let Some(wv) = &w.webview {
                    let _ = wv.set_bounds(panel_bounds(&parts));
                }
            }
            if resized || (r.x, r.y) != w.pos {
                w.window.set_outer_position(LogicalPosition::new(r.x, r.y));
                w.pos = (r.x, r.y);
            }
            if top != w.top {
                w.window.set_window_level(if top { WindowLevel::AlwaysOnTop } else { WindowLevel::Normal });
                w.top = top;
            }
            if resized || look != w.look {
                w.look = look;
                w.window.request_redraw();
            }
        }
    }

    fn draw(&mut self, i: usize) {
        let r = self.world.widgets[i].rect();
        let parts = self.parts(i);
        let def = &self.defs[i];
        let scale = self.wins[i].window.scale_factor() as f32;
        let (pw, ph) = ((r.w * scale as f64).ceil() as u32, (r.h * scale as f64).ceil() as u32);
        let Some(mut pm) = Pixmap::new(pw.max(1), ph.max(1)) else { return };
        let view = WidgetView {
            title: &def.title,
            state: self.stage.state(&def.id),
            parts: &parts,
            on_top: self.world.on_top(i),
            docked: self.world.widgets[i].dock.is_some(),
            html: def.panel.kind == PanelKind::Html,
        };
        render::draw_widget(&mut pm, scale, &mut self.fonts, &view);
        if let Some(p) = self.wins[i].presenter.as_mut() {
            p.present(pm.take(), pw.max(1), ph.max(1), r.w, r.h);
        }
    }

    fn save(&self) {
        config::save_layout(&self.dir, &self.world);
    }

    // ---- actors ----

    fn deliver(&mut self, msg: Message) {
        if !self.stage.post(msg.clone()) {
            eprintln!("desktop-widgets: no widget named {:?}", msg.to);
        }
        for id in self.stage.run() {
            self.state_changed(&id);
        }
    }

    fn state_changed(&mut self, id: &str) {
        let Some(i) = self.defs.iter().position(|d| d.id == id) else { return };
        self.wins[i].window.request_redraw();
        self.push_state(i);
    }

    fn push_state(&self, i: usize) {
        if let (Some(wv), Some(st)) = (&self.wins[i].webview, self.stage.state(&self.defs[i].id)) {
            let json = serde_json::to_string(st).unwrap_or_else(|_| "{}".into());
            let _ = wv.evaluate_script(&format!("window.dwState && window.dwState({json})"));
        }
    }

    // ---- input ----

    fn cursor(&self, i: usize) -> (f64, f64) {
        platform::cursor().unwrap_or((self.wins[i].pos.0 + self.wins[i].local.0, self.wins[i].pos.1 + self.wins[i].local.1))
    }

    fn press(&mut self, el: &ActiveEventLoop, i: usize) {
        let (lx, ly) = self.wins[i].local;
        match self.parts(i).hit(lx, ly) {
            Hit::Button(Button::Shade) => {
                self.world.toggle_shade(i);
                self.after_change();
            }
            Hit::Button(Button::OnTop) => {
                self.world.toggle_on_top(i);
                self.after_change();
            }
            Hit::Button(Button::Detach) => {
                self.world.detach(i);
                self.after_change();
            }
            Hit::Panel if self.defs[i].panel.kind == PanelKind::Native => {
                let id = self.defs[i].id.clone();
                self.deliver(Message::new("host", &id, "press", serde_json::json!({})));
            }
            _ => {
                let now = Instant::now();
                if self.wins[i].last_click.is_some_and(|t| now - t < Duration::from_millis(400)) {
                    self.wins[i].last_click = None;
                    self.world.toggle_shade(i);
                    self.after_change();
                    return;
                }
                self.wins[i].last_click = Some(now);
                self.refresh_docks(el);
                let c = self.cursor(i);
                self.world.begin_drag(i, c, self.alt);
                self.dragging = Some(i);
            }
        }
    }

    fn release(&mut self) {
        if self.dragging.take().is_none() {
            return;
        }
        let out = self.world.end_drag();
        self.set_overlays(false, None);
        if matches!(out, Some(DropOutcome::Clicked)) {
            // A plain click may still have detached (Option-click).
            self.sync();
            return;
        }
        self.wins.iter_mut().for_each(|w| w.last_click = None);
        self.after_change();
    }

    fn after_change(&mut self) {
        self.sync();
        self.save();
    }

    fn reset_layout(&mut self, el: &ActiveEventLoop) {
        let _ = std::fs::remove_file(self.dir.join("layout.toml"));
        let screens = self.screens(el);
        let docks = self.compute_docks(&screens);
        self.world = Layout::default().to_world(&self.defs, docks, screens.iter().map(|s| s.visible).collect());
        self.build_overlays(el);
        self.after_change();
    }

    fn start(&mut self, el: &ActiveEventLoop) {
        let screens = self.screens(el);
        let docks = self.compute_docks(&screens);
        let layout = config::load_layout(&self.dir);
        self.world = layout.to_world(&self.defs, docks, screens.iter().map(|s| s.visible).collect());
        // Anything saved on a screen that's gone comes back to the main screen.
        for i in 0..self.world.widgets.len() {
            let r = self.world.widgets[i].rect();
            if !screens.iter().any(|s| s.frame.intersects(&r)) {
                if let Some(s) = screens.first() {
                    self.world.widgets[i].x = s.visible.x + 40.0;
                    self.world.widgets[i].y = s.visible.y + 40.0 + i as f64 * 30.0;
                }
            }
        }
        self.build_overlays(el);
        self.create_widget_windows(el);
        self.build_tray();
        self.sync();
        self.save();
    }

    fn build_tray(&mut self) {
        use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
        let menu = Menu::new();
        let shade = MenuItem::new("Shade all", true, None);
        let unshade = MenuItem::new("Unshade all", true, None);
        let refresh = MenuItem::new("Refresh dock regions", true, None);
        let reset = MenuItem::new("Reset layout", true, None);
        let quit = MenuItem::new("Quit Desktop Widgets", true, None);
        let _ = menu.append_items(&[&shade, &unshade, &PredefinedMenuItem::separator(), &refresh, &reset, &PredefinedMenuItem::separator(), &quit]);
        self.menu = Some(MenuIds { shade: shade.id().clone(), unshade: unshade.id().clone(), refresh: refresh.id().clone(), reset: reset.id().clone(), quit: quit.id().clone() });
        let proxy = self.proxy.clone();
        MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
            let _ = proxy.send_event(UserEvent::Menu(e.id));
        }));
        let size = 36;
        let icon = tray_icon::Icon::from_rgba(render::tray_icon_rgba(size), size, size).ok();
        let mut b = tray_icon::TrayIconBuilder::new().with_menu(Box::new(menu)).with_tooltip("Desktop Widgets");
        if let Some(icon) = icon {
            b = b.with_icon_templated(icon);
        }
        match b.build() {
            Ok(t) => self.tray = Some(t),
            Err(e) => eprintln!("desktop-widgets: no tray icon ({e}); quit with Cmd-Q on a widget or Ctrl-C"),
        }
    }

    fn quit(&mut self, el: &ActiveEventLoop) {
        self.save();
        el.exit();
    }
}

fn draw_overlay(fonts: &mut Fonts, o: &mut Overlay) {
        let scale = o.window.scale_factor() as f32;
        let (pw, ph) = ((o.rect.w * scale as f64).ceil() as u32, (o.rect.h * scale as f64).ceil() as u32);
        let Some(mut pm) = Pixmap::new(pw.max(1), ph.max(1)) else { return };
        if o.shown {
            let label = if o.id.ends_with("left") || o.id.ends_with("above") { "Dock region · left of the Dock" } else { "Dock region · right of the Dock" };
            render::draw_dock_overlay(&mut pm, scale, fonts, o.rect.w, o.rect.h, label, o.hot);
        }
        if let Some(p) = o.presenter.as_mut() {
            p.present(pm.take(), pw.max(1), ph.max(1), o.rect.w, o.rect.h);
        }
}

fn panel_bounds(p: &Parts) -> wry::Rect {
    let r = p.panel.inset(if p.shaded { 1.0 } else { 2.0 });
    wry::Rect { position: wry::dpi::LogicalPosition::new(r.x, r.y).into(), size: wry::dpi::LogicalSize::new(r.w, r.h).into() }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if !self.started {
            self.started = true;
            self.start(el);
        }
    }

    fn user_event(&mut self, el: &ActiveEventLoop, ev: UserEvent) {
        match ev {
            UserEvent::Menu(id) => {
                let Some(m) = &self.menu else { return };
                if id == m.shade {
                    self.world.shade_all(true);
                    self.after_change();
                } else if id == m.unshade {
                    self.world.shade_all(false);
                    self.after_change();
                } else if id == m.refresh {
                    self.world.docks.clear();
                    self.refresh_docks(el);
                    self.after_change();
                } else if id == m.reset {
                    self.reset_layout(el);
                } else if id == m.quit {
                    self.quit(el);
                }
            }
            UserEvent::Endpoint(v) => match Message::from_json(&v, "endpoint", None) {
                Ok(m) => self.deliver(m),
                Err(e) => eprintln!("desktop-widgets: bad endpoint message: {e}"),
            },
            UserEvent::Ipc(id, body) => {
                let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) else { return };
                if v.get("msg").and_then(|m| m.as_str()) == Some("ready") {
                    if let Some(i) = self.defs.iter().position(|d| d.id == id) {
                        self.push_state(i);
                    }
                    return;
                }
                match Message::from_json(&v, &id, Some(&id)) {
                    Ok(m) => self.deliver(m),
                    Err(e) => eprintln!("desktop-widgets: bad panel message from {id}: {e}"),
                }
            }
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, wid: WindowId, event: WindowEvent) {
        let Some(i) = self.wins.iter().position(|w| w.window.id() == wid) else {
            if let WindowEvent::RedrawRequested = event {
                if let Some(k) = self.overlays.iter().position(|o| o.window.id() == wid) {
                    self.draw_overlay_at(k);
                }
            }
            return;
        };
        match event {
            WindowEvent::RedrawRequested => self.draw(i),
            WindowEvent::ScaleFactorChanged { .. } => self.wins[i].window.request_redraw(),
            WindowEvent::ModifiersChanged(m) => {
                self.alt = m.state().alt_key();
                self.cmd = m.state().super_key();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let l = position.to_logical::<f64>(self.wins[i].window.scale_factor());
                self.wins[i].local = (l.x, l.y);
                if self.dragging == Some(i) {
                    let c = self.cursor(i);
                    self.world.drag_to(c);
                    if let Some(d) = &self.world.drag {
                        if d.moved {
                            let hot = d.hover_dock;
                            self.set_overlays(true, hot);
                        }
                    }
                    self.sync();
                }
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => self.press(el, i),
            WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. } => self.release(),
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => match event.logical_key {
                Key::Named(NamedKey::Escape) if self.dragging.is_some() => {
                    self.dragging = None;
                    self.world.cancel_drag();
                    self.set_overlays(false, None);
                    self.sync();
                }
                Key::Character(ref c) if self.cmd && c.as_str() == "q" => self.quit(el),
                _ => {}
            },
            WindowEvent::CloseRequested => self.quit(el),
            _ => {}
        }
    }
}
