# Relay

A skinnable desktop shell of **capsules**: small widgets with hardware-style casings that you build, arrange, and dock together. Think Sonique-era media players: widgets that snap end-to-end, carry drawers and bays, and change shape and size to suit what they're showing.

## Status

Exploration. The only artefact so far is a web prototype:

- [`prototypes/relay-capsule-study.html`](prototypes/relay-capsule-study.html) — capsule assembly study: draggable capsules (Jarvis, GitDiscuss), socket joining with end-cap retraction, a release grip, an independent Checks widget that docks into a drawer bay, and paged digital screens. Open it directly in a browser; no build step.

## Direction

- **Desktop app on every platform** (macOS, Windows, Linux). The web prototype is the design study, not the target.
- **Form factors** (not built yet): capsules expand, collapse, and morph between sizes and shapes, as Sonique did.
- **Skinnable**: casing geometry and screens are data, not hard-coded.
- **macOS dock regions**: dockable zones either side of the Dock that capsules can attach to, adjusting as the Dock and screen change.

## Open questions

- Shell: Tauri, Electron, or native per platform?
- Window model: one transparent always-on-top surface, or one OS window per capsule?
- Widget contract: what a capsule declares (screens, controls, sockets, sizes) and where its data comes from.
