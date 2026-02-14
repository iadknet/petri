# V2 Copy Policy

## Rule

If legacy logic is reused:
1. Copy code into `v2`.
2. Adapt and test it in `v2`.
3. Record provenance in the commit message or stage notes.

## Non-Rule

Do not integrate or link `v2` to legacy crates/modules as dependencies.
