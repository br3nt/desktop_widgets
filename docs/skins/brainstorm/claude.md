# Skin brainstorm: Claude

Stage 1: ideas only, nothing built. Five aesthetics, each with what makes it work, ten-plus join and shade ideas, morphing, angles, readability and a build plan. Then models for combining skins, then my top picks.

---

## 0. Ground rules I'm carrying into every skin

These come from the prototype's design record and the README. They act as filters on every idea below.

1. **Still frames must look finished.** The footprint goal is zero CPU when idle. Ambient motion (drifting motes, boiling felt, breathing glow) can only run during an event: a drag, a join, a hover, a notification. The resting skin has to be beautiful as a single frame. Any idea that needs a loop to look right is marked *(ambient)* and is opt-in.
2. **Ornament stays outside the useful centre.** The display rectangle is sacred. Chrome may frame it, light it and reflect onto its bezel, but never sits over its text.
3. **Glow is a promise.** The prototype's 120 px "near" and 40 px "ready" thresholds stay. Each skin needs its own *near* signal (something starts to happen) and *ready* signal (unambiguous: release now and it connects). Glow is only one way to say it; a needle appearing or runes lighting are others.
4. **The seam carries its release.** The prototype's narrow release grip worked because it lived in the seam. Every join style below names its release affordance, ideally something the aesthetic already has: a zipper pull, a knot, a bell, a keystone.
5. **Physical attachment is not data wiring.** Joining changes the casing, never the agents.

### A taxonomy, so the five joins per skin differ in kind

I sorted join ideas into kinds and made sure each skin draws from several:

| Kind | What happens | Seam afterwards |
|---|---|---|
| **Fuse** | Two casings become one (the current prototype) | None, or a decorative feature |
| **Bridge** | A third structure spans a gap; widgets stay apart | Open span, desktop visible through it |
| **Bind** | Material wraps or stitches across the seam | Visible binding |
| **Clasp** | A separate object pins the two together | One object sitting on the seam |
| **Interlock** | Edge shapes key into each other | Jigsaw-like or overlapping edge |
| **Hinge** | A pivot that also sets the angle | A joint you can rotate |
| **Grow** | Something living extends from one to the other | Organic, asymmetric seam |

And for shade modes:

| Kind | Motion | Strip |
|---|---|---|
| **Roll** | Content rolls up around an axis | A cylinder or scroll |
| **Fold** | Panels fold over each other | Folded edges or closed covers |
| **Sink** | Slides into a pocket, slot or floor | The lip of the container |
| **Distil** | The widget is replaced by a symbolic row (beads, lancets, stars) | A row of status glyphs |
| **Cinch** | Compresses or gathers | Gathered or coiled band |

---

## 1. Gothic cathedral / basilica

### What makes it gothic

- **Geometry from a compass.** Pointed arches (two arcs meeting), trefoils, quatrefoils, cusps, ogees, rose windows, blind arcading. Gothic ornament is ruler-and-compass geometry, which is good news: it's procedural by nature.
- **Verticality and bays.** A cathedral is a repeat of identical bays (pier, arch, window, vault). That maps onto widgets that grow by adding bays.
- **Light through glass.** The defining experience isn't stone, it's coloured light falling on stone. The displays are the windows. That's the key move for this skin: **the display is a lit window in a stone wall**.
- **Materials:** pale weathered limestone, darker mortar joints, lead came between glass pieces, iron strapwork on doors, touches of gilding on bosses and keystones.
- **Motifs:** crockets climbing gables, finials, pinnacles, gargoyle spouts, label stops (little carved heads at the end of a drip moulding), friezes of processing figures, diaper patterns, ribbed vaults.
- **Motion:** heavy, slow, settling. Stone grinds and seats with a small dust puff. Light shifts.

**Cheap version:** a grey rounded rectangle, blackletter font, clip-art gargoyle in the corner, purple gradient "stained glass".
**Overboard version:** every surface carved, tracery over the display, blackletter on data, sub-pixel crockets turning into grey fuzz, coloured glass behind the text so nothing can be read.

### Readability

- The display is a **clear quarry-glass pane**: small diamond leading at the very edge only, pale and frosted in light themes or deep night-blue glass with gilt text in dark themes. The stained colour lives in the tracery *around* the display (head of the arch, side lights), never behind text.
- Blackletter only for the carved nameplate, and even then a rounded, legible textura. Data in a clean serif or the existing mono.
- At small sizes tracery drops a level of detail: a rose window becomes a plain oculus with six petals, then a ring.

### Join styles

