# Petri V2 (Greenfield)

`v2` is a fully isolated rewrite target focused on mesh-DNA cognition and emergent behavior.

## Boundaries

- Implement new runtime/application code only under `v2/`.
- Legacy crates and web app are reference-only.
- Reuse from legacy is copy-only; do not import old crates into `v2`.

## Workspace

Rust crates:
- `crates/v2-core`
- `crates/v2-server`
- `crates/v2-cli`

Web app:
- `web/`

## Quick Commands

```bash
cd v2
cargo check

cd v2/web
npm install
npm run build
```
