//! desktop-widgets: one small host process drawing widget casings natively.

mod actor;
mod app;
mod casing;
mod config;
mod docks;
mod geom;
mod model;
mod platform;
mod render;
mod server;
mod snap;
mod snapshot;
mod text;

use std::sync::Mutex;
use winit::event_loop::EventLoop;

fn main() {
    let dir = config::config_dir();
    let file = config::load_widgets(&dir);

    #[allow(unused_mut)]
    let mut builder = EventLoop::<app::UserEvent>::with_user_event();
    #[cfg(target_os = "macos")]
    {
        use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
        // Not in the Dock or Cmd-Tab.
        builder.with_activation_policy(ActivationPolicy::Accessory);
    }
    let event_loop = builder.build().expect("event loop");
    let proxy = event_loop.create_proxy();

    if file.endpoint_port != 0 {
        let p = Mutex::new(proxy.clone());
        let q = Mutex::new(proxy.clone());
        let r = server::start(
            file.endpoint_port,
            move |v| {
                if let Ok(p) = p.lock() {
                    let _ = p.send_event(app::UserEvent::Endpoint(v));
                }
            },
            // Ask the event loop for a snapshot; it answers between events.
            move || {
                let (tx, rx) = std::sync::mpsc::channel();
                q.lock().ok()?.send_event(app::UserEvent::State(tx)).ok()?;
                rx.recv_timeout(std::time::Duration::from_secs(1)).ok()
            },
        );
        match r {
            Ok(()) => eprintln!("desktop-widgets: messages on http://127.0.0.1:{}/send", file.endpoint_port),
            Err(e) => eprintln!("desktop-widgets: endpoint on port {} unavailable ({e})", file.endpoint_port),
        }
    }
    eprintln!("desktop-widgets: config in {}", dir.display());

    let mut app = app::App::new(proxy, dir, file);
    if let Err(e) = event_loop.run_app(&mut app) {
        eprintln!("desktop-widgets: {e}");
    }
}
