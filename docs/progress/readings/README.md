# Feature readings

Hand-written measured evidence for closed roadmap features: comparison tables,
per-seed dumps, neighborhood rows, and pass-by-pass verification logs. One file
per feature, named for its spec (`tNN-fNN-<slug>.md`).

The spec at `docs/specs/roadmap/<id>.md` remains the record of judgment: its
Performance and Goal Impact section carries the predeclaration, the one-line
verdict, and every user decision, and links here. The machine-written report is
`docs/progress/features/<id>.json`. These files are the human-readable middle
layer between the two, and are not rewritten after closure.

Readings files sit two levels under `docs/`, the same depth as
`docs/specs/roadmap/`, so relative links copied out of a spec (`../../progress/`,
`../../roadmaps/`) resolve unchanged.

## Historical records

These closed records describe a construction that no longer exists. They are
kept unedited; read them as history, not as the current model.

| Record | Historical because | Since |
| --- | --- | --- |
| T13.F02 to T13.F07 recruitment-path readings and records (`t13-f0[2-7]*` readings, `t13-f02`, `t13-f07`, `-s0`, `-s0-pilot` summaries) and every `recruitment_paths` block in summaries closed before T19.F04 | Activation used hand-built action banks; the same `recruitment-paths-v1` string names a different construction after T19.F04 | T19.F04 closure, `4bbe3d4c` |
| Steering structural readings `bank_written`/`bank_written_fraction` (T11.F21 to T19.F03 summaries) | "Executed node writes a bank"; T19.F04's `move_voted` ("contributes to a `Move` sink") is a different structure under the same `steering-v1` | T19.F04 closure |
