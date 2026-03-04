# Completion Reference

**Parent skill:** feature-6-complete/SKILL.md

## Completion Gate Commands (snapshot)

The canonical source is `AGENTS.md ## Completion Gate`. Always re-read it in case it has changed. Current snapshot:

```bash
scripts/check-plan-harness.sh --mode strict
scripts/check-doc-harness.sh --mode strict
scripts/check-architecture-harness.sh --mode strict
cd v3 && cargo fmt --all -- --check
cd v3 && cargo test --workspace
cd v3 && cargo clippy --workspace --all-targets -- -D warnings
```

**If frontend files were touched**, also run:

```bash
cd frontend && npm run build
```

## Move to Completed (after merge to main)

```bash
# Move feature directory
mv docs/features/in_progress/FEATURE-NAME/ docs/features/completed/FEATURE-NAME/

# Maintain .gitkeep so empty directory stays tracked
[ -z "$(ls -A docs/features/in_progress/ 2>/dev/null)" ] && touch docs/features/in_progress/.gitkeep

git add docs/features/completed/FEATURE-NAME/ docs/features/in_progress/
git commit -m "feat: mark FEATURE-NAME as completed"
```
