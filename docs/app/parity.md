# App parity with the capsule prototype

The host app in `host/` must feel and behave at least as well as `prototypes/capsule-study.html`. This
file is the spec, extracted by Claude on 13 Oct 2026 by reading the prototype's code and its design
record (the "Design record" section at the bottom of the page), driving it with real pointer events,
and comparing it with the app. The numbers below are the prototype's own numbers; match them unless a
line says otherwise. Brent's words: "we had so many discussions around the correct magnetic feel and
about the visual feedback the user should have."

Reference frames (outside the repo): `~/Development/desktop_widgets-worktrees/refs/proto/`.
`proto-*.jpg` are the prototype at each stage below; `app-*.jpg` are the app on 13 Oct for comparison.

Keep everything the app already has that the prototype lacks (shade mode, docks, always-on-top per
group, persistence, actors and the endpoint). Add, don't replace.

## 1. Magnetic guidance while dragging (the feel)

The prototype's rule, `proximity()` and `magneticGoal()`:

- `target` is the dragged widget's **aligned pose** next to a compatible socket (for capsule B next
  to A: `x = a.x + W + 12, y = a.y`). Distance is measured from the raw pointer-driven position to
  that pose, in points.
- Bands: `near = 120`, `capture = 40`.
  - `> 120`: free. The widget is 1:1 with the pointer. No feedback.
  - `120 → 40` (approach): pull strength `s = (1 - d/120)^0.8 * 0.85`; the goal is
    `raw + (target - raw) * s`. Measured: at 110 px the widget sits 11 px closer than the pointer;
    at 60 px, 29 px closer.
  - `≤ 40` (captured): `s = 0.88`; the widget sits almost on the pose. Releasing here joins.
- Motion toward the goal is a **critically damped spring**, not a lerp and not a hard snap:
  `v += ((goal - x) * 400 - v * 40) * dt` in three sub-steps per frame, `dt` capped at 32 ms. Settles
  to the goal with no bounce. Outside the field, position follows the pointer exactly with no spring.
- **No hysteresis, no cooldown, no escape gate.** Move away and the pull and glow fade the same way
  they grew; approach again and it behaves exactly like the first time. (The prototype's design
  record: "No hidden penalty for changing direction.")
- Every connector uses the same rule: capsule sockets and the drawer bay (section 5).
- Snapping to screen edges may stay as it is (the prototype has none); it must not fight the field.

## 2. Proximity feedback (the glow is a promise)

- Socket lights: a lime glow strip at the inner end of **both** the dragged widget and its partner
  (`socketLight()`: a 12×70 rounded rect with a blur filter, plus a 4×63 bright core, at the socket
  edge).
- Brightness from the same rule: `0` outside 120 px; `0.18 + 0.44 * ((120 - d) / 80)^0.8` in the
  approach band (0.25 at 110 px, 0.53 at 60 px); `1.0` when captured.
- Brightness eases toward its target each frame: `v += (target - v) * (1 - exp(-dt / 40 ms))`. It
  eases out when leaving, never cuts.
- Meaning: dim-to-medium = nearby but releasing will **not** connect; full = releasing **will**
  connect. The same mapping for every connector.
