# Frontend instructions

- Keep transport code faithful to the typed v3 server API; never fabricate runtime state in the UI.
- Wire types are generated into `src/types/generated/`; never hand-copy them. Regenerate with
  `PETRI_WRITE_TS_TYPES=1 cargo test -p v3-server generated_types`, then `npm run lint:fix`.
- Preserve keyboard access, visible focus, semantic controls, readable contrast, and accessible labels.
- Use TDD for behavior changes and bug fixes. Run `npm run lint`, `npm run test`, and
  `npm run build` from this directory for frontend changes.