1. **Flying buttress (Bridge).** The widgets stay a gap apart. An arched buttress builds itself across: voussoirs drop in one by one from the springing on A to the landing on B, the keystone falls last and seats with a puff of dust. Seam: an open arch with the desktop visible through it. *Near:* the springing stone on each end lights at its joint. *Ready:* a ghost outline of the full arch appears. *Release:* pull the keystone up; the arch collapses stone by stone back into each widget.
2. **Half-rose window (Interlock).** Each widget's joinable end carries half a rose window. As they approach, each half rotates so its petals line up with the other's. *Near:* the halves start turning. *Ready:* the petals align and the glass brightens. Joined: one full rose window sits on the seam, backlit, and it is the join indicator. *Release:* grab the rose and turn it a sixth of a turn; the halves unlock. The rose is also the natural angle control (see angles).
3. **Compound pier (Fuse).** The two end walls dissolve into one clustered column. The column builds up in courses from the plinth, a statue niche forms in it, and a small figure (a lamp-bearer, abstract, faceless) steps into the niche holding the status lamp. Seam: a load-bearing pier, with one shared lamp.
4. **Lead came solder (Bind).** The two display windows become one window with a shared frame. A bright bead of molten solder runs down the lead line between them, glowing orange then cooling to grey. Each pane keeps its own content. *Release:* a thin knife drawn down the came; it cracks open.
5. **Nave doors (Clasp).** The gap is a pointed doorway with oak doors and iron strap hinges. Joining swings the doors shut and a heavy bar drops across them. *Release:* lift the bar. Quirky option: a portcullis instead, dropping with a chain rattle.
6. **Frieze procession (Fuse, decorative).** Both widgets carry a frieze band along the top. On join the band becomes continuous and a line of tiny carved figures walks across the seam from one widget to the other, then freezes back into stone. Once-only motion per join, no idle cost. Seam hidden under a carved boss.

### Shade modes

1. **Cornice.** The façade drops down behind its own top moulding. The strip is the cornice: crockets along the top edge, the nameplate carved in the frieze, one lancet slit showing the key value in lit glass.
2. **Clerestory (Distil).** The widget becomes a row of narrow lancet windows. Each lancet is one status: lit amber for running, cold blue for idle, red for needs-you. A strip of light; very readable as a status bar, no text needed. Hover a lancet for a tooltip.
3. **Triptych closing (Fold).** Gothic altarpieces were hinged: the painted wings closed over the centre and their backs were painted in grey monochrome (grisaille). Here the top and bottom wings fold over the display and their outsides show grisaille: the name and one value in grey-on-grey relief. The strip is the closed altarpiece. Historically grounded and unlike any other shade.
4. **Crypt (Sink).** The widget lowers into a stone floor slab. Only the tips of its arches and the glow of its windows remain above the floor line, like a building sunk to its window heads.
5. **Inscription drum.** The strip becomes a carved stone band that rotates like a drum to show successive lines (task, count, log), each line carved in capitals.

### Morphing

- **Build by bays.** Small = one lancet. Mid = triple lancet with a little rose above. Large = a nave elevation: arcade, triforium, clerestory, three display tiers. Morphing adds or removes bays, and the scaffolding moment is the animation: timber scaffolding appears, stones rise, scaffold falls away.
- **Window types as form factors:** lancet (tall), rose (round display), oculus (tiny round readout), Perpendicular window (wide grid for tables).

### Angled connections

- **Apse geometry.** Gothic apses are polygons: chevets with radiating chapels at fixed fractions of a circle. Sockets accept angles in steps of 22.5° or 30°, and the joint is a canted wall segment with its own narrow lancet. The user sets the angle; the canted wall widens or narrows to fit.
- **Rose as dial.** With the half-rose join, the rose window is the hinge. Turn it to set the angle; the petals act as the detents.
- **Fan vault** at the inside of tight angles: ribs fan out to fill the wedge.

### Build lightly

- Tracery from primitives: `pointedArch(span, rise)`, `foil(n, r)`, `rose(n, rings)`. Generate SVG path data once at load; reuse with `<use>`.
- Limestone: `feTurbulence` (fractal noise, low frequency) into `feDiffuseLighting` with a distant light gives a convincing weathered stone. Mortar joints as a dashed course pattern.
- Stained glass: a seeded Voronoi on a small grid gives the pieces; fill with a five-colour palette; lead came is a dark stroke. Light through glass: the glass layer uses `mix-blend-mode: screen` with a soft glow underneath.
- **Risks:** `feDiffuseLighting` over a large area is slow and it re-runs on any transform in some engines. Render stone once into an image (canvas `toDataURL` or `<filter>` on a hidden SVG snapshotted once) and reuse it as a fill. Tracery aliasing below 1 px strokes: level-of-detail switch by widget scale.

---

## 2. Felt

### What makes it felt

