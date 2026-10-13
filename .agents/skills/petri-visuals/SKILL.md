---
name: petri-visuals
description: >
  Inline node-inspector-style visuals for explaining Petri. Use whenever you
  explain, compare or explore how a creature's brain is wired, what a sensor
  feeds, how votes or actions are computed, or what a proposed mechanism would
  look like: show a show_widget visual (inputs, compute nodes, action sinks,
  weights, example ticks) instead of text or code-block wiring examples.
---

# Petri visuals

The user understands inspector-style visuals far better than text; text
examples of wiring do not land. When explaining, comparing or exploring a
mechanism, wiring, vote flow, sensor or proposal, show a visual first and keep
the reply short.

## Make one

1. Once per session, before the first widget, call the visualize `read_me`
   (module `interactive`) without mentioning it.
2. Write the spec: format in `spec.md`, full example in `viz/example.json`. Use
   real inspector labels and genome kinds. Mark proposed pieces with
   `"cat": "proposal"`, `"isNew": true` or a group box.
3. Optional check: `sh viz/preview.sh spec.json > page.html`, then open the page
   in a headless browser from a local server and take a screenshot.
4. Call `show_widget` with this template. Paste `viz/petri-viz.min.js` exactly
   as `cat` prints it (about 13,300 characters); never retype or edit it. In
   SPEC, write every `<` inside a string as `\u003c` so text cannot end the
   script.

```html
<h2 class="sr-only">One sentence saying what the visual shows.</h2>
<div id="pv"></div>
<script>PASTE viz/petri-viz.min.js HERE</script>
<script>PetriViz.mount("#pv", SPEC);</script>
```

## Habits that work

- One idea per view. Use several views for comparisons (today versus proposal).
- Give 1 to 3 example ticks with real-looking values so numbers visibly flow.
- Explanations go in the reply, not the widget: a caption and one note per tick.
- Reply shape follows the user's ADHD rules: first line says what to click.

## Changing the renderer

Edit `viz/petri-viz.js`, run `sh viz/build.sh` (writes the min file and prints
its size), check it with `viz/preview.sh`, then run
`npm run test -- petriVizDrift` in `frontend/`. That test fails when colors,
labels or categories drift from the inspector, when the minified copy is stale,
and when `crates/v3-core/src/runtime/cgp/eval.rs` changes: then review
`evalKind` and the test's math cases, and update its eval.rs fingerprint. Any file
change here also needs its sha256 updated under `petri-visuals/` in
`.agents/skills.lock.json`, or `make check` fails its skill provenance check.