- Status text changes with the state (the prototype's `#phase`): "Socket nearby · move closer",
  "Ready · release to join", "Magnetic hold released", "Latch tension · pull right". In the app, show
  this somewhere unobtrusive (a small caption under the dragged widget, or in the nameplate) while
  dragging; it must not need a separate window to read.

## 3. Join animation and the shared housing

On release while captured (`join(fromDrag = true)`), three phases, smoothstep easing
(`t*t*(3-2t)`), each must be visible:

1. **Sockets illuminated**: light rises to 1.0 over 90 ms.
2. **Caps retracting**: the inner end caps of both widgets retract (`cap` 0 → 1 over 160 ms; the cap
   is drawn with `scale(1 - retract)` so it shrinks into the shell).
3. **Closing the shared casing** (310 ms): both widgets glide to their final poses. The final
   spacing is `JOIN_DISTANCE = 316` with `W = 390`: the shells **overlap by 86 px**; the inner caps
   are gone. In the last 32% of this phase the two individual shells cross-fade (`blend`) into **one
   continuous casing** drawn from A's left edge to B's right edge, with only the outer caps (cyan
   left, lime right). The light fades to 0 as the casing closes.

Joined state (`proto-08-joined.jpg`): one casing, two displays side by side, a **narrow centred
vertical grip** between them (the release latch, section 4) with a larger invisible grab area. The
displays never move or change during the join: "Preserve useful centres; relocate ornament."

Pressing Join from a button (not a drag) uses the slower timings 260 / 340 / 750 ms.

Draw each detached widget as one coherent stack (shell, caps, display together), and the joined pair
as one stack. Joining must not steal clicks from a neighbour's buttons.

## 4. Separation: the release latch

- Joined widgets move as one when any nameplate is dragged.
- To separate: drag the slim vertical release grip to the right. Tension `= dx / 72` clamped 0..1
  drives a visual on the latch (`--tension`); B shifts right by up to 10 px (`dx * 0.14`). Status:
  "Latch tension · pull right".
- Release before 72 px: **reseat** (B slides back over 140 ms, tension decays). Status: "Latch
  reseated."
- At 72 px: **breakaway**. The pair becomes two; the inner caps re-extend over 180 ms while B keeps
  following the pointer; the aligned pose shifts to account for the re-extended caps. From here it is
  an ordinary drag, so moving B back toward A re-enters the magnetic field with no penalty. Status:
  "Latch released. Keep dragging away; Escape restores the assembly."
- **Escape** during any drag cancels it and restores the exact pre-drag assembly (positions, joined
  state, drawer contents).
- Keep the app's Option-drag as a shortcut, but the latch is the primary, discoverable way.

## 5. The drawer (activity bay) and cartridges: containment

- Jarvis has a drawer. Its "▾ activity bay" tab button opens it: a tray slides down from beneath
  Jarvis's casing (`transition: transform 450ms cubic-bezier(.2,.75,.2,1)`), anchored at
  `(a.x + 36, a.y + 121)`, with a title "Activity cartridge bay", a close button, an empty recess
  labelled "Drop the Checks widget here" with an "Insert Checks widget" button, and a small readout
  (activity running, a concurrency knob 1..6 that cycles on click). Opening it pushes Jarvis up if it
  would run off the bottom of the stage (`a.y > 310 → 310`), and the joined partner moves with it.
- The recess is an **authored bay**: its target pose is `(a.x + 59, a.y + 165)`. Dragging Checks
  toward it uses the section 1 rule exactly (120 / 40, same pull strengths, same spring). Feedback:
  the recess glows lime with the section 2 brightness; its label reads "Nearby · move closer" then
  "Release to dock".
- Release while captured: Checks tweens into the recess (170 ms if within 75 px, else 350 ms), seats
  at `scale(.98)` and is **latched**. The Insert button does the same from anywhere.
- A latched cartridge is pulled out deliberately: dragging its nameplate deflects it by
  `clamp(dx * 0.13, ±5)` px (it resists); at 46 px of pointer travel it releases and becomes a free
  drag. Releasing before that reseats it (140 ms). "Eject" frees it to `(a.x + 430, a.y + 190)`.
- Closing the drawer keeps its contents loaded; separating the capsules keeps the drawer with Jarvis.
- Checks has **no fusion sockets**: it cannot capsule-join, only dock in a bay. A skin may opt out of
  joining; containment is a separate capability.

## 6. Displays: pages and physical controls

- Each capsule display has three **pages** (Jarvis: Status / Task / Log; GitDiscuss: Queue / Thread /
  Log) with a tab line under the display and a "next" lens button; pages slide horizontally
  (`transition: transform 280ms cubic-bezier(.2,.7,.2,1)`). Content per page as in the prototype
  (event log lines, "need a decision", etc.).
- A **run/pause** lens button (orange) toggles the simulated activity; the status line reflects it
  ("03 agents running" / "03 agents paused", "Activity running" / "Activity paused").
- GitDiscuss's "▸ view thread" tab jumps to its Thread page.
- The display contents and page survive joining, separating, docking and ejecting ("Each display
  keeps its own screen and state").
- These controls are on the casing (physical layer) and must keep working in every state.

## 7. Design mode and authored sockets

- A "Design mode" toggle shows the authored connections: dashed outlines of each capsule's sockets
  with labels ("End socket · A", "Joined sockets"), and the bay's outline. Sockets show at 20%
  opacity when joining is disabled.
- "Capsule sockets" can be switched off: new joins are blocked, an existing assembly is retained.

## 8. Look of the chrome (the physical layer)

Match `proto-01-idle.jpg` closely: smoky green-grey brushed metal casing with a 3 px dark outline, a
light highlight arc along the top, a lower rail line, a soft drop shadow; curved end caps with cyan
(left) and lime (right) lens details; inset dark display with a cyan/lime readout; engraved
nameplate; lens buttons (orange run/pause, blue next); tab buttons ("▾ activity bay", "▸ view
thread"). The app's current flat capsule is an approximation; the casing drawing must be upgraded
until a side-by-side at 1:1 reads as the same object. Keep the drawing modular so skins can replace
each part.

Shade mode (the app's own feature) must show the widget name and its one key value unclipped
(currently "12 / 14 p…" is cut off).

## 9. Motion and input rules

- Honour reduced motion: with it on, animations complete instantly.
- Keyboard: arrow keys nudge a focused widget 10 px; on the release latch, Right arrow separates.
- Pointer: dragging only starts after 3 px of movement; a press without movement is a click.
- Zero idle CPU: animate only while a tween, spring or glow is in flight; stop the frame loop when
  everything has settled (the prototype cancels its rAF loops the same way).

## 10. How to verify (both builder and reviewer)

- Prototype truth: `prototypes/capsule-study.html` (serve it over HTTP and drive it, or read the
  code: `DOCKING`, `proximity`, `magneticGoal`, `runDragFrame`, `join`, `separate`, `drawer`,
  `insert`, `eject`, and the pointer handlers) and the frames in `refs/proto/`.
- App: launch the release build, then use `cliclick` for real drags (`cliclick dd:x,y m:x,y du:x,y`,
  `kd:alt` for Option) with `w:` waits, and `screencapture -x -R x,y,w,h out.png` mid-drag. Add and
  use a `GET /state` endpoint on the local server that returns every widget's pose, group, shade,
  dock, drag feel, light brightness and tween phase as JSON, so numbers can be asserted, not guessed.
- Unit tests for every rule in sections 1–5 (pull strength at 110/60/25 px, spring settling without
  overshoot, glow curve, join distance and phase timings, latch thresholds 72/46 px, bay target,
  Escape restore). Pure logic lives in modules without windowing dependencies.

## Checklist (one item per run; builder ticks with a one-line note)

- [ ] FIX (reviewer): Run `probe.sh drag gitdiscuss 300 0 --shot /tmp/d.png` live on an unlocked
      screen and record in the parity note: mid-drag `drag.moved = true`, `drag.members` = all 3
      group members, every member moved by 300 ±1 pt after release, and both PNGs show the casings
      (not wallpaper). Fix probe.sh or the app if any of these fail.
      Attempted 13 Oct: blocked, screen locked again (`CGSSessionScreenIsLocked = true`, a
      `screencapture` of the widget region shows only wallpaper). A raw cliclick drag from
      gitdiscuss's grab point (796,306) by +300 left every widget in place and `/state` showed
      `drag: none` mid-drag, so pointer events don't reach the app while locked either. Nothing in
      this item can be checked until a run happens on an unlocked screen.
      Attempted again 13 Oct (later run): still locked (`CGSSessionScreenIsLocked = true`), and it
      stayed locked through a 10 min wait; nothing launched or changed.
      Attempted a third time 13 Oct: locked at start (`ioreg` `CGSSessionScreenIsLocked = true`)
      and through a 15 min poll every 5 s; release build is current, nothing launched or changed.
      Attempted a fourth time 13 Oct (09:47–10:07 ACDT): locked at start and through a 20 min
      poll every 5 s; release build is current, nothing launched or changed.
      Attempted a fifth time 13 Oct (10:07–10:28 ACDT): locked at start and through a 20 min
      poll every 5 s; release build is current, nothing launched or changed.
      Attempted a sixth time 13 Oct (10:28–10:48 ACDT): locked at start and through a 20 min
      poll every 5 s; release build current, `cargo test --release` 54 passed; nothing launched
      or changed.
      Attempted a seventh time 13 Oct (10:48–11:09 ACDT): locked at start and through a 20 min
      poll every 5 s; release build current, `cargo test --release` 54 passed; nothing launched
      or changed.
      Attempted an eighth time 13 Oct (11:09–11:29 ACDT): locked at start and through a 20 min
      poll every 5 s; release build current, `cargo test --release` 54 passed; nothing launched
      or changed.
      Attempted a ninth time 13 Oct (11:29–11:50 ACDT): locked at start and through a 20 min
      poll every 5 s; release build current, `cargo test --release` 54 passed; nothing launched
      or changed.
      Attempted a tenth time 13 Oct (11:50–12:10 ACDT): locked at start and through a 20 min
      poll every 5 s; release build current, `cargo test --release` 54 passed; nothing launched
      or changed.
      Attempted an eleventh time 13 Oct (12:10–12:31 ACDT): locked at start and through a 20 min
      poll every 5 s; release build current, `cargo test --release` 54 passed; nothing launched
      or changed.
      Attempted a twelfth time 13 Oct (12:31–12:51 ACDT): locked at start and through a 20 min
      poll every 5 s; release build current, `cargo test --release` 54 passed; nothing launched
      or changed.
- [ ] FIX (reviewer): Run `probe.sh drag gitdiscuss 300 0 --escape` live and confirm the group
      returns to its start pose (x=624 ±1). The app is an Accessory app (`main.rs:30`) and Escape
      is a `KeyboardInput` on the widget window (`app.rs:598`), so the key may never arrive if the
      window is not key; if it doesn't, make the pressed window key on mouse-down (or drop
      `--escape` from probe.sh and say so).
      Code reading (13 Oct, not yet a live check): winit 0.30.13's `WinitWindow` returns true from
      `canBecomeKeyWindow`, and the casing windows are not non-activating panels, so AppKit should
      activate the app and make the pressed window key on mouse-down. Expect Escape to arrive;
      confirm live before changing any code.

- [x] `GET /state` debug endpoint plus `host/scripts/probe.sh` (cliclick drag helper that prints
      `/state` and takes region screenshots), so every later item can be verified with numbers.
      Verified: unit tests (`snapshot.rs`, `server.rs`); live release build served `/state`,
      reflected a `/send` change, refused a foreign Origin (403). The Mac was locked during this
      run, so the cliclick drag and screenshot path is not yet exercised live; `probe.sh` now
      refuses drags and shots while the screen is locked. Also unverified: whether `--escape`'s key
      press reaches the widget window.
- [ ] Magnetic field (section 1): bands, pull strengths, critically damped spring, 1:1 outside the
      field, no hysteresis. Replace the hard edge-snap join with release-while-captured.
- [ ] Socket lights and eased glow (section 2), with the status caption while dragging.
- [ ] Join animation, overlap to 316 and the shared continuous casing with inner caps retracted
      (section 3); grouped pair draws as one stack.
- [ ] Release latch between joined displays: tension, reseat, breakaway with caps re-extending,
      Escape restore (section 4).
- [ ] Display pages with sliding transition, run/pause and next lens buttons, view-thread tab
      (section 6).
- [ ] Jarvis drawer: slide-down tray, close, concurrency knob; opening pushes Jarvis up (section 5).
- [ ] Bay docking with the same field and glow; Insert; seated cartridge at scale .98 (section 5).
- [ ] Cartridge latch: resisted pull-out at 46 px, reseat, Eject (section 5).
- [ ] Chrome drawing upgraded to match the prototype 1:1 (section 8), including shade-strip text
      fit.
- [ ] Design mode and the sockets toggle (section 7).
- [ ] Reduced motion, keyboard nudges, latch Right-arrow (section 9).
- [ ] Final side-by-side review: every section verified against the prototype with `/state` numbers
      and screenshots; `docs/app/review.md` says parity reached or lists what's left.
