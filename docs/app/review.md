# Review

## Item

FIX (reviewer): Run `probe.sh drag gitdiscuss 300 0 --escape` live and confirm the group returns to
its start pose (x=624 ±1) (commit 5020b44, docs only).

## Verified

- Screen unlocked, release build current, my own binary was the only `desktop-widgets` running.
- `probe.sh drag gitdiscuss 300 0 --escape --shot /tmp/rv-esc.png`: mid-drag `moved=true`,
  `members=jarvis,gitdiscuss,checks`, cursor 1096,306, x = 924/924/1284. After Escape + release:
  624/624/984, y unchanged (408/292/292), panel-demo untouched at 155,535, `drag: none`.
- Escape with the button still held (`--no-release`, then `cliclick kp:esc`): `/state` right after
  Escape already shows 624/624/984 and `drag: none`, before any release. Moving the pointer on to
  1296,406 with the button still down moved nothing; the later release moved nothing either. That
  matches the prototype's `cancelDrag()` (line 325): `stopDrag(); restore(saved); light=0`, an
  instant restore, and the drag is over, so later pointer moves are ignored.
- Screenshots: mid-drag frame shows the three casings at +300 with the gitdiscuss/checks seam
  intact; the held-after-Escape frame shows them back at the start pose. Not wallpaper.
- `layout.toml` byte-identical after the cancelled drags (cancel not persisted).
- Regression: plain drag +200 then -200 moved all three members exactly and returned them;
  `layout.toml` back to the original. Idle CPU 0.0 % (`ps` twice, `top` over 2 s).
- The builder's parity note matches what I measured.

## Must fix

- None.

## Should fix

- `probe.sh --escape` reads `/state` only after the release, so it cannot tell "Escape cancelled"
  from "release dropped it at the start". Add a `-- after escape` summary between `kp:esc` and
  `du:` so the probe shows the cancel before the button comes up.
- The prototype says "Move cancelled. The original assembly and contents are restored." on cancel;
  the app has no feedback line yet. Carry this to whichever item adds the feedback/status text.

## Verdict

PARITY: Escape reaches the pressed window, restores the exact pre-drag pose instantly while the
button is still down, ends the drag, and does not persist.
