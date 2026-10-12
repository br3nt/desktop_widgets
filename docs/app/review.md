# Review

## Item

`GET /state` debug endpoint plus `host/scripts/probe.sh` (cliclick drag helper that prints
`/state` and takes region screenshots). Commit 58ae572.

## Verified

- `cargo build --release` clean; `cargo test --release`: 54 passed, 0 failed (includes
  `snapshot.rs` x3 and `server.rs` `/state` 200/403/503 tests).
- Live release build: `GET /state` → 200 `application/json` in 0.5 ms. Reports 4 widgets
  (jarvis 624,408 360x116; gitdiscuss 624,292; checks 984,292 260x104; panel-demo 155,535 shaded
  h=26), groups (g5 = jarvis, gitdiscuss, checks), 2 docks, 1 screen, `drag: null`, grip/panel/
  button rects in screen points, `grab` point on the nameplate, actor state.
- `POST /send {"to":"jarvis","msg":"set","agents":5}` → `/state` shows `actor.agents = 5`
  within 300 ms.
- Foreign `Origin` → 403; `Host: evil.example` → 403; no CORS header.
- `probe.sh summary`, `state FILTER`, `grab checks` (→ `1106,306`) work; `grab nope` exits 1
  with a message.
- Idle CPU: 0.0 % over 10 s; 50 back-to-back `/state` calls cost 0.01 s CPU total and the
  process returned to 0.0 % idle.
- `layout.toml` unchanged by the run.
- **Not verified: the screen was locked again** (`CGSSessionScreenIsLocked = true`).
  `probe.sh drag` and `probe.sh shot` correctly refuse with "the screen is locked" (exit 1), so the
  real-mouse path (cliclick drag → app moves the group → mid-drag `/state` → screenshot) has
  still not been run once. Nothing to compare against the `refs/proto` frames for this item.

## Must fix

- Run `probe.sh drag gitdiscuss 300 0 --shot /tmp/d.png` live on an unlocked screen and record in
  the parity note: mid-drag `drag.moved = true`, `drag.members` = all 3 group members, every member
  moved by 300 ±1 pt after release, and both PNGs show the casings (not wallpaper). Fix probe.sh
  or the app if any of these fail.
- Run `probe.sh drag gitdiscuss 300 0 --escape` live and confirm the group returns to its start
  pose (x=624 ±1). The app is an Accessory app (`main.rs:30`) and Escape is a `KeyboardInput` on
  the widget window (`app.rs:598`), so the key may never arrive if the window is not key; if it
  doesn't, make the pressed window key on mouse-down (or drop `--escape` from probe.sh and say so).

## Should fix

- Spec section 10 asks `/state` for "drag feel"; nothing reports it yet. Add it (band, pull
  strength, captured target, spring offset) with the magnetic-field item, since that is the
  first item that needs it.
- `light` and `tween` are placeholders (0.0 / null); fill them with the glow and join items.
- `/state` waits up to 1 s for the event loop and the server is single-threaded, so a stalled
  loop also stalls `/send` for that second. Fine for a debug endpoint; worth a comment.
- Docks report `y = 1067` with the screen ending at 38 + 1029 = 1067, i.e. entirely below the
  visible frame. Check against the dock placement when the bay item lands.

## Verdict

NOT YET: the endpoint is at parity with the spec; the cliclick drag/screenshot path, which is the
point of `probe.sh`, is still unexercised because the Mac was locked for both runs.
