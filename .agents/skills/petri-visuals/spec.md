# petri-viz spec format

`PetriViz.mount(target, spec)` renders into `target` (a selector or element).
A spec is plain JSON. Full working example: `viz/example.json`.

```json
{ "views": [View, ...], "startView": 0, "startTick": -1 }
```

## View

| Key | Meaning |
|---|---|
| `name` | Button label; buttons appear when there are 2+ views |
| `caption` | One line under the buttons |
| `header` | Replaces `Graph (n in · m compute · k out)` |
| `nodes` | The graph, see Node |
| `groups` | `[{ "label", "ids": [...] }]`: dashed teal box around nodes in one column |
| `projections` | `[{ "ring", "kind", "offset", "w", "gain"?, "isNew"?, "note"? }]`: rows like the inspector's ring projection list; `offset` is any integer, read mod 8; `gain` names a node whose tick value scales the votes |
| `projectionsTitle` | Default `Ring projections` |
| `grid` | `{ "title"?, "cols", "rows": [{ "label", "key", "w": [...] }], "vote"? }`: weight table; `key` is a node id (its tick value) or a number; a `null` weight shows "no edge"; `vote` names a bottom row of Σ value × weight per column |
| `ticks` | Example ticks, see Tick; adds an "Example tick" button bar |
| `cards` | `[[label, value], ...]`: metric cards under the panel |
| `notes` | `[string, ...]`: small lines under the panel |
| `edgeLabels` | `false` hides weights on edges (use when a grid shows them) |
| `sort` | `true` orders later columns by their sources' height; default keeps spec order |
| `empty` | Text shown when there are no nodes |

## Node

Every node has an `id`. `from` is `[[sourceId, weight], ...]`. `dim: true`
draws a silent or unused node dimmed and dashed.

- **Input** `{ "id", "in": "Nearest1.kin" }`. Use inspector labels: `OccRing[E]`,
  `FoodRing[N]`, `AreaOcc.crowding`, `Nearest2.dx`, `Energy`, `Age`,
  `PrevOutcome.damage`, `Queue1.type`, `Votes.Move[E]`, `Bus[2]`, `Mem[3]`.
- **Compute** `{ "id", "kind", "from" }`. `kind` uses the genome JSON form:
  `"Add"`, `"WeightedSum"`, `"Multiply"`, `"Negate"`, `"Abs"`, `"Min"`, `"Max"`,
  `"Sigmoid"`, `"Tanh"`, `"Relu"`, `"Clamp01"`, `{"Threshold": t}`, `"GreaterThan"`,
  `"Select"`, `{"DecayIntegrator": a}`, `{"Momentum": b}`, `{"Oscillator": f}`,
  `"AdaptiveGain"`, `{"Constant": v}`. Label, color and number follow the
  inspector; `idx: false` hides the number.
- **Sink** `{ "id", "out": "Vote Move[E]", "sub"?, "from" }`. `sub` defaults to
  `action vote`; use `action parameter` for `Param …`. Add `"cat": "output_value"`
  for `Bus[n]` and `Write Mem[n]`. Labels: `Vote Move[E]`, `Vote Drive StealEnergy`,
  `Vote Eat`, `Vote Terminate`, `Param StealEnergy.amt`.
- **Custom** `{ "id", "label", "cat", "from"?, "kind"?, "sub"? }` for anything the
  genome cannot express. `"cat": "proposal"` (teal) marks a proposed mechanism.
  Without `kind` it evaluates as a weighted sum; pin any other value with
  `tick.values`.

Columns follow the inspector: inputs and constants first, compute by depth,
sinks last. A graph wider than the panel scrolls sideways rather than shrinking
its text, so keep labels short and graphs to about five columns.

## Tick

```json
{
  "name": "Weak stranger to the E",
  "values": { "i0": 1, "i3": 0.2 },
  "prev": { "c4": 0.3 },
  "prevOut": { "c2": 0.5 },
  "rings": { "OccRing": [0, 0, 1, 0, 0, 0, 0, 0] },
  "scene": { "creatures": [[1, 0, "stranger"]], "arrow": { "kind": "Steal" } },
  "note": "One line saying what to notice."
}
```

- `values` sets inputs. Compute nodes then run in the order listed, as the
  runtime runs the genome's node array, with v3-core's math; sinks run last.
  Any node listed in `values` is pinned instead.
- `prevOut` gives a node's previous output: what a compute node reads from a
  source listed after it (or from itself) because that source has not run yet
  this tick. Default 0.
- `prev` is a stateful node's operator state before this run (default 0, as in
  the runtime). The node shows `old→new`.
- `rings` gives a projection ring's 8 cells (N, NE, E, SE, S, SW, W, NW); the
  projection row then lists the votes it adds.
- `scene` draws a 7×7 view around the creature (white): `[dx, dy, "kin" |
  "stranger" | "other"]`, with dy positive = south. `arrow` is a direction or
  `{ "kind" }`, which picks the sink whose label contains `kind[DIR]` with the
  highest positive value. `r` changes the radius (default 3).