- **Fuzzy edges, not hard ones.** The silhouette has a halo of stray fibres. This is 80% of the look.
- **Puffiness.** Shapes are slightly stuffed: soft shading toward the edge, no specular highlights, matte everywhere.
- **Hand-cut irregularity.** Corners not quite matching, layered appliqué with each layer's edge visible and casting a soft short shadow.
- **Thread.** Blanket stitch around edges, running stitch as decoration, French knots as dots, cross-stitch.
- **Notions:** buttons with two or four holes, zippers, ribbon, pom-poms, safety pins, dressmaker's pins with round heads, sequins as the only shiny thing.
- **Light:** warm, soft, diffuse; a tabletop under a lamp.
- **Motion:** stop-motion. 12 frames per second, a little "boil" (shapes shift a pixel between frames) during movement. Squash and stretch.

**Cheap version:** noise texture over rounded rectangles, a jaunty rounded font, and that's it.
**Overboard version:** fuzz on the display text, stop-motion jitter on display content so text shakes, everything wobbling all the time.

### Readability

- The display is a **smooth patch**: a flat cream cotton or vinyl panel stitched into the felt, crisp edges, text printed on it like a fabric label. Fuzz stops at the stitch line.
- **Cross-stitch digits.** For big numbers ("03", "12 / 14") use a 5×7 cross-stitch matrix: each lit cell is an X stitch. Readable at large size, charming, and it's the skin's own LCD. Small text stays a normal font.
- Stop-motion steps apply to chrome only. Display content slides smoothly.

### Join styles

1. **Zipper (Interlock).** Each joinable edge has a zipper half sewn on. *Near:* the zip teeth lift and the pull tab swings into view. *Ready:* the teeth on both sides line up. Joining runs the pull along the seam and the teeth mesh. Seam: a zip. *Release:* the zipper pull *is* the release grip, the best fit for the prototype's release control of any idea in this document. Drag it back down and the sides part.
2. **Blanket stitch (Bind).** A needle appears with thread, darts in and out along the seam doing blanket stitch, then the thread pulls tight and the felt puckers slightly. Seam: visible stitches. *Release:* a pair of tiny scissors snips the end and the thread pulls out in one long strand.
3. **Button tab (Clasp).** A felt tab with a buttonhole flops over from A and buttons onto a big button on B. The tab overshoots, wobbles and settles. *Release:* tug the tab.
4. **Mitten hands (Grow, character).** Two small felt mitten hands come out of the ends and clasp, stop-motion. The handshake is the seam. Optional face stitches turn widgets into characters. Risky (cute can tip into childish) but memorable.
5. **Ribbon bow (Bind).** A ribbon threads through grommets on both edges, criss-cross like a shoelace, then ties itself into a bow at the top. *Release:* pull the bow tail.
6. **Velcro (Fuse).** Hook-and-loop strips; the widgets press together with a squash. On release, fibres stretch between them as they part, then snap back. The visible part of the effect happens on the way out.
7. **Appliqué patch (Clasp).** A patch is laid over the seam, pinned with three round-headed pins, then the pins vanish and stitching appears around the patch. The patch can carry a little badge for the group.

### Shade modes

1. **Bolster (Roll).** The widget rolls up into a stuffed tube and a ribbon ties round it. The strip is a stitched bolster with the name embroidered on it. Unrolling bounces.
2. **Pocket (Sink).** The widget slides down into a felt pocket; only the top edge and a woven name tag peek out, like a hanky in a breast pocket. The pocket's top edge carries one value in cross-stitch.
3. **Drawstring (Cinch).** The casing gathers into ruffles as if a drawstring was pulled; the strip is a gathered band with a toggle. Pull the toggle to open.
4. **Pleats (Fold).** Concertina folds, each fold edge visible as a stripe of slightly different felt colour. The stripes can be status colours.
5. **Woven label (Distil).** The strip is a clothing label: woven name tape with the widget name and value. The most minimal felt shade, good on glasses displays.

### Morphing

- **Stuffing.** Growing inflates the shape like a cushion being filled; shrinking deflates with wrinkles. Squash on landing.
- **Patches.** New panels are sewn on: a patch slides in, pins appear, stitches run round it.
- **Felt is flexible**, so silhouettes can change continuously: a capsule can bend into a crescent without breaking the illusion.

### Angled connections

- Fabric bends. The joint is a **cloth hinge**: any angle works, the inner corner shows crease folds, the outer corner stretches slightly. The crease count scales with the angle.
- A **safety pin** as a visible pivot is a cute explicit-angle alternative.

### Build lightly

