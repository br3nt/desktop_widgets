//! Widgets are actors. Each has its own state, changed only by the messages it receives
//! through its mailbox; handlers may send messages to other widgets. The host never edits
//! widget state directly: it turns input (clicks, the local endpoint, HTML panels) into
//! messages and renders panels from the resulting state.
//!
//! The handlers here are small Rust functions standing in for Zapt scripts. A Zapt
//! runtime slots in behind `Behaviour`: same messages in, state changes and sends out.

use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, VecDeque};

pub type State = Map<String, Value>;

#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub name: String,
    pub data: Map<String, Value>,
}

impl Message {
    pub fn new(from: &str, to: &str, name: &str, data: Value) -> Self {
        let data = match data {
            Value::Object(m) => m,
            _ => Map::new(),
        };
        Message { from: from.into(), to: to.into(), name: name.into(), data }
    }

    /// Parse the wire form `{"to":"jarvis","msg":"set","agents":4}`; other keys are the payload.
    /// `default_to` fills a missing `to` (an HTML panel talking to its own widget).
    pub fn from_json(v: &Value, from: &str, default_to: Option<&str>) -> Result<Message, String> {
        let obj = v.as_object().ok_or("message must be a JSON object")?;
        let to = obj.get("to").and_then(Value::as_str).or(default_to).ok_or("missing \"to\"")?;
        let name = obj.get("msg").and_then(Value::as_str).ok_or("missing \"msg\"")?;
        let mut data = obj.clone();
        data.remove("to");
        data.remove("msg");
        data.remove("from");
        Ok(Message { from: from.into(), to: to.into(), name: name.into(), data })
    }
}

/// A widget's message handlers.
pub trait Behaviour {
    fn receive(&self, me: &str, state: &mut State, msg: &Message, send: &mut Vec<Message>);
}

pub fn behaviour(name: &str) -> Box<dyn Behaviour> {
    match name {
        "jarvis" => Box::new(Jarvis),
        "gitdiscuss" => Box::new(GitDiscuss),
        "checks" => Box::new(Checks),
        _ => Box::new(Generic),
    }
}

fn num(s: &State, k: &str) -> i64 {
    s.get(k).and_then(Value::as_i64).unwrap_or(0)
}
fn put(s: &mut State, k: &str, v: impl Into<Value>) {
    s.insert(k.into(), v.into());
}

/// Every behaviour understands `set`: merge the payload into state.
fn set(state: &mut State, msg: &Message) -> bool {
    if msg.name == "set" {
        for (k, v) in &msg.data {
            state.insert(k.clone(), v.clone());
        }
        true
    } else {
        false
    }
}

pub struct Generic;
impl Behaviour for Generic {
    fn receive(&self, _me: &str, state: &mut State, msg: &Message, _send: &mut Vec<Message>) {
        set(state, msg);
    }
}

/// Jarvis: agent control. Counts agents, hears from GitDiscuss and Checks.
pub struct Jarvis;
impl Behaviour for Jarvis {
    fn receive(&self, _me: &str, s: &mut State, msg: &Message, _send: &mut Vec<Message>) {
        match msg.name.as_str() {
            "set" => {
                set(s, msg);
            }
            "press" => {
                let running = !s.get("running").and_then(Value::as_bool).unwrap_or(true);
                put(s, "running", running);
            }
            "task" => {
                let title = msg.data.get("title").and_then(Value::as_str).unwrap_or("new task");
                put(s, "agents", num(s, "agents") + 1);
                put(s, "caption", "Jarvis / picked up from GitDiscuss");
                put(s, "detail", format!("Working on: {title}"));
            }
            "checks" => {
                let (p, t) = (msg.data.get("passed").and_then(Value::as_i64).unwrap_or(0), msg.data.get("total").and_then(Value::as_i64).unwrap_or(0));
                put(s, "caption", "Jarvis / verify shell assembly");
                put(s, "detail", format!("{p} checks passed · {} queued", (t - p).max(0)));
            }
            _ => {}
        }
        // Derived display fields always follow the counters.
        let running = s.get("running").and_then(Value::as_bool).unwrap_or(true);
        put(s, "value", format!("{:02}", num(s, "agents")));
        put(s, "unit", if running { "agents running" } else { "agents paused" });
    }
}

