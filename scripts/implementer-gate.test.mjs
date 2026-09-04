import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import test from 'node:test';

const gate = join(process.cwd(), 'scripts', 'implementer-gate');

function git(repository, ...args) {
  const result = spawnSync('git', ['-C', repository, ...args], { encoding: 'utf8' });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}

function write(repository, path, content) {
  const file = join(repository, path);
  mkdirSync(file.slice(0, file.lastIndexOf('/')), { recursive: true });
  writeFileSync(file, content);
}

function validRoadmap(repository) {
  write(repository, 'docs/roadmap.md', '# Program\n\n**Status**: Active\n\n## Track Roadmaps\n\n- [ ] **T01 — Core** — [Roadmap](roadmaps/t01-core.md) — Depends on: None\n\n## Final Success Criteria\n\n- [ ] Program complete.\n');
  write(repository, 'docs/roadmaps/t01-core.md', '# T01 — Track\n\n**Status**: Planned\n\n## Track Success Criteria\n\n- [ ] Track complete.\n\n## Executable Features\n\n- [ ] **T01.F01 — Foundation** — Depends on: None\n');
  write(repository, 'docs/specs/roadmap/t01-f01-foundation.md', '# T01.F01 — Feature\n\n**Status**: Planned\n**Feature**: T01.F01\n**Last updated**: whenever\n\n## Goal\n\nGoal.\n\n## Implementation Tasks\n\n- [ ] Implement.\n\n## Verification\n\n- [ ] Verify.\n\n## Success Criteria\n\n- [ ] Succeed.\n');
}

function repo() {
  const repository = mkdtempSync(join(tmpdir(), 'petri-implementer-gate.'));
  git(repository, 'init', '-q', '-b', 'main');
  git(repository, 'config', 'user.email', 'test@example.com');
  git(repository, 'config', 'user.name', 'Test');
  git(repository, 'config', 'commit.gpgsign', 'false');
  validRoadmap(repository);
  write(repository, 'src/lib.rs', 'pub fn one() -> u8 { 1 }\n');
  git(repository, 'add', '.');
  git(repository, 'commit', '-q', '-m', 'base');
  git(repository, 'checkout', '-q', '-b', 'feature');
  return repository;
}

function run(cwd, input = JSON.stringify({ cwd, hook_event_name: 'SubagentStop' })) {
  return spawnSync('sh', [gate], {
    input,
    encoding: 'utf8',
    env: { ...process.env, NODE: process.execPath },
  });
}

function breakTrack(repository) {
  const path = join(repository, 'docs/roadmaps/t01-core.md');
  writeFileSync(path, readFileSync(path, 'utf8').replace('**Status**: Planned', '**Status**: Bogus'));
}

test('no roadmap document changed: exit 0 without running the checker', () => {
  const repository = repo();
  try {
    write(repository, 'src/lib.rs', 'pub fn one() -> u8 { 2 }\n');
    const result = run(repository);
    assert.equal(result.status, 0, result.stderr);
  } finally { rmSync(repository, { recursive: true, force: true }); }
});

test('uncommitted roadmap breakage: exit 2 with the checker output', () => {
  const repository = repo();
  try {
    breakTrack(repository);
    let result = run(repository);
    assert.equal(result.status, 2, result.stderr);
    assert.match(result.stderr, /invalid Status/);

    validRoadmap(repository);
    result = run(repository);
    assert.equal(result.status, 0, result.stderr);
  } finally { rmSync(repository, { recursive: true, force: true }); }
});

test('committed roadmap breakage on the feature branch is caught via the merge base', () => {
  const repository = repo();
  try {
    breakTrack(repository);
    git(repository, 'commit', '-q', '-am', 'break');
    const result = run(repository);
    assert.equal(result.status, 2, result.stderr);
    assert.match(result.stderr, /invalid Status/);
  } finally { rmSync(repository, { recursive: true, force: true }); }
});

test('cwd inside a subdirectory resolves to the worktree root', () => {
  const repository = repo();
  try {
    breakTrack(repository);
    const result = run(join(repository, 'src'));
    assert.equal(result.status, 2, result.stderr);
  } finally { rmSync(repository, { recursive: true, force: true }); }
});

test('missing or unusable cwd never blocks', () => {
  const repository = repo();
  try {
    assert.equal(run(repository, '{"hook_event_name":"SubagentStop"}').status, 0);
    assert.equal(run(repository, JSON.stringify({ cwd: join(repository, 'does-not-exist') })).status, 0);
  } finally { rmSync(repository, { recursive: true, force: true }); }
});