- Fuzz: `feTurbulence` (high frequency) into `feDisplacementMap` at a small scale on the shape's alpha gives ragged edges; a second turbulence pass thresholded with `feComponentTransfer` gives a fibre halo.
- Puffiness: blur the alpha (`feGaussianBlur`), feed it as a height map to `feDiffuseLighting`. The classic "pillow emboss".
- Felt surface: low-contrast, very high-frequency noise multiplied over the fill.
- Stitches: `stroke-dasharray` on an inset path; blanket stitch is a dashed path plus short perpendicular ticks generated along it.
- Stop-motion: `steps()` easing on chrome transforms, plus swapping the turbulence `seed` among three values each frame during motion.
- **Risks:** changing `seed` re-runs the whole filter each frame; fine for two widgets, heavy for fifty. Only boil the widget being moved. Lighting filters at high DPI cost real time; cache the puffed casing as an image per size.

---

## 3. Elven and intricate

### What makes it elven

- **Thin, long, tapering.** Hairline strokes that swell and taper like a pen stroke. Tall narrow proportions. Points at the ends of everything.
- **Growth logic.** Ornament grows like plants: spirals, tendrils, whiplash curves (art nouveau), leaves along a vein. Asymmetric but balanced.
- **Materials:** white gold and silver, moonstone, mother-of-pearl, pale translucent glass, frosted leaf.
- **Light:** cool, from within; edges catch moonlight. Faint glow, never neon.
- **Motifs:** leaves, stars, crescent moons, interlace, branching, feather veins, calligraphic script.
- **Motion:** slow, unfurling, weightless. Things grow rather than slide.

**Cheap version:** Celtic knot border in green and gold, Papyrus font, sparkle.
**Overboard version:** 0.5 px hairlines that disappear on standard displays, constant glitter particles, script on data, ornament wrapping the display.

### Readability

- Display: **moonstone pane**, a pale milky surface with dark grey ink; or night mode, a deep blue-violet pane with silver text. A clean humanist sans for data; calligraphy only for nameplates.
- Minimum stroke: 1 device pixel for hairlines, doubled on 1× displays. Hairline ornament never runs parallel to text closer than 6 px or it reads as an underline.

### Join styles

1. **Vine (Grow).** Tendrils grow from each end, reach across, find each other and spiral around one another, then leaf out. *Near:* a tendril tip starts unfurling toward the other widget. *Ready:* the tips touch. Seam: a braided living stem with a few leaves. *Release:* drag apart and the tendrils unwind and retract.
2. **Moonlight bridge (Bridge).** A thin arched span of light draws itself across the gap; when joined it solidifies into silver filigree. The widgets float apart with a delicate bridge between them. This one makes "joined but not touching" feel natural.
3. **Weave (Bind).** The hairline border of each widget unravels at the joinable end into separate threads, and the threads re-weave into one interlace knot at the seam. Release unweaves.
4. **Leaf brooch (Clasp).** A single leaf-shaped cloak pin descends and pins the two together, set slightly at an angle. One beautiful object, very calm. The brooch is the release: lift it off.
5. **Runic seal (Interlock).** Lines of runes (invented glyphs, not a real script) run along each edge. On approach they light up, and when aligned they match like a lock. Joined: the runes fade to faint and return only on hover. *Near* and *ready* map perfectly onto runes lighting then matching.
6. **Quicksilver (Fuse).** Edges become liquid silver, meet with surface tension, ripple and seal into one rim.

### Shade modes

1. **Leaf furl (Roll).** The widget curls lengthwise into a slender leaf. The leaf's central vein is a progress line, the leaf tip shows the count.
2. **Constellation (Distil).** The widget collapses into a thin silver line with a few star points. Each star is a status; brightness shows level. Expanding draws the constellation lines back out into a casing. The most delicate shade I have, and ideal for glasses.
3. **Wing fold (Fold).** Translucent panels fold back along their veins like a dragonfly's wings, leaving the slim body as the strip.
4. **Lantern sleep (Sink).** The casing dims until only its lit rim remains, a glowing outline with the nameplate in script. Hover wakes it.
5. **Mist (Distil).** The casing dissolves into mist and recondenses as one line of text in a frosted band. *(Mist motion only during the transition.)*

### Morphing

- **Growth.** Bigger forms grow: a branch extends, a new leaf opens with a new display on it. Shrinking is autumn: leaves fold and drop back into the branch.
- Silhouettes follow plant forms: leaf (mid), seed pod (small), branch with leaves (large, several displays).

### Angled connections

- **Branching.** An angled join reads as a fork in a branch. Any angle looks natural because branches fork at all angles; the knot at the fork reforms to the angle. Optional preference for "growth angles" (about 137.5°, the golden angle) as soft detents, a nice nerdy touch.

### Build lightly

