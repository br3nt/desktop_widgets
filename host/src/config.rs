//! Config files: widget definitions (`widgets.toml`) and the saved arrangement (`layout.toml`).

use crate::docks::DockRegion;
use crate::model::{Placed, World};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const DEFAULT_WIDGETS: &str = include_str!("../widgets.default.toml");

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PanelKind {
    Native,
    Html,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Panel {
    pub kind: PanelKind,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct WidgetDef {
    pub id: String,
    pub title: String,
    /// Actor behaviour name (see `actor.rs`). Defaults to "generic".
    #[serde(default = "generic")]
    pub behaviour: String,
    #[serde(default = "default_w")]
    pub width: f64,
    #[serde(default = "default_h")]
    pub height: f64,
    pub panel: Panel,
    /// Initial actor state (strings, numbers, booleans).
    #[serde(default)]
    pub state: serde_json::Map<String, serde_json::Value>,
}

fn generic() -> String {
    "generic".into()
}
fn default_w() -> f64 {
    360.0
}
fn default_h() -> f64 {
    116.0
}
fn default_port() -> u16 {
    47800
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct WidgetsFile {
    /// Local message endpoint port on 127.0.0.1 (0 = off).
    #[serde(default = "default_port")]
    pub endpoint_port: u16,
    /// Dock length guess (points) when the OS can't tell us where the Dock is.
    #[serde(default)]
    pub dock_length: Option<f64>,
    #[serde(rename = "widget", default)]
    pub widgets: Vec<WidgetDef>,
}

pub fn parse_widgets(s: &str) -> Result<WidgetsFile, String> {
    toml::from_str(s).map_err(|e| e.to_string())
}

// ---- layout ----

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SavedWidget {
    pub id: String,
    pub x: f64,
    pub y: f64,
    #[serde(default)]
    pub shaded: bool,
    pub group: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dock: Option<String>,
    #[serde(default)]
    pub shaded_before_dock: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SavedGroup {
    pub id: u32,
    pub on_top: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SavedDock {
    pub id: String,
    pub on_top: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Layout {
    #[serde(rename = "widget", default)]
    pub widgets: Vec<SavedWidget>,
    #[serde(rename = "group", default)]
    pub groups: Vec<SavedGroup>,
    #[serde(rename = "dock", default)]
    pub docks: Vec<SavedDock>,
}

impl Layout {
    pub fn from_world(w: &World) -> Layout {
        Layout {
            widgets: w
                .widgets
                .iter()
                .map(|p| SavedWidget {
                    id: p.id.clone(),
                    x: p.x.round(),
                    y: p.y.round(),
                    shaded: p.shaded,
                    group: p.group,
                    dock: p.dock.clone(),
                    shaded_before_dock: p.shaded_before_dock,
                })
                .collect(),
            groups: w.groups.iter().map(|(&id, &on_top)| SavedGroup { id, on_top }).collect(),
            docks: w.docks.iter().map(|d| SavedDock { id: d.id.clone(), on_top: d.on_top }).collect(),
        }
    }

    /// Build a world for `defs`, restoring saved state where present. New widgets get
    /// a default cascade position and their own group.
    pub fn to_world(&self, defs: &[WidgetDef], docks: Vec<DockRegion>, screens: Vec<crate::geom::Rect>) -> World {
        let mut w = World { screens, ..Default::default() };
        let mut groups: BTreeMap<u32, bool> = self.groups.iter().map(|g| (g.id, g.on_top)).collect();
        let mut next = groups.keys().chain(self.widgets.iter().map(|s| &s.group)).max().copied().unwrap_or(0) + 1;
        let origin = w.screens.first().map(|s| (s.x, s.y)).unwrap_or((0.0, 0.0));
        for (k, d) in defs.iter().enumerate() {
            let mut p = Placed::new(&d.id, origin.0 + 40.0, origin.1 + 40.0 + k as f64 * (d.height + 24.0), d.width, d.height, 0);
            if let Some(s) = self.widgets.iter().find(|s| s.id == d.id) {
                p.x = s.x;
                p.y = s.y;
                p.shaded = s.shaded;
                p.group = s.group;
                p.dock = s.dock.clone();
                p.shaded_before_dock = s.shaded_before_dock;
            } else {
                p.group = next;
                next += 1;
            }
            groups.entry(p.group).or_insert(false);
            w.widgets.push(p);
        }
        groups.retain(|g, _| w.widgets.iter().any(|p| p.group == *g));
        w.groups = groups;
        let mut docks = docks;
        for d in &mut docks {
            if let Some(s) = self.docks.iter().find(|s| s.id == d.id) {
                d.on_top = s.on_top;
            }
        }
        w.set_docks(docks);
        w
    }
}

pub fn parse_layout(s: &str) -> Result<Layout, String> {
    toml::from_str(s).map_err(|e| e.to_string())
}

pub fn layout_to_string(l: &Layout) -> String {
    toml::to_string_pretty(l).unwrap_or_default()
}

/// `~/Library/Application Support/desktop_widgets` on macOS, the platform config dir elsewhere.
/// `DESKTOP_WIDGETS_DIR` overrides it.
pub fn config_dir() -> PathBuf {
    if let Ok(d) = std::env::var("DESKTOP_WIDGETS_DIR") {
        return PathBuf::from(d);
    }
    dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("desktop_widgets")
}

pub fn load_widgets(dir: &Path) -> WidgetsFile {
    let path = dir.join("widgets.toml");
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => {
            let _ = std::fs::create_dir_all(dir);
            let _ = std::fs::write(&path, DEFAULT_WIDGETS);
            DEFAULT_WIDGETS.to_string()
        }
    };
    match parse_widgets(&text) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("desktop-widgets: {} is invalid ({e}); using built-in widgets", path.display());
            parse_widgets(DEFAULT_WIDGETS).expect("built-in widgets parse")
        }
    }
}

pub fn load_layout(dir: &Path) -> Layout {
    std::fs::read_to_string(dir.join("layout.toml")).ok().and_then(|t| parse_layout(&t).ok()).unwrap_or_default()
}

pub fn save_layout(dir: &Path, w: &World) {
    let _ = std::fs::create_dir_all(dir);
    let tmp = dir.join("layout.toml.tmp");
    if std::fs::write(&tmp, layout_to_string(&Layout::from_world(w))).is_ok() {
        let _ = std::fs::rename(&tmp, dir.join("layout.toml"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::docks::Pack;
    use crate::geom::Rect;

    #[test]
    fn default_widgets_parse() {
        let f = parse_widgets(DEFAULT_WIDGETS).unwrap();
        let ids: Vec<&str> = f.widgets.iter().map(|w| w.id.as_str()).collect();
        assert_eq!(ids, ["jarvis", "gitdiscuss", "checks", "panel-demo"]);
        assert_eq!(f.widgets[3].panel.kind, PanelKind::Html);
        assert!(f.widgets[3].panel.url.as_deref().unwrap().contains("panel-demo.html"));
        assert_eq!(f.widgets[0].panel.kind, PanelKind::Native);
        assert!(f.widgets[0].state.contains_key("value"));
    }

    fn defs() -> Vec<WidgetDef> {
        parse_widgets(DEFAULT_WIDGETS).unwrap().widgets
    }

    fn dock() -> Vec<DockRegion> {
        vec![DockRegion { id: "dock-left".into(), rect: Rect::new(0.0, 1000.0, 700.0, 60.0), pack: Pack::FromRight, on_top: true }]
    }

    #[test]
    fn layout_round_trips_through_toml() {
        let mut w = Layout::default().to_world(&defs(), dock(), vec![Rect::new(0.0, 25.0, 1728.0, 1000.0)]);
        // Join jarvis+gitdiscuss, shade one, set on-top, dock checks, dock off-top.
        w.widgets[1].x = w.widgets[0].x + w.widgets[0].w;
        w.widgets[1].y = w.widgets[0].y;
        let g = w.widgets[0].group;
        w.widgets[1].group = g;
        w.toggle_on_top(0);
        w.toggle_shade(1);
        w.dock(2, "dock-left");
        w.relayout_docks();
        w.toggle_on_top(2);

        let text = layout_to_string(&Layout::from_world(&w));
        let back = parse_layout(&text).unwrap().to_world(&defs(), dock(), w.screens.clone());
        for (a, b) in w.widgets.iter().zip(&back.widgets) {
            assert_eq!((a.id.as_str(), a.x.round(), a.y.round(), a.shaded, a.group, &a.dock), (b.id.as_str(), b.x, b.y, b.shaded, b.group, &b.dock));
        }
        assert!(back.on_top(1));
        assert!(!back.docks[0].on_top);
        assert_eq!(back.widgets[2].dock_w, w.widgets[2].dock_w);
    }

    #[test]
    fn new_widgets_get_their_own_groups() {
        let w = Layout::default().to_world(&defs(), vec![], vec![]);
        let mut gs: Vec<u32> = w.widgets.iter().map(|p| p.group).collect();
        gs.dedup();
        assert_eq!(gs.len(), 4);
        assert_eq!(w.groups.len(), 4);
    }
}
