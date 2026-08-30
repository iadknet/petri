# Petri V3 UI capture

- **Capture UTC:** 2026-08-30T02:45:56Z
- **Source commit:** `85dfa14525ef7874e1fb734e00e0ff4790cb1158`
- **Runtime:** release-mode local Petri V3 application
- **Viewport:** 1600 × 1000 CSS pixels
- **World:** 512 × 512 cells
- **Seed:** `424242`
- **Initial population:** 64 creatures
- **Captured state:** initialized, started, paused, then advanced once with
  **Step**; paused at tick 84 with 2,875 creatures and the **Evolution**
  statistics tab open.
- **Image SHA-256:** `d42a0996726d071bd3288acd649c59770c566ad390aa8ac1f9ad4d3460dfe261`

The capture was produced from a detached worktree at the exact source commit.
Its tracked tree and index were clean before capture; generated dependencies and
build output remained untracked inside that disposable worktree. This isolated
the capture from uncommitted work in the primary checkout.

The real initialized simulation was driven through the repository's
agent-browser snapshot/ref workflow. Browser errors and console output were
empty immediately after capture, and the unannotated PNG was visually inspected
at its original 1600 × 1000 resolution.
