# Review

## Item

Magnetic field (section 1): bands, pull strengths, critically damped spring, 1:1 outside the
field, no hysteresis. Replace the hard edge-snap join with release-while-captured (commit 9b6aad9).

## Verified

- Code against the prototype (`proximity`, `magneticGoal`, `runDragFrame`, lines 334-388):
  `magnet.rs` has the same bands (120 / 40), the same strength (`(1 - d/120)^0.8 * 0.85`, 0.88
  captured), the same spring (`400`, `40`, 3 sub-steps, dt ≤ 32 ms, 0.03 / 0.1 rest snap), and
  distance from the raw pose to the aligned pose. No capture history anywhere. 73 tests pass.
- Live, release build, Jarvis at 200,300 and GitDiscuss dragged toward Jarvis's right socket
  (aligned pose 572,300), pointer held still 800 ms at each distance:
  - 110 pt: offset −230.8 vs raw −218 (12.8 closer), `approach`, brightness 0.26.
  - 60 pt: −297.3 vs −268 (29.3 closer), `approach`, 0.53.
  - 25 pt: −325 vs −303 (22 = 25 × 0.88), `captured`, 1.0.
  - Pulled out to 178 pt: `free`, offset = raw (−150) 30 ms later, no spring.
  - Back to 60 and 25 pt: identical numbers to the first pass (no hysteresis).
  - Release at 25 pt: GitDiscuss at 560,300, flush, same group.
- Spring timing: jumped from 300 pt away to a raw pose 60 pt right / 30 pt below the aligned pose;
  sampled `/state` every ~34 ms. Offset ran −17 → −179 → −236 → −277 → −290 → −294.49 x and
  1 → 16.75 y, monotonic, no overshoot, `settled=true` after ~0.74 s. Goal −294.49,16.75 matches the
  formula for d = 67.1. Released there (approach band): stayed at 606,317 and did not join.
- Off-axis capture (raw −14,−17 from the pose, d = 19.5): joined flush at 560,300.
- Slow sweep from 115 pt on the overlapping side through the pose in 2 pt steps: `approach` down
  to 41, `captured` from 39, smooth position, the same partner throughout, no flicker.
- Shaded panel-demo dragged next to Jarvis's left end: `free`, no partner (shaded offers none).
- Screenshots at 60 and 25 pt match the geometry of `proto-03-near-60px.jpg` and
  `proto-04-ready-25px.jpg`. No glow and no status text yet (next item, by design).
- Regressions: group drag +100,+50 moved both members exactly; Option-drag separated; double-click
  shade/unshade; Checks into dock-left (shaded, packed at 109,1079) and back out; `layout.toml`
  written on join. Idle CPU 0.0 % and 0.0 % while holding the button still in the field (no frames
  once settled).

## Must fix

- None.

## Should fix

- Once in ~10 scripted drags the release never reached the app: after `cliclick du:` the drag stayed
  live (`drag.pressed=gitdiscuss`) and GitDiscuss followed the bare pointer until the next click.
  Not reproduced in 8 more tries, and it may predate this item. The prototype cancels on
  `lostpointercapture`. Guard against it: on each `CursorMoved`/spring tick during a drag, if
  `NSEvent.pressedMouseButtons` has no left button, end the drag as a release. Test: post `dd`,
  some `dm`, then a mouse-up event the window never sees (e.g. `du` over another app after
  raising it), and check that `/state` shows `drag: none` after the next pointer move.
- Release while captured jumps to flush (up to ~17 pt) instantly. Fine until the join animation
  (section 3) replaces it; make sure that item starts from the spring's current position.

## Verdict

PARITY: the bands, pull strengths, spring and 1:1 free drag match the prototype's numbers live, with
no hysteresis and no idle cost.
