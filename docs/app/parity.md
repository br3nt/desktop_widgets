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

## 0. The original design chat (29 Sep 2026): the rules in Brent's words

The prototype came out of a Codex Desktop session on 29 Sep. The full transcript is
`docs/app/design-chat.md`; Brent's screenshots from it are `docs/app/chat/chat-*.png` (06 = the
first, too-bulky release piece and the overlap artefact; 07 = the wide "⟨⟨⟨⟩" separator he rejected;
08 = the slim vertical bar he approved; 01 = the Dock whose empty left/right ends are the dynamic
docking slots). The Playwright checks Codex wrote against the prototype are in `docs/app/verify/`;
their assertions are the acceptance numbers. Read all of it before building anything. What the
chat settles, beyond the numbers in sections 1–9:

- **What joining means.** "The controls retain their identity and state. The group owns the outer
  casing, so the internal end caps disappear and the exterior ones move outward." Joined =
  `(end cap | Jarvis controls | GitDiscuss controls | end cap)`. Joining two widgets visually must
  never connect their workflows.
- **Sequence of a join** (agreed 09:15–09:27): "compatible sockets illuminate, the inner caps
  retract, the contents slide together, and a shared bevel closes over the seam. Pulling a module
  away reverses it."
- **The feel is hardware** (09:37): "like when a ship gets locked in and guided in... or when
  magnets come close together, there is a tactile feeling that they want to come together, and you
  need to apply force to stop that happening." Codex's plan Brent approved: approach = sockets wake
  and the dragged capsule pulls toward alignment; capture = it settles into a groove and resists a
  little before breaking free; latch = release completes the join with "a short, firm seating
  motion, more precision mechanism than rubbery bounce"; undock = ordinary dragging moves the
  assembly, a dedicated release grip pulls one capsule against the latch until it gives way. "We
  shouldn't move the pointer or make the interface feel like it's fighting you. Attraction should
  help, and pulling away should always work."
- **The glow is a promise, on every connector** (09:48–09:50): "if the connection is lit up, that
  if I drop, it WILL capture the widget... two stages of the light up glow... close... a connection
  point is near by, but we wont connect, then bright... connection will happen when you let go."
  "I think all connections should have this." The bright state uses the same condition as the drop
  action; pulling away clears it immediately. Both stages must visibly glow (09:58: "i expected
  there still to be glow, but one is brighter than the other"). Measured in the checks: nearby glow
  > idle + 5 luminance, ready > nearby + 10, on the bay and on both capsule sockets.
- **The glow curve** (10:21, verbatim): "there needs to be a definitive start... some distance in
  which we want to indicate that a connection is close. At this distance, there a low dim. And
  moving into and out of this boundary, the dim turns on and off with a relatively fast but
  notible easing. Then, as the widget is moved closer to the docking point, the brightness
  increases and moving away decreases... medium brightness. Then when we are at the distance that
  docking will definitely take place, full glow. the glow isn't dependent on distance any more, but
  one solid glow within the definite docking boundary. But transitioning into and out of the
  definite docking boundary again uses easing... fast but noticable." The checks assert: entering
  value between 0 and dim; dim < mid < high < 0.8 across the band; the capture crossing is eased
  (a value between high and full is observable); leaving fades rather than switching off.
- **Same distances everywhere** (10:21): "the proximity and definite lock distances are different
  for the draw dock and the bigger widget docking points. I think it should be the same." Codex
  fixed it to radial 120 px proximity / 40 px capture for both, equal in every direction.
- **No second-order rules** (10:21): "if get close then move the widget away, then move it close
  again, the second time the connection doesnt light up." The cause was a `blocked` flag; removed.
  "Readiness now depends on current proximity." Never reintroduce a lockout or reset boundary.
- **Don't change behaviour when asked for visuals** (09:58): "you change the magnetic behaviour...
  i didnt ask for that... i just asked for the visual indicators to change." Capture and escape
  thresholds are fixed; visual work must leave them alone.
- **The release control** (09:48, 09:58, 10:21): the first grip was "waaaay to big and bulky",
  the next "too wide... too much empty chrome space". Approved: a slim vertical pull bar, ink
  ≤ 6 px wide and 20–28 px tall, hit target ≥ 20×32, geometrically centred between the two
  displays (gap centre and bar centre within 0.1 px) with highlight and shadow balanced so it also
  looks centred; display gap 22 px, never the old 52.
- **Layering** (09:48): shells, caps and displays are drawn per capsule, front to back; a rear
  capsule's end cap must never paint over the front capsule's casing.
- **The drawer** (09:37): "its rails guide the widget into position, it seats with a small
  mechanical click, and the drawer has a definite closed stop." Brent: "I love the draw and
  docking the ui element into the draw!" A contained widget travels with the drawer when it closes
  and is still there when it reopens.
- **Two layers of UI** (09:27): "chrome" is the physical housing, buttons, knobs and drawers, a
  thing "you could literally 3d print and hold in your hand"; "ui surface" is the digital display
  inside it, whose content depends on the program. Display content slides between pages while the
  housing stays fixed.
- **Design mode** reveals the authored connection points, previews valid joins and can disable
  joining; a skin author may define no sockets at all. Skins may also define bays that hold other
  widgets, like the drawer, "much like the dynamic docking points either side of the Mac dock".
- **Dynamic Dock slots** (08:40–08:43): left slot = screen left edge → Dock left edge, right slot
  = Dock right edge → screen right edge, height = stable Dock height. Permission-free detection
  from the window list (`CGWindowListCopyWindowInfo`, Dock-owned windows) with `visibleFrame` as
  the fallback, Accessibility-based precision as an optional enhancement. Recalculate on display,
  orientation, auto-hide, icon count and scale changes; ignore transient magnification; apply
  hysteresis so tiny changes don't resize docked widgets. If a slot gets too narrow, widgets step
  large → medium → small, stack, or undock. Dropping into a slot uses the same field and two-stage
  glow as every other connector.
- **Acceptance numbers from the checks** (prototype coordinates, capsule W = 390): B's rest x is
  594 (684 in the magnetic checks after reset), A at 90. Dragging B's grip 90 px left and 24 px
  down must leave B left of 584 and above 233 while held (pull and guidance), not joined. Dragging
  140 px left then back 30 px: B stays left of 563 (resists the 30 px outward pull); a further
  60 px out tracks the pointer again within 4 px and release leaves them separate. Captured
  release joins; a 35,15 drag of the joined pair moves both by exactly 35. With joining disabled
  in design mode there is no attraction at all (B within 1 px of its raw pose). Release grip: a
  35 px pull keeps joined with B shifted < 15 px and reseats within 1 px on release; 120 px
  separates and keeps A's page and run state. Bay: Checks at (260,425) dragged 10,10 toward the
  bay moves below x 242 and y 408 (attracted toward 201,380); a 10,10 pull after docking does not
  eject; 100,80 ejects. Escape mid-drag restores exactly (B back to 684, Checks still docked).
  Reduced motion keeps every state and keyboard Enter on the grip separates.

## 1. Magnetic guidance while dragging (the feel)

The prototype's rule, `proximity()` and `magneticGoal()`:

- `target` is the dragged widget's **aligned pose** next to a compatible socket (for capsule B next
  to A: `x = a.x + W + 12, y = a.y`). Distance is measured from the raw pointer-driven position to
  that pose, in points.
