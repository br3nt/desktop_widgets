# Desktop Widgets

Working name. A skinnable desktop shell of **capsules**: small widgets with hardware-style casings that you build, arrange, and dock together. Think Sonique-era media players: widgets that snap end-to-end, carry drawers and bays, and change shape and size to suit what they're showing.

## Ideas note

The living idea list is in Brent's Obsidian vault: [App Ideas / Desktop Widgets - Connectables](obsidian://open?vault=Notes&file=App%20Ideas%2FDesktop%20Widgets%20-%20Connectables) (`Notes/App Ideas/Desktop Widgets - Connectables.md`). It grows over time; read it before planning.

So far it covers: widgets for Spotify (bring back the desktop media player), social feeds from friends and family, RSS, chat (IRC and other protocols), agents and agent flows, bookmarks, and networked drives (iCloud, Dropbox, Drive, NAS, S3). Not everything needs a Dock or taskbar presence. Widgets talk to each other and share state through something like Zapt, with interop so parts can use other languages.

## Status

A first native host runs on macOS (see [Running the app](#running-the-app)). Before that came a web prototype:

- [`prototypes/capsule-study.html`](prototypes/capsule-study.html) — capsule assembly study: draggable capsules (Jarvis, GitDiscuss), socket joining with end-cap retraction, a release grip, an independent Checks widget that docks into a drawer bay, and paged digital screens. Open it directly in a browser; no build step.

## Running the app

The first native host lives in [`host/`](host/): one Rust process (winit windows, tiny-skia drawing, ab_glyph text, wry for opt-in HTML panels, a tray menu). It's built and tested on macOS. The Windows and Linux paths are behind `cfg` but not yet compiled or tested.

```sh
cd host
cargo run --release            # run from the terminal
cargo test                     # snapping, socket field, joining, shade, docks, config, actors, endpoint, /state
scripts/bundle-mac.sh          # builds ~/Applications/Desktop Widgets.app (menu-bar only, ad-hoc signed)
```

After bundling, launch **Desktop Widgets** from Spotlight. It has no Dock icon and doesn't appear in Cmd-Tab. Use the capsule icon in the menu bar to Shade all, Unshade all, Refresh dock regions, request exact Dock position, Reset layout, and Quit. Cmd-Q quits when a widget has focus.

What works (tested on macOS 15):

- **Drag** a widget by its casing (anything outside the display and buttons). Away from any socket, edges snap within 12 pt to other widgets and to the screen's usable edges.
- **Join:** capsules have a socket at each end. Within 120 pt of lining up end to end, a dragged widget is pulled toward the socket (a critically damped spring); within 40 pt it's captured, and releasing joins the two into a group that moves together. Move away and the pull fades the same way it grew. Widgets with `sockets = false` in `widgets.toml` (Checks) never join. **Separate:** drag with Option held, or press the detach button (the third lens, shown only when grouped).
- **Shade:** double-click the casing or press the top lens button to collapse to a strip showing the name and key value. Group members below slide up or down to stay flush.
- **Always on top** is per group: the middle lens button (lit lime when on). New widgets start on top.
- **Docks:** two regions either side of the Dock (or above and below it when the Dock is on a side). They show faintly while you drag. Dropping a widget there shades it and packs it next to the Dock. Docked widgets use the region's on-top setting. Drag a docked widget out to undock it. Dropping a group into a dock docks each member. Exact Dock bounds need Accessibility permission (menu: *Exact Dock position…*). Without it the Dock's length is estimated from its preferences and the running apps. Regions are recomputed at each drag start and from the menu.
- **Widgets are actors.** Each has its own state, changed only by messages it receives. Clicking the GitDiscuss display hands a discussion to Jarvis. Clicking Checks approves a check and tells Jarvis. The HTML panel's *Nudge Jarvis* button posts a message too. Outside programs can message any widget:

  ```sh
  curl -X POST http://127.0.0.1:47800/send -H 'Content-Type: application/json' -d '{"to":"jarvis","msg":"set","agents":4}'
  ```

- **Inspecting it:** `GET /state` returns every widget's pose, group, shade, dock, on-top, parts (nameplate, display, buttons) in screen points, its actor state, and the drag in progress. [`host/scripts/probe.sh`](host/scripts/probe.sh) wraps it with real mouse drags (cliclick) and region screenshots:

  ```sh
  host/scripts/probe.sh summary                                 # one line per widget
  host/scripts/probe.sh drag gitdiscuss 300 0 --shot /tmp/d.png # drag by its nameplate, report mid-drag and after
  ```

- **HTML panels:** set `panel = { kind = "html", url = "…" }` on a widget to get a transparent OS webview over its display slot. The demo panel ([`host/examples/panel-demo.html`](host/examples/panel-demo.html), served by the host at `http://127.0.0.1:47800/examples/panel-demo.html`) uses container queries to switch to a one-line layout in shade mode. The page receives its widget's state through `window.dwState(state)` and sends messages with `window.ipc.postMessage(JSON)`. A panel can message its own widget, plus any widgets listed in its `talks_to`, and it stays on its configured origin.
- **Endpoint safety:** the endpoint listens on 127.0.0.1 only. It refuses requests from web pages (foreign `Origin`, a non-JSON content type, or a `Host` other than 127.0.0.1/localhost) and sends no CORS headers. Any local process can still post to it, because there's no token yet.
- **Persistence:** positions, groups, shade, dock membership and on-top settings save on every change and restore on launch. Actor state (counters and text) isn't saved yet.

Config files (macOS: `~/Library/Application Support/desktop_widgets/`; elsewhere the platform config dir; override with `DESKTOP_WIDGETS_DIR`):

- `widgets.toml`: widget definitions (written from [`host/widgets.default.toml`](host/widgets.default.toml) on first run), plus the endpoint port.
- `layout.toml`: the saved arrangement.

Measured on an M-series MacBook (macOS 15.6, release build, idle). RSS includes shared system framework pages; the physical footprint is the process's own memory:

| Widgets | Host RSS | Host physical footprint | Extra processes |
|---|---|---|---|
| 3 native (Jarvis, GitDiscuss, Checks) | 69 MB | 23 MB | none |
| 3 native + 1 HTML panel | 87–100 MB | 34–36 MB | WebKit WebContent ~29 MB, Networking ~21 MB RSS |

Idle CPU is 0: no CPU time accrued over 10 s idle. The host only redraws on events.

Known gaps: the first click on an HTML panel while another app is active only focuses the panel. Joined widgets keep their own casings (no shared housing yet). Narrow docks squeeze strips, so long names get cut off. Dock regions don't yet track Dock changes live (they refresh at each drag start).

## Prior art

[`docs/prior-art.md`](docs/prior-art.md): Konfabulator, Dashboard, Windows Gadgets, Rainmeter, Winamp/Webamp, Sonique, Plasma, Übersicht, WidgetKit, Zebar and more. What worked, what killed them, and where this project is new.

## Direction

- **Lightest possible footprint.** People may run tens to hundreds of widgets, so no Electron and no browser engine per widget. Casey Muratori's view applies: modern software is orders of magnitude slower and bigger than the hardware requires, and this project should prove it doesn't have to be. Footprint is a feature with budgets that are measured and enforced in CI. Proposed starting budgets:
  - Zero CPU when idle. Redraw only when something changes.
  - Host process in the low tens of MB.
  - Well under 1 MB per skin-drawn widget.
  - Instant startup.
- **Every major OS from day one.** The framework runs on macOS, Windows and Linux. Individual widgets may be OS-specific.
- **Snapping and shade mode first.** Widgets snap together like Winamp's main window, equaliser and playlist, and collapse to a thin strip like Winamp's shade mode. Test with many different skins and shapes.
- **Angled connections.** A socket accepts a range of angles, and the user can set and change the angle of a joint.
- **Form factors** (not built yet): widgets expand, collapse and morph between sizes and shapes, as Sonique did.
- **Skins at two levels.** A generic widget design that any skin can style, plus widget authors packaging their own skins. A skin is one shareable file or folder.
- **User-defined placement regions.** Users define their own dynamic regions (the Mac example: either side of the Dock). Regions track the host OS's layout: Dock, taskbar and menu bar position, auto-hide, and display and resolution changes. Widgets inside a region adjust to it.
- **Always-on-top per group.** Each widget group has its own always-on-top setting. Docking regions have one too, and a widget or group docked into a region inherits the region's setting.

- **Actors all the way down.** Each widget is an actor (Zapt) with its own memory and state. That is its sandbox and also how widgets connect. Apps can start actors when they run, so which widgets exist depends on what's running.
- **One system across machines.** Widgets and actors on the laptop, the NUC and the VPSs all appear as one system. Parts can be disconnected at any time; a disconnected part is offline, not broken.
- **Functional, not decorative.** Widgets do things. Any single widget can die, but the framework stays useful. Data sources are separate actors, so a dead web service means swapping one source, not rewriting the widget.
- **Shape-aware surfaces.** A display's content knows its real shape (circle, polygon, ring), not only its width and height.

### Display types

A **display** is a shaped slot in a casing. Every display gets the same shape information (safe insets, the largest rectangle that fits, the exact outline) and the same input events, whatever fills it. One widget can mix surface types. Roughly cheapest first:

1. **Drawn by the skin**: sprites, LCD digits, knob strips, vector shapes (the Winamp and Sonique way). The default.
2. **Declarative UI**: the widget describes what to show and the host draws it natively. The best fit for widgets on other machines: send data, not pixels.
3. **Text or terminal grid**: a character-cell display, so any command-line tool can become a widget.
4. **2D drawing API**: an immediate-mode canvas for graphs, meters and custom gauges.
5. **Shader**: a GPU fragment shader, for visualisers.
6. **Video and camera**: the OS's native video layer.
7. **HTML/CSS through the OS webview** (WKWebView, WebView2, WebKitGTK): opt-in per surface because each costs roughly 20–50 MB, with that cost shown to the user.
8. **Embedded native OS views**: maps, system pickers. OS-specific.
9. **Legacy plugins**: Sonique `.svp` and Winamp visualisers, for fun.
10. **3D scene**: later.

### Frontends: desktop, VR and AR

Widget logic lives in actors that have no display of their own. A **frontend** draws them. The desktop is the first frontend; VR and AR headsets and glasses are more.

- **Flat panels first, 3D objects optional.** Native VR interfaces are mostly flat panels floating in space, so widgets stay flat panels in every mode. A widget may also offer a 3D casing.
- **Joints stay 2D.** Sockets connect along the panel's plane, with the angle range described above. There are no 3D joints, because they're confusing and would create distortions that are hard to recover from when a group moves back to a flat desktop.
- **Regions become anchors.** In VR and AR a region can be a desk surface, a wall, your wrist, a fixed spot in your field of view, or next to a real object.
- **Shade mode matters most on glasses**, where the display only has room for a strip.
- **Targets:** OpenXR (the cross-vendor standard: SteamVR, Meta Quest and Meta's new VR Glasses, Pico, Android XR); a separate native visionOS frontend, since Apple doesn't support OpenXR; small glasses displays in shade mode only.

## Open questions

- Renderer and toolkit: native rendering in a single host process (e.g. Rust with a 2D renderer) versus a trimmed webview. See the footprint goal.
- Window model: one OS window per docked group, or per widget?
- Isolation: how to keep one widget from reading another's data or the user's files, when many widgets share one process.
- Widget contract: what a widget declares (screens, controls, sockets and their angle ranges, sizes, required permissions) and where its data comes from.
- HTML/CSS surfaces within the footprint budget: a lightweight HTML/CSS engine (Sciter, litehtml, Blitz) or the OS webview only for surfaces that need it. How to hand the surface its shape (CSS custom properties, `env()`-style safe areas, an SVG path).
- Messaging between widgets: Zapt as the shared language, plus a language-neutral protocol for widgets written in other languages.
