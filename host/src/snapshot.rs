//! The JSON behind `GET /state`: every widget's pose, group, shade, dock and parts in screen
//! points, plus the drag in progress, so behaviour can be checked with numbers rather than
//! screenshots alone. No windowing code here; the shell calls `state` on request.
//!
//! `light` (socket glow brightness 0..1) and `tween` (join/latch phase) are reported now so
//! the shape stays stable; they stay 0 and null until the glow and the tweens exist.

use crate::casing::{self, Button};
use crate::geom::Rect;
use crate::model::World;
use serde_json::{json, Value};

fn rect(r: Rect) -> Value {
    json!({ "x": r.x, "y": r.y, "w": r.w, "h": r.h })
}

fn button_name(b: Button) -> &'static str {
    match b {
        Button::Shade => "shade",
        Button::OnTop => "on_top",
        Button::Detach => "detach",
    }
}

/// Snapshot of the arrangement. `actor` returns a widget's actor state, if any.
pub fn state(world: &World, actor: &dyn Fn(&str) -> Option<Value>) -> Value {
    let widgets: Vec<Value> = (0..world.widgets.len())
        .map(|i| {
            let p = &world.widgets[i];
            let r = p.rect();
            let group_size = world.group_size(i);
            let parts = casing::parts(r.w, r.h, p.shaded, group_size > 1);
            let at = |q: Rect| q.offset(r.x, r.y);
            let grip = at(parts.grip);
            let buttons: serde_json::Map<String, Value> = parts.buttons.iter().map(|(b, q)| (button_name(*b).to_string(), rect(at(*q)))).collect();
            json!({
                "id": p.id,
                "x": r.x,
                "y": r.y,
                "w": r.w,
                "h": r.h,
                "shaded": p.shaded,
                "group": p.group,
                "group_size": group_size,
                "dock": p.dock,
                "on_top": world.on_top(i),
                // A point on the nameplate: press here to drag the widget (and its group).
                "grab": { "x": grip.x + grip.w / 2.0, "y": grip.y + grip.h / 2.0 },
                "grip": rect(grip),
                "panel": rect(at(parts.panel)),
                "buttons": buttons,
                "light": 0.0,
                "tween": Value::Null,
                "actor": actor(&p.id).unwrap_or(Value::Null),
            })
        })
        .collect();
    let groups: Vec<Value> = world
        .groups
        .iter()
        .map(|(id, top)| json!({ "id": id, "on_top": top, "members": world.members(*id).iter().map(|&m| world.widgets[m].id.clone()).collect::<Vec<_>>() }))
        .collect();
    let docks: Vec<Value> = world
        .docks
        .iter()
        .map(|d| json!({ "id": d.id, "rect": rect(d.rect), "on_top": d.on_top, "members": world.dock_members(&d.id).iter().map(|&m| world.widgets[m].id.clone()).collect::<Vec<_>>() }))
        .collect();
    let drag = world.drag.as_ref().map(|d| {
        json!({
            "pressed": world.widgets[d.pressed].id,
            "members": d.members.iter().map(|&m| world.widgets[m].id.clone()).collect::<Vec<_>>(),
            "moved": d.moved,
            "cursor": { "x": d.cursor.0, "y": d.cursor.1 },
            "hover_dock": d.hover_dock.and_then(|k| world.docks.get(k)).map(|r| r.id.clone()),
        })
    });
    json!({
        "widgets": widgets,
        "groups": groups,
        "docks": docks,
        "screens": world.screens.iter().map(|s| rect(*s)).collect::<Vec<_>>(),
        "drag": drag,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Placed;

    fn world() -> World {
        let mut w = World::default();
        w.widgets.push(Placed::new("a", 100.0, 100.0, 300.0, 100.0, 1));
        w.widgets.push(Placed::new("b", 400.0, 100.0, 300.0, 100.0, 1));
        w.widgets.push(Placed::new("c", 100.0, 600.0, 200.0, 80.0, 2));
        w.groups.insert(1, true);
        w.groups.insert(2, false);
        w
    }

    #[test]
    fn reports_pose_group_shade_and_on_top() {
        let mut w = world();
        w.toggle_shade(2);
        let s = state(&w, &|_| None);
        let a = &s["widgets"][0];
        assert_eq!((a["id"].as_str(), a["x"].as_f64(), a["y"].as_f64(), a["w"].as_f64(), a["h"].as_f64()), (Some("a"), Some(100.0), Some(100.0), Some(300.0), Some(100.0)));
        assert_eq!(a["group"], 1);
        assert_eq!(a["group_size"], 2);
        assert_eq!(a["on_top"], true);
        assert_eq!(a["dock"], Value::Null);
        let c = &s["widgets"][2];
        assert_eq!(c["shaded"], true);
        assert_eq!(c["h"].as_f64(), Some(crate::model::SHADE_H));
        assert_eq!(s["groups"][0]["members"], json!(["a", "b"]));
        assert_eq!(s["drag"], Value::Null);
    }

    #[test]
    fn parts_are_in_screen_points_and_grab_is_on_the_casing() {
        let w = world();
        let s = state(&w, &|_| None);
        let b = &s["widgets"][1];
        let parts = casing::parts(300.0, 100.0, false, true);
        assert_eq!(b["grip"]["x"].as_f64(), Some(400.0 + parts.grip.x));
        assert_eq!(b["panel"]["y"].as_f64(), Some(100.0 + parts.panel.y));
        assert!(b["buttons"]["detach"].is_object(), "grouped widgets report the detach lens");
        assert!(s["widgets"][2]["buttons"]["detach"].is_null());
        // Pressing at the grab point starts a drag, not a button or the display.
        let (gx, gy) = (b["grab"]["x"].as_f64().unwrap(), b["grab"]["y"].as_f64().unwrap());
        assert_eq!(parts.hit(gx - 400.0, gy - 100.0), casing::Hit::Casing);
    }

    #[test]
    fn reports_the_drag_in_progress() {
        let mut w = world();
        w.begin_drag(2, (150.0, 610.0), false);
        let s = state(&w, &|_| None);
        assert_eq!(s["drag"]["moved"], false);
        w.drag_to((250.0, 410.0));
        let s = state(&w, &|_| None);
        let d = &s["drag"];
        assert_eq!(d["pressed"], "c");
        assert_eq!(d["members"], json!(["c"]));
        assert_eq!(d["moved"], true);
        assert_eq!((d["cursor"]["x"].as_f64(), d["cursor"]["y"].as_f64()), (Some(250.0), Some(410.0)));
        assert_eq!(s["widgets"][2]["x"].as_f64(), Some(200.0));
    }

    #[test]
    fn includes_actor_state_and_placeholders_for_later_feedback() {
        let w = world();
        let s = state(&w, &|id| (id == "a").then(|| json!({ "agents": 3 })));
        assert_eq!(s["widgets"][0]["actor"]["agents"], 3);
        assert_eq!(s["widgets"][1]["actor"], Value::Null);
        assert_eq!(s["widgets"][0]["light"].as_f64(), Some(0.0));
        assert_eq!(s["widgets"][0]["tween"], Value::Null);
    }
}