- Filigree: log spirals and clothoids generated in JS, emitted as SVG paths. Tapered strokes: draw as filled outline shapes (offset the path both ways with a width function) rather than strokes.
- Growth: animate `stroke-dashoffset` along the vine path, then scale leaves in from 0 at their attachment points.
- Leaves: one leaf symbol, placed with `<use>` and transforms.
- Mother-of-pearl: `conic-gradient` with low saturation pastels at low opacity over white.
- **Risks:** tapered outlines need path offsetting (a small amount of code; do it once at load). Many thin elements at 1× DPI go grey; level-of-detail drops spiral turns at small sizes. Dashoffset animation on dozens of paths at once is fine; on hundreds it isn't.

---

## 4. Wire, beads and stones

### What the reference photos show

- **Ref 3, pendant:** oxidised copper wire, tight coil wraps forming the bezel border, spiral scrolls at the top, tree-of-life wire lines over a malachite cabochon. The dark recesses with bright highlights on the top of each wire turn are what read as "wire".
- **Ref 3/4, hoop trees:** a ring frame; a trunk of many parallel wires twisted into a rope, fanning at the bottom into roots, each root wrapped onto the hoop with a few tight coils; branches splitting into twigs ending in single beads (peridot chips, multicolour glass).
- **Ref 3, gem trees:** a wire tree on a stone base, beads clustered at twig ends.
- **Ref 5, bugs:** chunky silver wire legs and antennae, seed bead bodies in bright colours, can-tab wings, googly eyes. Playful, toy-like.
- **Ref 6, caged beads:** a spiral of copper wire forming a cage around a bead, loops at each end.
- **Ref 7, strands:** stones of mixed shapes (rondelles, ovals, nuggets, heishi discs, pearls) with small gold seed beads and metal bead caps as spacers.
- **Ref 8, nest:** a wild tangle of fine dark wire, a lapis cabochon held by a coil, metal roses, feather charms, a curl at the end.

### What makes it wire

- **Every line is a wire with volume:** dark edge, mid body, bright highlight along the top.
- **Wraps hold everything together.** Nothing is glued; everything is coiled, twisted, looped or caged.
- **Stones and beads are the colour.** The metal is neutral (copper, oxidised copper, silver); the stones carry the palette.
- **Ends curl.** Wire ends are tucked or scrolled.
- **Motion:** springy, tinkling. Beads slide on wire and click; wire flexes and settles.

**Cheap version:** grey stroked rectangle with some coloured circles.
**Overboard version:** a nest of hundreds of wires over the display; beads jiggling all the time; every edge coiled so the casing is all texture and no shape.

### Readability

- The display is a **cabochon or polished slab**: a dark stone (obsidian, labradorite, deep malachite) held by a coiled bezel, with light text. The stone's sheen is a soft highlight confined to one corner, outside the text area.
- **Bead meters.** Bar charts as bead strands: the Jarvis activity bars become short strands of beads, the count of lit beads is the value. Status as bead colour.
- Wire chrome stays at the rim and corners. A nest is allowed as ornament on one corner, never across the face.

### Join styles

1. **Coil wrap (Bind).** A wire end leaps from A to B's frame and wraps around it in a tight helix, three times, like the roots wrapped onto the hoop in ref 4. Three short coiled bands mark the seam. *Near:* wire ends lift and point toward each other. *Ready:* they touch. *Release:* grab a band and unwind.
2. **Twisted trunk (Fuse).** Several parallel wires leave each end, meet in the middle and twist together into a rope as the widgets close in, like the tree trunk in ref 4. The twist rotates as it tightens. Seam: a twisted cable that frays into each widget's frame.
3. **Jump ring (Hinge).** Each widget has a loop on its end. A jump ring opens, swings through both loops and snaps shut. The widgets hang from each other and *can pivot*. This is the most natural angle mechanism in any of the five skins (see angles). *Release:* twist the ring open.
4. **Bead strand bridge (Bridge).** A strand of beads with gold spacers strings across the gap, beads sliding along the wire one at a time and clicking into place like an abacus. The strand can carry information: its beads show the shared queue count or the group's status colours. Seam: a short necklace.
5. **Caged bead (Clasp).** A bead at the seam with a spiral cage forming around it from both sides (ref 6). It's also a ball joint: rotate the bead to change the angle.
6. **Nest (Grow).** Wild fine wires grow from both edges and tangle into each other (ref 8), a small cabochon or wire rose forms at the centre. *Release:* pull apart and strands stretch, then snap back with a twang.
7. **Overlapping hoops (Interlock).** Hoop-framed widgets join by overlapping their rings into two linked circles; the shared lens shape between them is a tiny bonus display or the status lamp.

### Shade modes

