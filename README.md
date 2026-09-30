# Desktop Widgets

Working name. A skinnable desktop shell of **capsules**: small widgets with hardware-style casings that you build, arrange, and dock together. Think Sonique-era media players: widgets that snap end-to-end, carry drawers and bays, and change shape and size to suit what they're showing.

## Ideas note

The living idea list is in Brent's Obsidian vault: [App Ideas / Desktop Widgets - Connectables](obsidian://open?vault=Notes&file=App%20Ideas%2FDesktop%20Widgets%20-%20Connectables) (`Notes/App Ideas/Desktop Widgets - Connectables.md`). It grows over time; read it before planning.

So far it covers: widgets for Spotify (bring back the desktop media player), social feeds from friends and family, RSS, chat (IRC and other protocols), agents and agent flows, bookmarks, and networked drives (iCloud, Dropbox, Drive, NAS, S3). Not everything needs a Dock or taskbar presence. Widgets talk to each other and share state through something like Zapt, with interop so parts can use other languages.

## Status

Exploration. The only artefact so far is a web prototype:

- [`prototypes/capsule-study.html`](prototypes/capsule-study.html) — capsule assembly study: draggable capsules (Jarvis, GitDiscuss), socket joining with end-cap retraction, a release grip, an independent Checks widget that docks into a drawer bay, and paged digital screens. Open it directly in a browser; no build step.

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