/// GitDiscuss: a review queue. Pressing the display hands the next discussion to Jarvis.
pub struct GitDiscuss;
const THREADS: [&str; 3] = ["Shared casing geometry", "Capsule joining · design feedback", "Dock region sizing"];
impl Behaviour for GitDiscuss {
    fn receive(&self, me: &str, s: &mut State, msg: &Message, send: &mut Vec<Message>) {
        match msg.name.as_str() {
            "set" => {
                set(s, msg);
            }
            "press" => {
                let pending = num(s, "pending");
                if pending > 0 {
                    let title = THREADS[(pending as usize) % THREADS.len()];
                    put(s, "pending", pending - 1);
                    put(s, "detail", format!("Sent to Jarvis: {title}"));
                    send.push(Message::new(me, "jarvis", "task", json!({ "title": title })));
                } else {
                    put(s, "detail", "Queue clear");
                }
            }
            _ => {}
        }
        put(s, "value", format!("{:02}", num(s, "pending")));
    }
}

/// Checks: pressing approves one waiting check and tells Jarvis.
pub struct Checks;
impl Behaviour for Checks {
    fn receive(&self, me: &str, s: &mut State, msg: &Message, send: &mut Vec<Message>) {
        match msg.name.as_str() {
            "set" => {
                set(s, msg);
            }
            "press" => {
                let (p, t) = (num(s, "passed"), num(s, "total"));
                if p < t {
                    put(s, "passed", p + 1);
                }
                let p = num(s, "passed");
                send.push(Message::new(me, "jarvis", "checks", json!({ "passed": p, "total": t })));
            }
            _ => {}
        }
        let (p, t) = (num(s, "passed"), num(s, "total"));
        put(s, "value", format!("{p} / {t}"));
        put(s, "detail", if p >= t { "all approved".to_string() } else { format!("{} waiting for approval", t - p) });
    }
}

struct Actor {
    state: State,
    behaviour: Box<dyn Behaviour>,
    mailbox: VecDeque<Message>,
}

/// All actors plus delivery. Single-threaded: the host drains mailboxes after each input.
pub struct Stage {
    actors: BTreeMap<String, Actor>,
}

/// Message hops per drain: a ping-pong loop between widgets stops here instead of spinning.
pub const HOP_BUDGET: usize = 256;

impl Stage {
    pub fn new() -> Self {
        Stage { actors: BTreeMap::new() }
    }

    pub fn spawn(&mut self, id: &str, behaviour: Box<dyn Behaviour>, state: State) {
        self.actors.insert(id.into(), Actor { state, behaviour, mailbox: VecDeque::new() });
    }

    pub fn state(&self, id: &str) -> Option<&State> {
        self.actors.get(id).map(|a| &a.state)
    }

    /// Queue a message. `to = "all"` broadcasts. Unknown targets are dropped (returns false).
    pub fn post(&mut self, msg: Message) -> bool {
        if msg.to == "all" {
            for a in self.actors.values_mut() {
                a.mailbox.push_back(msg.clone());
            }
            return true;
        }
        match self.actors.get_mut(&msg.to) {
            Some(a) => {
                a.mailbox.push_back(msg);
                true
            }
            None => false,
        }
    }

    /// Deliver queued messages until quiet or the hop budget runs out. Returns ids whose state changed.
    pub fn run(&mut self) -> Vec<String> {
        let mut changed: Vec<String> = Vec::new();
        let mut hops = 0;
        loop {
            let next = self.actors.iter().find(|(_, a)| !a.mailbox.is_empty()).map(|(k, _)| k.clone());
            let Some(id) = next else { break };
            if hops >= HOP_BUDGET {
                for a in self.actors.values_mut() {
                    a.mailbox.clear();
                }
                eprintln!("desktop-widgets: message hop budget hit; dropped the rest");
                break;
            }
            hops += 1;
            let mut out = Vec::new();
            {
                let a = self.actors.get_mut(&id).unwrap();
                let msg = a.mailbox.pop_front().unwrap();
                let before = a.state.clone();
                a.behaviour.receive(&id, &mut a.state, &msg, &mut out);
                if a.state != before && !changed.contains(&id) {
                    changed.push(id.clone());
                }
            }
            for m in out {
                self.post(m);
            }
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stage() -> Stage {
        let mut s = Stage::new();
        let st = |v: Value| v.as_object().unwrap().clone();
        s.spawn("jarvis", behaviour("jarvis"), st(json!({"agents": 3, "running": true})));
        s.spawn("gitdiscuss", behaviour("gitdiscuss"), st(json!({"pending": 2})));
        s.spawn("checks", behaviour("checks"), st(json!({"passed": 12, "total": 14})));
        s.spawn("note", behaviour("generic"), State::new());
        s
    }

    fn get<'a>(s: &'a Stage, id: &str, k: &str) -> &'a Value {
        &s.state(id).unwrap()[k]
    }

