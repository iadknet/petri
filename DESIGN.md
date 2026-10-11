---
name: Petri long-term record
description: A field record on a dark slate sheet, where three plots are censused after every closure.
colors:
  sheet: "#14181c"
  sheet-2: "#191e23"
  sheet-3: "#20262c"
  rule: "#2a3138"
  rule-2: "#3a434c"
  ink: "#ebe7dd"
  ink-2: "#b9b5aa"
  muted: "#8a877d"
  plot-11-orchards: "#199e70"
  plot-22-canyon: "#d95926"
  plot-33-confluence: "#3987e5"
  accent-amber: "#e8b44c"
  good: "#4cc46a"
  bad: "#ef6b6b"
typography:
  display:
    fontFamily: "Geist Sans, ui-sans-serif, system-ui, sans-serif"
    fontSize: "26px"
    fontWeight: 600
    lineHeight: 1.1
    letterSpacing: "-0.02em"
  headline:
    fontFamily: "Geist Sans, ui-sans-serif, system-ui, sans-serif"
    fontSize: "20px"
    fontWeight: 600
    letterSpacing: "-0.015em"
  title:
    fontFamily: "Geist Sans, ui-sans-serif, system-ui, sans-serif"
    fontSize: "15px"
    fontWeight: 600
    letterSpacing: "-0.01em"
  body:
    fontFamily: "Geist Sans, ui-sans-serif, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.55
    fontFeature: "\"tnum\" 1"
  body-small:
    fontFamily: "Geist Sans, ui-sans-serif, system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 400
    fontFeature: "\"tnum\" 1"
  label:
    fontFamily: "Geist Sans, ui-sans-serif, system-ui, sans-serif"
    fontSize: "11px"
    fontWeight: 400
    letterSpacing: "0.08em"
  mono:
    fontFamily: "JetBrains Mono, ui-monospace, monospace"
    fontSize: "12px"
    fontWeight: 500
rounded:
  swatch: "3px"
  sm: "4px"
  md: "6px"
  lg: "8px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "14px"
  lg: "16px"
  section: "44px"
  gutter: "40px"
components:
  picker:
    backgroundColor: "{colors.sheet-2}"
    textColor: "{colors.ink}"
    rounded: "{rounded.md}"
    padding: "6px 10px"
  field-cell:
    textColor: "{colors.ink}"
    padding: "8px 14px 9px"
  plot-strip-button:
    textColor: "{colors.ink}"
    typography: "{typography.body}"
    padding: "4px 14px 4px 12px"
  matrix-cell:
    backgroundColor: "{colors.sheet}"
    textColor: "{colors.ink}"
    padding: "10px 16px"
  matrix-plot-header:
    backgroundColor: "{colors.sheet-2}"
    textColor: "{colors.ink}"
    padding: "10px 16px"
  matrix-plot-header-hover:
    backgroundColor: "{colors.sheet-3}"
  badge-check:
    textColor: "{colors.bad}"
    rounded: "{rounded.sm}"
    padding: "1px 7px"
  badge-ok:
    textColor: "{colors.good}"
    rounded: "{rounded.sm}"
    padding: "1px 7px"
  badge-same:
    backgroundColor: "{colors.sheet-3}"
    textColor: "{colors.ink-2}"
    rounded: "{rounded.sm}"
    padding: "1px 7px"
  chip:
    backgroundColor: "{colors.sheet-3}"
    textColor: "{colors.ink-2}"
    rounded: "{rounded.sm}"
    padding: "0 7px"
  chip-flag:
    backgroundColor: "{colors.sheet-3}"
    textColor: "{colors.ink}"
    rounded: "{rounded.sm}"
  direction-tag:
    textColor: "{colors.ink-2}"
    rounded: "{rounded.sm}"
    padding: "0 6px"
  mover-flag:
    backgroundColor: "{colors.sheet-3}"
    textColor: "{colors.ink-2}"
    typography: "{typography.mono}"
    height: "18px"
    padding: "0 10px 0 5px"
  mover-flag-hover:
    backgroundColor: "{colors.rule-2}"
    textColor: "{colors.ink}"
  mover-flag-selected:
    textColor: "{colors.accent-amber}"
  ledger-header:
    backgroundColor: "{colors.sheet-2}"
    textColor: "{colors.muted}"
    typography: "{typography.label}"
    padding: "7px 8px"
  ledger-row:
    textColor: "{colors.ink}"
    padding: "7px 8px"
  ledger-row-hover:
    backgroundColor: "{colors.sheet-3}"
  tool-button:
    textColor: "{colors.ink-2}"
    rounded: "{rounded.sm}"
    padding: "0 8px"
  tool-button-pressed:
    backgroundColor: "{colors.sheet-3}"
    textColor: "{colors.ink}"
  tooltip:
    backgroundColor: "{colors.sheet-2}"
    textColor: "{colors.ink}"
    rounded: "{rounded.md}"
    padding: "8px 10px"
