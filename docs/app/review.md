# Review

## Item

Socket lights and eased glow (section 2), with the status caption while dragging (commit c3ef1ad).

## Verified

- Code against the prototype (`socketLight`, `proximity`, `glowTick`, `updateGlowTargets`):
  `glow.rs` eases `v += (t - v)(1 - exp(-dt/40))` with dt capped at 64 ms and a 0.002 snap, the
  same as `glowTick`. The first frame after the loop has been idle steps 16 ms, not a capped 64
  (`about_to_wait` seeds `now - FRAME`), which matches the prototype's `glowTime = now`. The light
  is a 12-wide rounded strip blurred at sigma 5 (three box passes) with a 4-wide `#d9ff92` core at
  63/70 height, both scaled by brightness, like `socketLight` and `joinGlow`. 86 tests pass.
- Live, release build, Jarvis at 200,300 and GitDiscuss dragged toward Jarvis's right socket,
  pointer held still 600 ms at each distance:
  - 110 pt: both facing ends 0.263 (formula 0.263), `approach`, caption "Socket nearby · move closer".
  - 60 pt: 0.530 (formula 0.529), same caption.
  - 25 pt: 1.0, `captured`, "Ready · release to join" in the brighter `#e4fca7` ink.
  - Only the facing ends light up. Jarvis's left and GitDiscuss's right stay at 0.
  - Pulled out to 206 pt: the target dropped to 0 at once and the caption cleared. The value fell
    1 → 0.67 → 0.45 → 0.29 → 0.19 at 16 ms frames (exp(−16/40) = 0.67 per frame), reaching 0 in
    about 250 ms. It eased out, with no cut.
  - Back at 60 / 25 pt: 0.530 / 1.0 again, so there is no lockout.
  - Release in the approach band: no join, lights 0.012 then 0. Release captured: joined at 560
    in one group, lights 0.67 → 0.44 → 0, then gone.
  - Escape while captured: restored to 603, lights faded to 0, caption cleared.
- Checks (no sockets) dragged past Jarvis: `free`, no caption, every light 0. Over dock-left:
  `hover_dock = dock-left` and no caption.
- Screenshots at 60 and 25 pt next to `proto-03` / `proto-04`: the strip sits over the inner
  end lens on both capsules, dim at 60 and clearly brighter at 25. It reads as the same light.
  The caption is a small dark pill centred under the dragged widget.
- Regressions, all fine: pair drag +100,+50 moved both members exactly; double-click
  shade/unshade; Checks into dock-left (shaded, packed at 109,1079) and back out; idle CPU 0.0 %
  over 10 s (CPU time unchanged at 0:02.21).

## Must fix

- None.

## Should fix

- Escape failed to cancel once in 5 tries. A drag started right after an earlier Escape-cancel
  stayed live, and the release at 109 pt left GitDiscuss there instead of restoring it. The next
  3 tries all worked. Like the dropped mouse-up in the last review, this looks like key/event
  delivery, not this item. Worth a check: is the pressed window key at the moment of the drag?
- Add a model test where a joined pair is dragged and the member nearest a third widget lights
  up (the facing-pair choice across group members). The live check only covered single widgets.
- The caption window stays (empty) at its last position after a drag. It's transparent and
  click-through, so nothing breaks. Hiding it when idle would keep it out of Mission Control and
  window lists.

## Verdict

PARITY. The brightness curve, both-socket lights, eased in/out, no lockout and the two-stage
caption match the prototype's numbers live, and idle CPU is still 0.
