# Skin formats: Sonique and Winamp

## Sonique (`.sgf`)

Inspected from a real skin, *turtle soup II – razor* (archive.org item `turtlesoupII-razor`), and from strings in `Sonique.exe` 1.x.

An `.sgf` is one file. It's a small archive (magic `DAF`, a `LIST` table of named entries) that holds:

- **`/skin.ini`**: the skin's settings file, with these sections:
  - `[skin]`: author, title and URL.
  - `[sonique colors]`: about 30 named colours (playlist, progress bars, status bar, knobs, logo, the built-in visualiser).
  - `[misc values]`: sprite-strip frame counts (e.g. `MISC_NAV_VOLUME_KNOB_images=17` frames of a rotating knob), slider sizes, and how far the drawer slides (`MISC_DRAWER_DISTANCE_TOSLIDE=94`).
  - `[misc locations]`: the x/y of every control state in every mode (`NAV_PLAYON_x`, `PAUSEON_y`, `DRAWER_x`, …).
- **`/jpeg/*`**: one image per mode or part: `navigator` (the big navigation console), `midsonique` (the mid-sized mode), `smallstate` (the smallest mode), `extra`, `misc`, `volumeknob`, `smallknob`, plus a thumbnail and its mask.
- **`/rgn/<mode>/<control>`**: Windows region data for the three modes (`nav`, `mid`, `small`). `…/frame` is the window's own outline for that mode, which is how Sonique had non-rectangular windows. Every control (`play`, `next`, `volume`, `drawer`, the drawer's `dknobamp`/`dknobbal`/`dknobpitch` knobs, `songposring`) gets its own hit shape, not a rectangle.

Takeaways:
- Shape is data. Every mode has an outline shape, and every control has a hit shape.
- Modes are part of the skin format. One skin supplies all three form factors.
- The set of controls is fixed. A skin can move and redraw controls but can't add new ones.
- Animation is pre-rendered frames (a strip of knob positions), not code.

## Winamp 2.x classic (`.wsz`)

A `.wsz` is a renamed zip of BMP sprite sheets (`main.bmp`, `cbuttons.bmp`, `titlebar.bmp`, `eqmain.bmp`, `pledit.bmp`, `numbers.bmp`, `text.bmp`, …) at fixed coordinates, plus optional `region.txt` (polygon outlines for non-rectangular windows), `pledit.txt` (playlist colours) and `viscolor.txt`.

Takeaways:
- The layout is fixed. Every skin re-paints the same sprite positions.
- That is why any of the tens of thousands of skins works with the standard player.
- It's also why skins can't add controls or change layout. That limitation led to Winamp 3/5 "modern" skins, which were XML plus scripting and lost the simplicity.

## What this project wants

Both layers, which neither format had:
1. **A generic widget contract**: a fixed set of named parts (casing, display, grip, buttons, sockets), in the way of Winamp and Sonique. Any skin can style any widget that uses the contract.
2. **Author-defined parts and modes**: a widget can declare extra parts and form factors, like Sonique's nav/mid/small, each with its own outline and hit shapes. Skins can target those specifically.

A skin is one file (an archive like `.sgf`/`.wsz`) or one folder.