---

# Design System: Petri long-term record

## Overview

**Creative North Star: "The Field Record on a Slate Sheet"**

Petri's progress surface reads like a long-term ecological monitoring record: three plots are counted after every intervention, and each intervention is logged in a ruled register. The sheet is dark slate, warm-inked, and ruled with hairlines. The ruling, the tracked uppercase field labels and the tabular figures give the page its structure. Panels, tiles and drop shadows do not.

Density stays low on purpose. Each reading is one row of the census matrix, and each cell says one thing: a value, two signed deltas, one small curve. Colour has fixed jobs. Each plot keeps its crayon on every surface. Amber marks only the closure being read. Green and red mark only verdicts. Everything else is ink on slate.

The system refuses the KPI-tile-and-chart-grid dashboard. It is built as a form you fill in, not as a set of cards.

**Key Characteristics:**
- Dark slate tonal ramp of three surface steps and two hairline weights. No elevation except the floating tooltip.
- Ruled cells everywhere: the field strip, the census matrix and the register share 1px rules and an 8px outer radius.
- Three fixed plot crayons, validated together for colour-vision deficiency on the sheet.
- One amber accent, reserved for the read closure and the interaction affordances (keyboard focus, link-hover underline, selection tint).
- Tracked uppercase field labels stand in for small caps. Figures are tabular. Large values are proportional.
- Charts are Observable Plot, styled from the same tokens. No chart chrome, no legends where position or colour already tells you.

## Colors

A cool slate field with warm off-white ink, three crayons for the plots, and one amber kept for a single job.

### Primary
- **Survey Amber** (accent-amber): the closure being read. It is the selected mover flag, its staff, the vertical read-closure rule in every census curve, the selected register row (10% tint, 45% bottom rule), keyboard focus (2px outline, 2px offset), link hover underlines and text selection (28% tint). Nothing else is amber.

### Plot crayons
- **Meadow Green** (plot-11-orchards): Orchards, plot 11.
- **Sandstone Ochre** (plot-22-canyon): Canyon, plot 22.
- **River Blue** (plot-33-confluence): Confluence, plot 33.

These three colours were checked against the sheet across all pairs (worst colour-vision ΔE 9.4). They are identity colours. A plot keeps its crayon in the strip button, the 2px header underline, the 14px header swatch, the census line, its 8% area fill, its dashed first-census rule, the register swatches and the stacked mobile dots.

### Verdict
- **Held Green** (good): better-than-before deltas, OK badges and good chips, always with a signed numeral or a word.
- **Check Red** (bad): worse deltas, CHECK badges, severe cost flags and the below-floor chip. Severe also turns the weight up to 600.

### Neutral
- **Slate Sheet** (sheet): page ground. Census curves and dots ring against it.
- **Survey Box** (sheet-2): matrix headers and corner, register header, picker, tooltip, `pre` blocks.
- **Raised Box** (sheet-3): hover tone for rows and headers, mover flags at rest, neutral chips and badges.
- **Hairline** (rule): every cell rule, the outer rules of the matrix, field strip and register, and delta-line rules.
- **Strong Hairline** (rule-2): control borders (picker, plot strip, tags, tool buttons, tooltip) and link underlines.
- **Field Ink** (ink): values, titles and verdict lines.
- **Second Ink** (ink-2): secondary prose, plot axes and tick labels, delta values at rest, mover-flag text.
- **Muted Ink** (muted): field labels, units, delta keys, definitions, small print, flat or no-change readings.

### Named Rules
**The One Amber Rule.** Amber means "the closure you are reading". Its only other uses are the interaction affordances: keyboard focus, link-hover underlines and the selection tint. Never use it for emphasis, warnings or decoration.

**The Crayon Identity Rule.** A plot's colour belongs to that plot on every surface, and no other element may use a crayon. The crayons are never averaged, mixed or reused for a fourth series.

**The Ink-Weight Flag Rule.** A cost flag is marked by ink and weight (field ink at 600). Only severe goes red. Do not invent a hue for "flagged".

**The Tint-Not-Fill Rule.** Coloured grounds are always translucent tints of their hue: plot buttons use 20%, 32% on hover and 48% when pressed (`color-mix`); verdict chips and badges use 10 to 16%; amber selections use 10 to 16%. Solid colour fills appear only as small swatches and dots.

## Typography