- Bands: `near = 120`, `capture = 40`.
  - `> 120`: free. The widget is 1:1 with the pointer. No feedback.
  - `120 → 40` (approach): pull strength `s = (1 - d/120)^0.8 * 0.85`; the goal is
    `raw + (target - raw) * s`. Measured: at 110 px the widget sits 11 px closer than the pointer;
    at 60 px, 29 px closer. (The code wins: the formula gives 12.8 px at 110 and 29.3 at 60; the
    11 was read before the spring had settled.)
  - `≤ 40` (captured): `s = 0.88`; the widget sits almost on the pose. Releasing here joins.
- Motion toward the goal is a **critically damped spring**, not a lerp and not a hard snap:
  `v += ((goal - x) * 400 - v * 40) * dt` in three sub-steps per frame, `dt` capped at 32 ms. Settles
  to the goal with no bounce. Outside the field, position follows the pointer exactly with no spring.
- **No hysteresis, no cooldown, no escape gate.** Move away and the pull and glow fade the same way
  they grew; approach again and it behaves exactly like the first time. (The prototype's design
  record: "No hidden penalty for changing direction.")
- Every connector uses the same rule: capsule sockets and the drawer bay (section 5).
- Snapping to screen edges may stay as it is (the prototype has none); it must not fight the field.
- App generalisation (built in `host/src/magnet.rs`): every widget with `sockets` (default true;
  Checks sets `sockets = false`) has a socket at each end. An end is free unless a group member
  sits flush against it; shaded and docked widgets offer none. The aligned pose puts the two ends
  12 pt apart with their **centres** level (the prototype's `y = a.y` with equal heights). For a
  moving group, the nearest free facing pair over all members wins. Edge snapping (to widget and
  screen edges) applies only while the drag is free. Until the join animation (section 3) exists,
  release while captured seats the pair flush (no gap, no overlap) and merges the groups; flush
  contact on its own no longer joins.

## 2. Proximity feedback (the glow is a promise)

- Socket lights: a lime glow strip at the inner end of **both** the dragged widget and its partner
  (`socketLight()`: a 12×70 rounded rect with a blur filter, plus a 4×63 bright core, at the socket
  edge).
- Brightness from the same rule: `0` outside 120 px; `0.18 + 0.44 * ((120 - d) / 80)^0.8` in the
  approach band (0.25 at 110 px, 0.53 at 60 px; the formula gives 0.263 and 0.529); `1.0` when
  captured.
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

- [x] FIX (reviewer): Run `probe.sh drag gitdiscuss 300 0 --shot /tmp/d.png` live on an unlocked
      screen and record in the parity note: mid-drag `drag.moved = true`, `drag.members` = all 3
      group members, every member moved by 300 ±1 pt after release, and both PNGs show the casings
      (not wallpaper). Fix probe.sh or the app if any of these fail.
      Verified live 13 Oct 18:35 ACDT (29 earlier runs were blocked by a locked screen): release
      build, `probe.sh drag gitdiscuss 300 0 --shot /tmp/d.png` showed mid-drag `moved=true`,
      `members=jarvis,gitdiscuss,checks`, cursor 1096,306; after release jarvis 624→924,
      gitdiscuss 624→924, checks 984→1284 (exactly +300, y unchanged), panel-demo untouched,
      `drag: none`; both PNGs show the three casings over the desktop. No code changes needed;
      layout.toml restored afterwards.
- [x] FIX (reviewer): Run `probe.sh drag gitdiscuss 300 0 --escape` live and confirm the group
      returns to its start pose (x=624 ±1). The app is an Accessory app (`main.rs:30`) and Escape
      is a `KeyboardInput` on the widget window (`app.rs:598`), so the key may never arrive if the
      window is not key; if it doesn't, make the pressed window key on mouse-down (or drop
      `--escape` from probe.sh and say so).
      Code reading (13 Oct, not yet a live check): winit 0.30.13's `WinitWindow` returns true from
      `canBecomeKeyWindow`, and the casing windows are not non-activating panels, so AppKit should
      activate the app and make the pressed window key on mouse-down. Expect Escape to arrive;
      confirm live before changing any code.
      Verified live 13 Oct 18:36 ACDT: release build, `probe.sh drag gitdiscuss 300 0 --escape
      --shot /tmp/esc.png` showed mid-drag `moved=true`, `members=jarvis,gitdiscuss,checks`,
      jarvis/gitdiscuss at x=924, checks 1284 (drag had started); after Escape + release all three
      back at 624/624/984, y unchanged, `drag: none`; both PNGs show the casings (moved, then
      restored); `layout.toml` byte-identical to before (cancel not persisted); idle CPU 0.0 %.
      Escape reaches the pressed window, so no code changes were needed.

