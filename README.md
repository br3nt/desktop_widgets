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

- **Desktop app on every platform** (macOS, Windows, Linux). The web prototype is the design study, not the target.
- **Form factors** (not built yet): capsules expand, collapse, and morph between sizes and shapes, as Sonique did.
- **Skinnable**: casing geometry and screens are data, not hard-coded.
- **macOS dock regions**: dockable zones either side of the Dock that capsules can attach to, adjusting as the Dock and screen change.

## Open questions

- Shell: Tauri, Electron, or native per platform?
- Window model: one transparent always-on-top surface, or one OS window per capsule?
- Widget contract: what a capsule declares (screens, controls, sockets, sizes) and where its data comes from.