**Display Font:** Geist Sans (with ui-sans-serif, system-ui), weights 400, 500 and 600, vendored.
**Body Font:** Geist Sans.
**Label/Mono Font:** JetBrains Mono (variable), vendored, for plot numbers, closure ids, mover flags and code paths.

**Character:** These are the live web UI's own faces (frontend/src/styles.css), so the record and the simulator share one voice. Geist carries the form. Mono marks anything that is an identifier rather than a word.

### Hierarchy
- **Display** (600, 26px, line-height 1.1, -0.02em): the page title and the large census value in each matrix cell. The cell value turns tabular figures off (proportional); its unit follows at 12px muted.
- **Headline** (600, 20px, -0.015em): section heads (Compute cost, Closure register, Supplementary census).
- **Title** (600, 15px, -0.01em): reading names in the matrix row labels. Disclosure summaries use 16px/600, and header field values use 16px/500.
- **Body** (400, 14px, 1.55): prose, verdict lines (13px) and ledes at a maximum of 80ch. Tabular figures are on for the whole page.
- **Body small** (400, 12px): delta lines, chips, tooltips, definitions, footer.
- **Label** (400 to 500, 11px, 0.08em, uppercase, muted): survey-form field labels, the picker label and register column heads. On narrow screens the register heads tighten to 0.04em.
- **Mono** (500, 12px; 10.5px in mover flags; 11px for code paths): plot numbers, closure ids and feature codes.

### Named Rules
**The Small-Caps Substitute Rule.** Geist ships no small caps. A field label on a ruled form cell is set as tracked uppercase at 11px muted. That is its only use. Never use it for a heading or a line above a heading.

**The Tabular Ledger Rule.** Every figure that can sit in a column is tabular, and every delta is signed (+, −). Only the single large value in a cell is proportional.

## Layout

The page is one sheet with a 1360px maximum width, centred, with 40px side gutters (18px below 820px). The vertical order is fixed:
1. The survey-sheet header: title row and picker.
2. The ruled field strip. Three equal field cells and a wider plots cell hold the record span, the closure count, the changed count and the plot strip.
3. The one-line latest-closure note.
4. The census matrix: the Cognition band, the Evolvability band, then an "Also watched" row.
5. Compute cost (flagged counters only, the rest folded), the closure register and the supplementary census, each 44px below the last.
6. The field key and the footer.

The census matrix is a CSS grid: a 236px label column, then one equal column per plot. Readings are rows, plots are columns, and every cell is ruled 1px on top and left, inside an 8px rounded outer rule. Full-width band rows name the two goals: Cognition (memory use) and Evolvability (mutations that change behaviour; new behaviour after long drift). A last "Also watched" row holds founder families and creatures alive as values with a vs-previous delta, no curves. Each reading cell stacks:
- the value;
- a two-column delta pair (since first / vs previous) on 1px-ruled lines;
- a mover lane 40px tall, only over the evolvability headline;
- a census curve 88px tall on a linear y scale from zero that the whole row shares.

At 1100px and below, the matrix collapses to one column. The plot headers and the corner cell hide, and each cell shows its own plot label instead. The field strip becomes three cells over a full-width plots cell, and the cost box stacks. At 820px and below, the register hides its number and run columns, and the per-plot trio stacks vertically with a crayon dot before each value.

Spacing is a loose 4px rhythm with recurring steps of 4, 8, 14, 16, 40 and 44px. Cell padding is 10px 16px, and field cells use 8px 14px.

## Elevation & Depth

The system is flat. Depth comes from tonal layering: the sheet is the ground, survey boxes sit one step up, and the raised tone is used for hover. A hover always steps up exactly one tone. Hairlines, not shadows, separate regions.

### Shadow Vocabulary
- **Floating tooltip** (`box-shadow: 0 10px 28px rgba(0, 0, 0, 0.45)`): the fixed-position chart tooltip only. It is the single element that floats above the sheet.

### Named Rules
**The Ruled-Not-Raised Rule.** Separate regions with 1px rules and tone steps, never with shadows or cards. The tooltip is the only exception.

## Shapes

The geometry is rectilinear with gently softened corners:
- 8px on outer ruled containers (field strip, matrix, register, tables, notice box);
- 6px on controls (picker, plot strip, tooltip, `pre`);
- 4px on chips, badges, tags and tool buttons;
- 2 to 3px on swatches.

Inner cell rules meet square. The one distinctive silhouette is the notched mover flag: a pennant clipped with a 6px V-notch on its free end and a 1.5px staff edge on the side it hangs from. It flips when it would overrun the cell edge. Census curves are 1.8px lines over an 8% area fill. The first census is an open diamond with a dashed rule (3,3 at 55% opacity) in the plot's colour. The read closure is a filled dot ringed in sheet colour.

