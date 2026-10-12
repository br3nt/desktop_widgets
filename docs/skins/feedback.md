# Brent's feedback on the skin study

Agents read this at the start of every run. It overrides your own preferences.

- Don't pick one join or shade style. Build options one at a time; this is an experiment.

## Brent, 13 Oct: the casings are still one shape in different colours

All three agents still use the same casing: the original rounded capsule with a dark screen and two round buttons in the bottom-right corner. Only the colour, texture and edge trim change. That is not a skin.

- **The skin changes the chrome.** The screen area may stay the same shape. Everything around it should change: the outline and silhouette, proportions, where the grips are, what and where the buttons are, how labels are carried. Buttons can move, change shape, or become something else (a lever, a bead, a bell pull, a wax seal), as long as they do the same job.
- **Mechanisms should feel different.** A felt join should feel soft and sewn; a gothic one heavy and masonry; a wire one twisted and springy. Different motion, different timing, different sound-like feel.
- **Wire, beads and stones must look like the reference photos** in `refs/` (open them and look before building). Wire wrapped around and through things, wire trees of life inside a hoop, beaded wire bugs and butterflies, beads caged in wire, strands of stones and pearls, wild nest-like wire around a cabochon. None of the current wire pages looks anything like them: they are a capsule with a thin copper outline.
- **Stone and stained glass must look realistic,** not flat colour fills. Stone needs real masonry: blocks with joints, weathering, chisel texture, light and shadow, carved depth. Stained glass needs lead came between pieces, light coming through, colour variation inside each piece.
- **Quality bar:** Claude's best pages are the minimum standard, and they now need to go further too (above).

What to do: before building any new item, rework every page you have so its casing follows this note, keeping its existing options working. Do it one page per run, starting with wire and gothic. Each page's rework is one checklist item (already added at the top of your checklist).

## Orchestrator notes (Claude, from checking screenshots)

- A recoloured copy of the original capsule with themed labels is not a skin. The casing's silhouette, material, surface detail and ornament must change so the aesthetic is unmistakable at a glance, at real size, before reading any text. The original capsule shape is not sacred: change the outline, proportions, grips and bezels freely. Only the widgets' content and the four states are fixed.
- The join or shade option you are building must be clearly visible and animated when its state is selected. Check it, don't assume it.
- **Codex (13 Oct):** your pages are tidy and well written, and the casing is still the plain capsule with a gold rim on every page; the gothic arcade is the closest to a real silhouette. The lower half of each page is essay text; keep it to a few lines and spend the effort on the casing. The four state tabs and the option drop-down are fine.
- **agy (13 Oct):** your design is the weakest of the three. The casings are the generic hull with a coloured rim and themed labels; the descriptions promise mechanisms ("spring-coiled wire cage", "masonry rotunda", "accordion bellows gusset") that the drawing does not show. Build what you describe, visibly, or don't describe it. Fewer, finished options beat many described ones. Your labels are tiny and the controls bar is crowded: fix readability.
- **Claude (13 Oct):** your pages are the strongest, but every casing is still the capsule. The button walls (60+ option buttons at the top) bury the widgets: group them into a compact drop-down or collapsible panel. The gothic page renders broken in the joined state in a headless check (widgets collapse into the top-left corner): verify and fix.
- **agy:** your gothic, felt and elven pages are the original capsule recoloured with themed labels. Before building any new item, rework those three pages so each casing is unmistakably its material and style (the first note above), keeping their existing options working. Treat that rework as your next checklist item.
- Shade strips must stay readable: the widget name and its one key value must be legible at real size, never clipped, overlapping or squashed. (agy's gothic portcullis shade currently garbles its text; fix it as part of your next item.)
- Brent views these on his phone. The published copy renders every page at 1300px wide scaled to fit the phone, so don't spend items on phone layouts. Do make sure dragging works with touch (pointer events, `touch-action: none` on draggable parts).
- When a widget grows (morph), it must not cover its neighbours. Grow into free space, or nudge neighbours aside and back. (Claude's gothic "Nave" currently grows over GitDiscuss.)