1. **Bead strand (Distil).** The widget collapses into a necklace: each bead is a status, each stone type a widget or state (amber running, labradorite idle, carnelian needs-you), with a flat tag bead carrying the name. Beads slide in along the wire as the casing collapses. Straight from ref 7, readable at a glance.
2. **Coil band (Cinch).** The casing compresses vertically into the tightly coiled band from the top of ref 3's pendant; a flat stamped plaque in the middle shows the name and one value.
3. **Hoop rim (Sink).** For hoop widgets: the tree retracts into the rim, branches folding down, beads gathering at the trunk; the strip is an arc of the rim with its wraps and one bead lit. For capsules, the frame wire alone stays, outlining an empty space with a single bead readout.
4. **Caterpillar (Distil, character).** The shade is a beaded caterpillar (ref 5) sitting on a wire: its body segments are statuses, its head nods when something changes. Optional: a skin variant for people who want personality.
5. **Folded branches (Fold).** The wire tree folds its branches down along the trunk like an umbrella closing; the strip is the trunk with the beads lined along it.

### Morphing

- **Tree growth.** Small = a single cabochon in a bezel. Mid = a branch with two displays. Large = a hoop tree where each canopy cluster is a display region. Growing extends branches and beads slide out to the new tips.
- **Bezel resize.** The coil bezel loosens (coils spread), the stone grows, the coils tighten again.

### Angled connections

- **Jump rings and caged beads are real pivots.** Any angle is legitimate because jewellery hangs and swings. The user drags to set the angle; the ring turns in the loops; detents are optional.
- Bent-wire joints: the connecting wire bends with a visible bend radius and a small coil at the bend to "lock" it.

### Build lightly

- **Round wire:** stroke the same path three times: wide dark, narrower mid, thin highlight offset 0.6 px toward the light. Reads as a cylinder without any gradient along the path.
- **Coils:** sample the path with `getPointAtLength` at load, emit a short curved stroke perpendicular at each step. Combine all coils into one `path d` per layer so a hundred coils are one element.
- **Twists:** two or three sinusoids with phase offsets along the path.
- **Beads:** circles with a radial gradient and a tiny highlight; faceted chips as small polygons with per-face flat shading; pearls with a soft gradient and a pink-green sheen.
- **Patina:** oxidised copper base `#3b2a22`, highlights `#d99a6c`; silver base `#5b6066`, highlights `#f2f4f6`.
- **Risks:** generating coils and nests costs at build time; cache the generated paths per size. The nest needs a seeded generator so it's stable across reloads. Many beads with gradients is fine in SVG up to a few hundred.

---

## 5. Hollow Knight and Silksong (the mood, not the art)

What to take: ink line work with thick-to-thin brush outlines; a muted palette of blue-greys and deep navies with one warm accent (chalky white in the first game, red and gold in Silksong); deep layered fog and parallax; a quiet, melancholy, underground civilisation; ironwork, lanterns, shells, bells, thread and needles; a cartographer's quill map. What not to take: characters, masks, the soul vessel, the specific HUD shapes, any recognisable glyph or location. The repo is public.

### What makes the mood

- **Silhouette first.** Dark shapes against lighter fog. Shapes are readable as silhouettes alone.
- **Brush outlines.** Thick at the base of a curve, thin at its tip; slightly uneven; tips that curl.
- **Carapace and ironwork.** Segmented shell plates; wrought-iron curls and spires; small lamps.
- **Thread, needle, bells** (the Silksong side): red thread, a pale needle, brass bells, pilgrims.
- **Light:** pale, sparse, cold; a single warm lamp; chalky white highlights.
- **Motion:** quick, precise, with weight; a dart, a pause, a settle. Small white motes *(ambient, opt-in)*.

**Cheap version:** dark blue gradient, white text, a bug icon.
**Overboard version:** copied HUD shapes, fog drifting over the display, everything dim and low-contrast so data is lost.

### Readability

- **Cartographer's sheet.** The display is a sheet of parchment pinned to the casing: dark ink text on cream. High contrast, distinctive, and true to the mood without copying anything.
- Night alternative: chalk-white text on deep navy, with ink-blue secondary text no darker than 4.5:1 contrast.
- Fog and vignette live behind the casing, never in front of the display.

### Join styles

1. **Silk stitch (Bind).** A pale needle darts across trailing red thread, loops through eyelets on both edges in two quick passes and pulls taut with a small bounce. Seam: taut criss-crossed threads. *Release:* a quick slash (drag across the threads); they part with a white flash.
2. **Carapace overlap (Interlock).** The joinable edges are shell plates. One widget's edge plate slides over the other's like insect segments telescoping, with a click. Seam: overlapping shell edges with a dark gap line.
3. **Bell and chain (Clasp).** A short chain with a small bell links eyelets on both. Joining rings the bell: a few concentric rings pulse out once. *Release:* tap the bell twice, or pull the chain.
4. **Ink bleed (Fuse).** The two outlines touch and the ink flows together; a splatter flicks out, then pulls back into a clean shared outline.
5. **Lamppost (Bridge).** A wrought-iron lamppost rises between the widgets with curling brackets reaching to each; its lantern is the join light. Generic ironwork, not any in-game structure.
6. **Mycelium (Grow).** Pale fungal threads reach across the gap and knit together; slow and quiet, the opposite of the needle.