## Components

### Plot strip (header)
A ruled segmented control. Each segment is filled with its plot's crayon as a 20% tint, rising to 32% on hover and 48% when pressed. Segments are divided by strong hairlines inside a 6px rounded border, and each shows the mono plot number in second ink, then the name. When one plot is brought forward, the others drop to 45% opacity.

### Census matrix
- **Corner and plot headers:** survey-box ground. Each plot header carries a 14px crayon swatch, the mono number and the name at 14px/600, with a 2px crayon underline. Hover steps up to the raised tone. A focused plot dims the other columns' headers and cells to 40%.
- **Row label:** reading title and a direction tag ("Higher is better": 11px/600, strong-hairline border, 4px radius), then the verdict line in field ink against the first census and definitions in muted small print.
- **Cell:** display value with a muted unit, then the ruled delta pair. Delta values are second ink at rest, green or red when they cross the typical move, field ink at 600 for a flag, and muted for "no change".
- **Band row:** a full-width ruled row with the goal name at headline size (20px/600) and its question in second ink. Cognition also carries "Learning and prediction are not measured yet" in muted small print.
- **Census curve:** shared row scale with a linear y axis from zero, three ticks, gridlines, an amber read-closure rule and the dashed first-census rule at the reading's first measured closure. Hovering one cell draws a shared hover line across every census cell and the register.
- **Below floor:** a mutations cell under its floor adds a red "below floor" chip beside the value.

### Mover flags
Notched mono pennants (10.5px, 18px tall) on a two-tier 40px lane, each with a faint 1px staff (35% opacity) dropping into the curve. At rest they are raised tone with second ink. Hover lifts them to strong-hairline tone with field ink. The selected flag turns amber at a 16% tint, and its staff turns amber.

### Closure register
A ruled ledger inside an 8px border. Its columns follow the two goals: memory use, mutations that change behaviour, floor, new behaviour after long drift, cost. The sticky header row is survey box with field labels, and each per-plot column head carries a row of three 44px crayon swatches under its label, one above each plot's numeral. Rows are 13px. The closure number is mono and muted. Each plot's delta is a right-aligned 12px value. Rows are clickable: hover is the raised tone, and the selected row is a 10% amber tint with an amber number. Consecutive no-change closures fold into one muted line that you can expand.

### Badges and chips
- **Badges:** verdict stamps for the latest-closure note (OK, CHECK, SAME) at 11px/600 and 0.06em tracking, as a tinted ground with a 4px radius.
- **Chips:** inline change readings, with an arrow or sign for direction and colour for better or worse. Small and flat changes are muted on a transparent ground.

### Picker and tool buttons
- **Picker:** a native select on survey box, with a strong-hairline border, 6px radius and 13px text.
- **Tool buttons:** 12px with a strong-hairline border and 4px radius. Hover and pressed step to the raised tone with field ink.
- **Focus:** keyboard focus everywhere is a 2px amber outline with a 2px offset.

### Supplementary census
Topic disclosures ruled top and bottom. The summary is a 16px/600 title with a muted 13px scope line. The disclosure marker is a CSS chevron drawn from two 1.5px borders, rotating 90° in 160ms. Open topics hold Plot charts in a 28px-gap stack, indented 21px.

### Motion
State transitions only: 120 to 160ms ease-out on background, opacity, colour and border. All transitions are removed under `prefers-reduced-motion`.

## Do's and Don'ts

### Do:
- **Do** draw every chart with the token hexes mirrored in script (identity colours for the crayons, transparent background, Geist 12px in second ink).
- **Do** keep each plot in its crayon everywhere, and show the first census as a dashed rule in that crayon.
- **Do** rule cells with 1px hairlines inside an 8px outer radius, and step hover up exactly one tone.
- **Do** sign every delta and set figures tabular. Leave only the large cell value proportional.
- **Do** pair colour with a signed numeral, an arrow, a word or position. Colour never carries meaning alone.
- **Do** put the read closure in amber across every census curve, flag, staff and register row at once.

### Don't:
- **Don't** use amber for anything except the read closure and the interaction affordances (keyboard focus, link-hover underline, selection tint).
- **Don't** give a cost flag its own hue. Use field ink at 600, and red only for severe.
- **Don't** build KPI tiles, cards or a chart grid. Readings live in the ruled matrix.
- **Don't** add drop shadows to sheets, cells or controls. The tooltip is the only floating element.
- **Don't** use tracked uppercase as an eyebrow or kicker above a heading. It belongs to field labels on ruled cells only.
- **Don't** introduce a fourth series colour from the crayon family, or average plots into one number or one line.
