# Prior art

Researched 2026-09-30. Rows marked *(background)* come from general knowledge rather than a source checked on the day. The 2026 indie entries rely on thin sources, so check their repos before leaning on them.

| Name | Years | Status | Platforms | Verdict |
|---|---|---|---|---|
| Konfabulator / Yahoo! Widget Engine | 2003–2012 | Dead: Yahoo shut it down ([Wikipedia](https://en.wikipedia.org/wiki/Dashboard_(macOS))) | Mac, Win | Invented the skinnable, art-first desktop widget (XML + JS). Killed by its acquirer, not its tech. |
| Apple Dashboard | 2005–2019 | Dead: off by default from Yosemite, removed in Catalina ([AppleInsider](https://appleinsider.com/articles/19/06/13/dashboard-has-been-permanently-pulled-from-macos-catalina)) | macOS | A separate overlay layer. Out of sight, out of mind. |
| Windows Sidebar / Gadgets | 2006–2012 | Killed: widgets ran with the user's full permissions, and after a 2012 security hole Microsoft disabled the feature rather than fix it ([bit-tech](https://bit-tech.net/news/tech/software/microsoft-kills-sidebar/1/), [SANS ISC](https://isc.sans.edu/diary/13651)) | Vista, 7 | Unrestricted gadgets were an attack surface. |
| Windows 11 Widgets board | 2021– | Active; third-party widgets via Windows App SDK ([Windows Central](https://windowscentral.com/windows-11-may-soon-support-third-party-widgets)) | Win 11 | A pop-out panel, not the desktop. Only seen when summoned. |
| Google Desktop gadgets | 2005–2011 | Discontinued "for the cloud" ([SlashGear](https://www.slashgear.com/1628912/why-google-discontinued-desktop-app)) | Win, Mac, Linux | Went down with the product that hosted it. |
| Opera Widgets | 2006–~2012 | Dead after Opera 12 ([OMG Ubuntu](https://www.omgubuntu.co.uk/2009/12/move-over-screenlets-opera-desktop-widgets-come-to-town)) | Cross-platform | Only ran inside one vendor's browser. |
| Samurize | 2003–~2010s | Dormant *(background)* | Win | System-meter configuration tool; inspired Rainmeter. |
| Rainmeter | 2001– | Alive: 4.5.26, May 2026 ([history](https://docs.rainmeter.net/history/)) | Win only | The standard for community-shared skins. No widget-to-widget model, no macOS. |
| DesktopX / ObjectDock / WindowBlinds (Stardock) | 1999–2010s | Mostly legacy or paid *(background)* | Win | Theming as a business. Scriptable objects that could talk to each other. |
| Sonique | 1999–~2004 | Dead *(background)* | Win | Radical non-rectangular UI, three sizes, fully skinnable. |
| Winamp skins; Webamp | 1997–; Webamp active | Webamp renders classic skins in the browser, with window snapping ([docs](https://docs.webamp.org/docs/features/skins/)) | Win, Web | The best-known docking-window design that shipped. The closest ancestor of this project's snapping. |
| SuperKaramba → Plasma plasmoids | 2004–; Plasma 6 | Plasmoids alive ([Wikipedia](https://en.wikipedia.org/wiki/KDE_Plasma)) | Linux | Widgets built into the desktop itself. Good design, but tied to KDE. |
| gDesklets / adesklets | 2003–~2010s | Dead or archived *(background)* | Linux | Fragmented across competing GUI toolkits. |
| Conky | 2005– | Alive *(background)* | Linux | Text-first system monitor; loved for being light and scriptable. |
| GeekTool | 2000s– | Abandoned; Intel-only ([RosettaCheck](https://rosettacheck.com/apps/org.tynsoe.GeekTool)) | macOS | Pins shell-command output to the desktop. |
| Übersicht | 2013– | Alive; runs natively on Apple Silicon | macOS | HTML/JS widgets on the desktop layer. Niche. |
| SketchyBar | 2020– | Alive: ~11.7k stars ([GitHub](https://github.com/felixkratz/sketchybar)) | macOS | Scriptable menu-bar replacement. |
| macOS WidgetKit desktop widgets | 2023– | Active | macOS | Interactive, but can't float above windows ([Six Colors](https://sixcolors.com/post/2023/07/first-look-macos-sonoma-public-beta/)), and no custom casings. |
| Plash | 2020s | Alive *(background)* | macOS | A webpage as the desktop wallpaper. |
| Zebar, YASB, Widgetsack, Wigify, Eww, AGS | 2020s | Alive; Zebar GPL-3.0, ~2.7k stars | Varies | Modern Rainmeter successors. Zebar is the closest cross-platform effort, but it makes bars and pop-ups; nothing docks together. |
| BumpTop; Stardock Fences | 2006–; Fences active | BumpTop acquired and dropped by Google in 2010 *(background)* | Win/Mac | Arranging and grouping items on the desktop as physical objects. |
| Yahoo Pipes; Quartz Composer | 2007–2015; 2005–2020s | Pipes shut down; QC deprecated *(background)* | Web, Mac | Loved visual wiring tools that their platform owners abandoned. |
| OpenDoc | 1992–1997 | Killed by Apple *(background)* | Mac | The ideal of documents built from parts that share state. Too early. |
| Docky; Deskmat | 2026 | Docky 0.7.0 went free and open source June 2026 ([it-connect](https://www.it-connect.fr/docky-le-dock-de-macos-reinvente-gratuit-et-open-source/)) | macOS | Dock replacements with widgets. Shows people want widgets around the Dock. |
| Agent-status widgets (WidgetAI etc.) | 2026 | Small indie projects | macOS | Single-purpose; can't be combined with other widgets. |

## What worked

- **Skins as identity.** Winamp, Sonique and Rainmeter communities formed around trading skins. That's why Rainmeter is still going after 25 years.
- **Snapping.** Winamp and Webamp windows snap to each other and to screen edges. Winamp's "shade mode" collapses a window to a thin strip, a smaller form of the same window.
- **Easy authoring.** Plain text, HTML or shell: Conky, Übersicht and Rainmeter all kept the bar low.
- **Always visible.** macOS widgets sitting under windows and Windows 11's pop-out panel both show what's lost when widgets hide.
- **Scripting glue.** Shell output and plugins let power users connect widgets to anything.

## What killed them

- **Security.** Gadgets ran with the user's full permissions, and one hole ended the whole feature.
- **Owner neglect.** Dashboard, Yahoo Widgets, Google gadgets, BumpTop, Pipes and OpenDoc were all left to die or shut down by the companies that owned them.
- **Platform shifts.** The move to the web and the cloud made desktop widgets look redundant.
- **Tied to one vendor.** Opera and Google widgets died with the products that hosted them.
- **Fragmentation.** Linux widget engines split across toolkits; Rainmeter never left Windows.
- **Money.** Stardock's paid theming shrank, and almost nobody made widget stores pay.
- **Platform lockdown.** Apple's WidgetKit is sandboxed with no custom casings; Microsoft curates Windows 11 widgets.

## Where this project is new

1. **Widgets that physically join into one shape.** Nothing since Winamp and Webamp does this, and they are media players, not general widget frameworks.
2. **Docking regions either side of the macOS Dock.** Docky and Deskmat replace the Dock instead.
3. **A shared scripting runtime so widgets can talk and share state.** Existing widget hosts keep widgets isolated. Pipes and Quartz Composer wired things together, but they weren't widgets.
4. **Widgets written in different languages working together.** No widget host found does this.
5. **Casings that change size and shape**, cross-platform.
6. **AI agents as widgets that can be combined** with feeds, chat and bookmarks.
7. **Deliberately staying out of the Dock and taskbar while always being on screen.**

## Lessons to apply

- **Sandbox widgets from day one.** Each one declares what it can touch (network, filesystem, other widgets). Unrestricted access is what killed Windows Gadgets.
- **Ship the core first:** Winamp-style snapping plus a Winamp shade-mode equivalent. Shared scripting comes after.
- **Web-tech rendering (Tauri) is viable.** Zebar and friends prove it, and it keeps authoring easy.
- **Make skins easy to share:** a single file or folder.
- **Local-first, open format and licence.** Don't depend on a company's product that can be shut down.
- **Float above windows.** WidgetKit widgets can't, and it's their most common complaint.
- **Keep the widget-to-widget layer small:** events plus shared state. Don't rebuild OpenDoc.
- **Test Dock-side placement early.** It needs native macOS window control (which layer a window sits on, Spaces, Dock auto-hide and resize) that Tauri may not expose. No prior art was found that solves it.
- **Don't appear in the Dock or taskbar.** Provide a global hotkey and a menu-bar or tray item so widgets stay reachable.