### Shade modes

1. **Curl (Roll).** The widget rolls up like a pill bug into an armoured band of segments; status shows as light leaking from the gaps between segments.
2. **Cocoon (Cinch).** Thread wraps around the widget, spinning it into a slim cocoon; the strip is the wrapped band with one lit seam. Hover loosens the wrap.
3. **Hanging sign (Distil).** The widget becomes a small signboard hanging on two chains from an iron bracket, the name and one value painted on it. Swings once when it changes.
4. **Fog sink (Sink).** The widget sinks behind a bank of fog, only its spire tips and lantern glow showing.
5. **Map roll (Roll).** The parchment display rolls up into a scroll tied with thread, a quill-written label on the outside.

### Morphing

- **Map unfurl.** Growing unrolls more of the parchment, revealing more "regions" (pages, panels). Shrinking rolls it back.
- **Moult.** A bigger form sheds its shell: the old casing cracks along a seam and the larger one steps out. Once per resize; memorable.

### Angled connections

- **Hanging joints.** Chains and brackets: angles look natural because things hang. An iron bracket with a curl sets the angle; the bell or lamp hangs plumb whatever the angle.
- **Shell articulation:** segments overlap more on the inside of the angle, like a bent abdomen.

### Build lightly

- Brush outlines: filled tapered shapes (same offset trick as elven), plus low-frequency `feTurbulence` displacement at a small scale for hand wobble. Apply wobble once and cache.
- Fog: two or three large radial-gradient layers behind the stage, moved with composited CSS transforms only during interaction.
- Parchment: warm base, low-contrast noise, darker blotchy edges via a radial mask.
- Thread: thin red stroke with a 1 px lighter highlight.
- **Risks:** the look relies on darkness and fog, which fights readability; keep display contrast measured. Ambient motes break zero-idle-CPU; they run only during transitions. The style can drift toward copying: review for any recognisable silhouette before shipping.

---

## 6. Combining skins

Can a felt widget dock with a gothic one? Can one skin's material wear another's ornament? Five models, with examples and what breaks.

### Model A: half-couplers (each skin draws its own half)

Every socket has a standard **mating face**: a fixed size, position and angle range. Each skin supplies a *half-coupler* drawn in its own style, ending in that face. Two widgets in different skins join coupler to coupler, like plugging a European plug into a travel adapter.

- **Gothic + felt:** a carved stone corbel on the gothic side ends in a round boss; a felt tab on the felt side buttons onto the boss.
- **Wire + elven:** a coil wrap on the wire side grips the elven widget's silver tendril, which happens to be the same diameter.
- **Hollow + gothic:** the gothic buttress lands on a wrought-iron bracket on the Hollow side.
- **What breaks:** the mating face is a lowest common denominator, so every cross-skin seam looks like a plug. Fusing (shared casing) is impossible across skins; only Bridge, Clasp and Hinge kinds work.
- **What it buys:** works for any pair with no pairwise authoring. A good default.

### Model B: layer stack (material, structure, ornament, palette, motion, display)

A skin isn't one thing; it's a stack of layers that can be picked separately:

| Layer | Decides |
|---|---|
| Structure | Silhouette, bays, where displays sit |
| Material | Surface rendering (stone, felt, wire, silver, ink) |
| Ornament | Motifs and decoration (tracery, stitches, filigree, coils) |
| Palette | Colours |
| Motion | Easing and character (heavy, stop-motion, growing, springy, darting) |
| Display | Screen treatment (quarry glass, cotton patch, moonstone, cabochon, parchment) |

Examples:
- **Felt cathedral.** Gothic structure and ornament, felt material, stop-motion motion. This is a church kneeler (a hassock): plush rose windows in appliqué, stitched tracery, a felt flying buttress. I think it's lovely and it's the strongest argument for this model.
- **Silver filigree.** Wire material with elven ornament: the elven vine join done in round silver wire with moonstone beads. The two aesthetics are cousins; it barely counts as mixing.
- **Ruined bug cathedral.** Gothic structure, Hollow palette and brush-line material. Rose windows as dim lit shells.
- **Felt bugs.** Wire structure (the bead caterpillar shade, jump rings) in felt material: a felt caterpillar with button eyes.
- **What breaks:** layers have hidden dependencies. Stained glass needs translucency, felt has none, so the felt cathedral needs a fallback (appliqué glass panels in bright felt with a stitched lead line). Stop-motion motion on elven growth looks broken, not stylised. So each layer must declare *requires* and *provides* (e.g. ornament "stained glass" requires material capability "translucent") and the builder offers only valid stacks or a declared fallback. Also, joins and shades belong to *which* layer? I'd put them in Structure with Material providing the rendering, so a felt cathedral's flying buttress is built from felt voussoirs that flop into place.

