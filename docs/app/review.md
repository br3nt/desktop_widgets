# Review

## Item

FIX (reviewer): Run `probe.sh drag gitdiscuss 300 0 --shot /tmp/d.png` live on an unlocked screen
and record the result (commit 5f7c873, docs only).

## Verified

- Screen unlocked (`CGSSessionScreenIsLocked` absent from `ioreg`). Release build current.
- `probe.sh drag gitdiscuss 300 0 --shot /tmp/d.png`: cliclick drag 796,306 → 1096,306 in 12 steps,
  4.0 s total. Mid-drag `/state`: `pressed=gitdiscuss members=jarvis,gitdiscuss,checks moved=true
  cursor=1096,306`. After release: jarvis 624→924, gitdiscuss 624→924, checks 984→1284, y
  unchanged (408/292/292), panel-demo untouched at 155,535, `drag: none`. Exactly +300 on every
  member.
- `/tmp/d.png` and `/tmp/d-after.png` (115,252 1469x349, Retina 2938x698): both show the three
  green casings at the moved position, the gitdiscuss/checks seam intact, the shaded Web panel at
  left. Not wallpaper.
- Persistence: `layout.toml` written with x = 924/924/1284 after the drag; the reverse drag
  (-300) put it back to 624/624/984; relaunching the app restored that pose exactly.
- Bonus for the next item: `probe.sh drag gitdiscuss 300 0 --escape` ended with the group at
  x=624/624/984, `drag: none`, so Escape does reach the window and cancels the drag.
- Idle CPU after the drags: 0.0 % (`ps` and `top`, 3 samples over 4 s).
- The builder's parity note matches what I measured; the collapsed 29 "attempted" lines are a fair
  cleanup. `layout.toml` restored to the original (diff clean); app quit.

## Must fix

- None.

## Should fix

- The next item (`--escape`) can be ticked from a single live run: it already returns the group to
  x=624. Record the mid-drag `/state` (moved=true before the Escape) as part of that tick so it is
  clear the drag had started, not just that nothing moved.
- Carried over: `/state` "drag feel" fields (band, pull, captured target, spring offset) arrive with
  the magnetic-field item; `light`/`tween` are still placeholders.

## Verdict

PARITY: the cliclick drag/screenshot path works end to end and the group moves exactly +300 pt as
one unit.