- [x] `GET /state` debug endpoint plus `host/scripts/probe.sh` (cliclick drag helper that prints
      `/state` and takes region screenshots), so every later item can be verified with numbers.
      Verified: unit tests (`snapshot.rs`, `server.rs`); live release build served `/state`,
      reflected a `/send` change, refused a foreign Origin (403). The Mac was locked during this
      run, so the cliclick drag and screenshot path is not yet exercised live; `probe.sh` now
      refuses drags and shots while the screen is locked. Also unverified: whether `--escape`'s key
      press reaches the widget window.
- [x] Magnetic field (section 1): bands, pull strengths, critically damped spring, 1:1 outside the
      field, no hysteresis. Replace the hard edge-snap join with release-while-captured.
      Verified 13 Oct: unit tests (`magnet.rs`, `model.rs`); live release build, GitDiscuss held
      at 110 / 60 / 25 pt from its aligned pose (672,300) settled at offsets −130.8 / −197.3 / −225
      (formula: raw − 12.8 / 29.3 / 22), feel approach / approach / captured, `settled=true` and
      0.0 % CPU while held still; pulled out to 178 pt it was 1:1 at once; back at 60 pt it was
      −197.3 again; release at 25 pt joined flush at x=660 in one group. Pair drag, Escape and
      Option-drag unchanged. No glow yet (next item).
- [x] Socket lights and eased glow (section 2), with the status caption while dragging.
      Verified 13 Oct: unit tests (`glow.rs` easing/curve/caption, `model.rs` lights per facing
      socket, `render.rs` pixel check near > idle + 5 and ready > near + 10 on both ends,
      `snapshot.rs`); live release build, GitDiscuss at 60 pt from Jarvis's right socket lit both
      facing ends at 0.53 with "Socket nearby · move closer" under it, at 25 pt 1.0 with "Ready ·
      release to join"; pulled free it faded 1 → 0.18 → 0.03 → 0 over ~150 ms; re-approach relit
      0.53 / 1.0 (no lockout); release captured joined at 560 and the lights faded out; Escape
      and an approach-band release both faded out and cleared the caption; idle CPU 0.0 %.
      The caption is a small click-through window under the pressed widget (above it near the
      screen bottom). Screenshots match `proto-03`/`proto-04`.
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
- [ ] Dynamic Dock slots (section 0, last bullet): slots from Dock geometry with visibleFrame fallback,
      hysteresis, ignore magnification, same field and two-stage glow when dropping a widget into a slot,
      slot widgets inherit always-on-top, recalculated on display and Dock changes.
- [ ] Design mode and the sockets toggle (section 7).
- [ ] Reduced motion, keyboard nudges, latch Right-arrow (section 9).
- [ ] Final side-by-side review: every section verified against the prototype with `/state` numbers
      and screenshots; `docs/app/review.md` says parity reached or lists what's left.