### Model C: the group decides (dominant skin)

A docked group has one skin. When a felt widget joins a gothic group, it is re-rendered in gothic for as long as it's docked. The widget keeps its identity through its accent colour and its nameplate glyph only.

- **Example:** drag a felt Checks widget into a gothic Jarvis's drawer bay; as it seats, a ripple passes over it and it turns to stone, with its felt colour kept as the colour of its glass.
- **What breaks:** a widget author's custom skin disappears whenever it's docked, and authors will hate that. Which skin wins when two groups merge? (Proposal: the group being dropped onto.) Undock must restore the original instantly.
- **What it buys:** groups always look coherent. Good for people who want order.

### Model D: pairwise seams (negotiated)

Both skins submit their join style; for known pairs there's an authored hybrid seam that uses both vocabularies.

- **Wire + gothic:** the wire coil-wraps around a gothic pier, the way ivy wire wraps a column.
- **Hollow + felt:** the Hollow needle stitches through the felt with red thread, a blanket stitch in Silksong colours.
- **Elven + gothic:** the vine grows up the tracery.
- **What breaks:** N skins means N² pairs. Only the popular ones get authored; everything else falls back to Model A. Fine as a layer on top of A, never as the only model.

### Model E: containment, not joining

Cross-skin widgets don't join edge to edge; one *holds* the other, like the prototype's drawer bay. Different materials are expected inside a container (a felt mouse in a stone niche, a wire bug in a felt pocket).

- **Gothic reliquary bay** holds a felt widget behind glass, and that looks deliberate.
- **Felt pocket** holds a wire widget; the wire tag hangs over the pocket edge.
- **Wire hoop** holds any widget at its centre as if it were the cabochon.
- **What breaks:** only works where a container exists; doesn't help two peers sit side by side.

### Model F: material bleed (experimental)

At the seam, the two materials blend over a short zone: stone turns to felt across 20 px via a noise-driven mask, like moss growing over stone or frost creeping. Weird and possibly wonderful as a one-off; most likely reads as a rendering bug. Worth one experiment on the combined page and no more.

### Recommendation

**A as the guaranteed baseline, B as the authoring model, D for a few showcase pairs, E for cross-skin containers.** C as a user preference ("make my groups match").

---

## 7. Top picks

What I most want to build first, and why.

1. **Wire: jump-ring hinge + bead-strand shade + cabochon display.** The most original of the five and the most directly grounded in Brent's references. The jump ring solves angled connections better than any engineered socket: jewellery pivots, so any angle looks right. The bead-strand shade turns status into stones, readable without text. The cabochon display gives a dark polished screen that suits the existing light-on-dark text.
2. **Felt: zipper join + pocket shade + cross-stitch digits.** The zipper pull is the prototype's release grip made physical: drag it up to join, down to release. The pocket shade is funny and clear. Cross-stitch digits are a native LCD for the skin. The fuzz and puffiness filters are the main technical test of the whole exploration.
3. **Gothic: half-rose join + clerestory shade + triptych shade.** The rose window that aligns as you approach is a natural near/ready indicator and a rotate-to-set angle dial. The clerestory is a status bar made of light. The triptych is grounded in real altarpiece practice and I haven't seen it in a UI.
4. **Hollow mood: silk stitch join + cocoon shade + parchment display.** The needle-and-thread join has the quickest, most satisfying motion of any idea here; the parchment display solves the dark-mood readability problem.
5. **Elven: vine join + constellation shade.** The constellation is the best shade for glasses and VR in this document: a line with a few stars. The vine is the clearest "grow" join.
6. **Combined: the felt cathedral.** Gothic structure in felt material, to prove the layer-stack model, plus one half-coupler join (stone boss and felt button tab) to prove the cross-skin baseline.

Ideas I'd hold back: the mitten hands and the bead caterpillar (cute, but they risk turning every widget into a character; keep as variants), and material bleed (likely to look like a bug).

### Shared engine hooks these imply

Every skin above can be driven by the same few inputs, which keeps the five pages comparable and hints at the skin contract:

- `proximity`: idle, near (0..1), ready.
- `join(progress 0..1)` with named phases (prepare, span, seat), so stone voussoirs, zipper teeth and needle passes all key off the same timeline.
- `shade(progress 0..1)`.
- `angle(degrees)` with the socket's allowed range and optional detents.
- `morph(fromSize, toSize, progress)`.
- Level of detail by rendered scale, so ornament simplifies before it turns to grey fuzz.