    #[test]
    fn wire_format_parses() {
        let m = Message::from_json(&json!({"to":"jarvis","msg":"set","agents":4}), "endpoint", None).unwrap();
        assert_eq!((m.to.as_str(), m.name.as_str()), ("jarvis", "set"));
        assert_eq!(m.data["agents"], 4);
        assert!(!m.data.contains_key("to"));
        let m = Message::from_json(&json!({"msg":"set"}), "panel-demo", Some("panel-demo")).unwrap();
        assert_eq!(m.to, "panel-demo");
        assert!(Message::from_json(&json!({"to":"x"}), "e", None).is_err());
    }

    #[test]
    fn set_from_outside_updates_jarvis_display() {
        let mut s = stage();
        s.post(Message::from_json(&json!({"to":"jarvis","msg":"set","agents":4}), "endpoint", None).unwrap());
        assert_eq!(s.run(), vec!["jarvis".to_string()]);
        assert_eq!(get(&s, "jarvis", "value"), "04");
    }

    #[test]
    fn checks_press_messages_jarvis() {
        let mut s = stage();
        s.post(Message::new("host", "checks", "press", json!({})));
        let changed = s.run();
        assert!(changed.contains(&"checks".to_string()) && changed.contains(&"jarvis".to_string()));
        assert_eq!(get(&s, "checks", "value"), "13 / 14");
        assert_eq!(get(&s, "checks", "detail"), "1 waiting for approval");
        assert_eq!(get(&s, "jarvis", "detail"), "13 checks passed · 1 queued");
    }

    #[test]
    fn gitdiscuss_hands_tasks_to_jarvis() {
        let mut s = stage();
        s.post(Message::new("host", "gitdiscuss", "press", json!({})));
        s.run();
        assert_eq!(get(&s, "gitdiscuss", "value"), "01");
        assert_eq!(get(&s, "jarvis", "value"), "04");
        assert!(get(&s, "jarvis", "detail").as_str().unwrap().starts_with("Working on:"));
    }

    #[test]
    fn jarvis_press_pauses() {
        let mut s = stage();
        s.post(Message::new("host", "jarvis", "press", json!({})));
        s.run();
        assert_eq!(get(&s, "jarvis", "unit"), "agents paused");
    }

    #[test]
    fn unknown_target_is_dropped_and_broadcast_reaches_all() {
        let mut s = stage();
        assert!(!s.post(Message::new("x", "nobody", "set", json!({}))));
        s.post(Message::new("x", "all", "set", json!({"hello": 1})));
        s.run();
        for id in ["jarvis", "gitdiscuss", "checks", "note"] {
            assert_eq!(get(&s, id, "hello"), 1);
        }
    }

    struct Echo;
    impl Behaviour for Echo {
        fn receive(&self, me: &str, s: &mut State, msg: &Message, send: &mut Vec<Message>) {
            put(s, "n", num(s, "n") + 1);
            send.push(Message::new(me, &msg.from, "ping", json!({})));
        }
    }

    #[test]
    fn ping_pong_loop_stops_at_hop_budget() {
        let mut s = Stage::new();
        s.spawn("a", Box::new(Echo), State::new());
        s.spawn("b", Box::new(Echo), State::new());
        s.post(Message::new("b", "a", "ping", json!({})));
        s.run();
        let total = num(s.state("a").unwrap(), "n") + num(s.state("b").unwrap(), "n");
        assert_eq!(total as usize, HOP_BUDGET);
    }
}
