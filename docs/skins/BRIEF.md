# Skin exploration brief

Three agents (Claude, Codex, agy) each explore on their own branch. The goal is **variety Brent can react to**, not one answer. He will say what he likes and doesn't; nothing here is decided.

## The five aesthetics (Brent's words)

1. **Gothic cathedral / basilica.** That intricate stonework, stained glass, statues and friezes. Not necessarily those exact things, but that style, design, ornateness and aesthetic.
2. **Felt.** The style where things are made in 3D felt: there are animations and images of 3D felt people and scenes. Make the UI look felt.
3. **Elven and intricate.** Thin, mystical elegance.
4. **Wire, beads and stones.** Wire-wrapped jewellery, wire gem trees and trees of life in a hoop, beaded wire figures (bugs, butterflies), caged beads, stone and pearl bead strands, wild nest-like wire with a cabochon stone. Reference photos are in `refs/` (not committed).
5. **Hollow Knight and Silksong.** Their mood and drawing style. Evoke it, but don't copy Team Cherry's characters or artwork; the repo is public.

## What to explore (open-ended, the more varied the better)

- **Many join styles per aesthetic.** How two widgets connect, the animation of connecting, and what the seam looks like once joined. Aim for several that are genuinely different from each other, not variations on one idea.
- **Many shade modes per aesthetic.** How a widget collapses to a thin strip (like Winamp's shade mode) and back.
- **Morphing**: widgets changing size and form factor.
- **Combining skins.** Can widgets in different skins dock together? Can one skin's material be combined with another's ornament? Try models, show examples.
- **Angled connections.** Sockets can accept a range of angles that the user can set.

Avoid both failure modes: boring and generic, and so overboard it stops being usable. Displays must stay readable.

## How you work: one option at a time, never pick a winner

This is an experiment. Don't choose one join or shade style; Brent wants to see the options and decide.

- Turn every idea in your brainstorm (joins, shades, morphs, angles, skin-combining) into a checklist in `docs/skins/progress/<agent>.md`, interleaving the five aesthetics.
- Each run, build exactly **one** unchecked item, add it to the right page as a new selectable option, make it work and look finished, tick it with a one-line note, and stop.
- Never remove or replace an earlier option. Pages grow.
- Each finished item is merged to `main`, which Brent is using live, so every run must leave your pages working.
- Read `docs/skins/feedback.md` at the start of every run. It holds Brent's likes and dislikes and overrides your own ideas.

## Mechanics (the only fixed rules, so results can be compared)

- Start from `prototypes/capsule-study.html` and use its widgets: Jarvis, GitDiscuss and the Checks widget, with the same content.
- Every skin page must show these states, switchable on the page with buttons or keys: **separate**, **joining** (mid-animation), **joined**, **shade**. Also show your alternative join styles and shade modes, switchable on the page.
- Self-contained HTML, CSS, SVG and JS. No external images, fonts or libraries; procedural textures and inline SVG are fine. Stay light: the project's goal is a tiny footprint.
- Output layout on your branch:
  - `docs/skins/brainstorm/<agent>.md`: ideas.
  - `prototypes/skins/<agent>/<skin>.html`: one page per aesthetic.
  - `prototypes/skins/<agent>/index.html`: links to your pages.
  - `prototypes/skins/<agent>/combined.html`: skin-combining experiments.
  - `<agent>` is `claude`, `codex` or `agy`. Skin file names: `gothic`, `felt`, `elven`, `wire`, `hollow`.
- Never use the word "relay" anywhere.
- Don't run git commands. The orchestrator commits for you.
