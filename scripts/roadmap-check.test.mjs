import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import test from 'node:test';

const checker = join(process.cwd(), 'scripts', 'roadmap-check.mjs');

function root() {
  return mkdtempSync(join(tmpdir(), 'petri-roadmap-check.'));
}

function run(repository) {
  return spawnSync(process.execPath, [checker, '--root', repository], { encoding: 'utf8' });
}

function write(repository, path, content) {
  const file = join(repository, path);
  mkdirSync(file.slice(0, file.lastIndexOf('/')), { recursive: true });
  writeFileSync(file, content);
}

function master(rows, { status = 'Active', criteria = '- [ ] Program complete.' } = {}) {
  return `# Program\n\n**Status**: ${status}\n\n## Track Roadmaps\n\n${rows.join('\n')}\n\n## Final Success Criteria\n\n${criteria}\n`;
}

function track(id, features, { status = 'Planned', criteria = '- [ ] Track complete.' } = {}) {
  return `# ${id} — Track\n\n**Status**: ${status}\n\n## Track Success Criteria\n\n${criteria}\n\n## Executable Features\n\n${features.join('\n')}\n`;
}

function spec(id, { status = 'Planned', checked = false } = {}) {
  const mark = checked ? 'x' : ' ';
  return `# ${id} — Feature\n\n**Status**: ${status}\n**Feature**: ${id}\n**Last updated**: whenever useful\n\n## Goal\n\nGoal. Mentioning readiness review or any retired workflow term is ordinary prose.\n\n## Implementation Tasks\n\n- [${mark}] Implement.\n\n## Verification\n\n- [${mark}] Verify.\n\n## Success Criteria\n\n- [${mark}] Succeed.\n`;
}

function base(repository, options = {}) {
  const featureChecked = options.featureChecked ?? false;
  const trackStatus = options.trackStatus ?? 'Planned';
  const trackChecked = options.trackChecked ?? (trackStatus === 'Complete');
  write(repository, 'docs/roadmap.md', master([
    `- [${trackChecked ? 'x' : ' '}] **T01 — Core** — [Roadmap](roadmaps/t01-core.md) — Depends on: None`,
  ], { status: options.masterStatus, criteria: options.masterCriteria }));
  write(repository, 'docs/roadmaps/t01-core.md', track('T01', [
    `- [${featureChecked ? 'x' : ' '}] **T01.F01 — Foundation** — Depends on: ${options.dependency ?? 'None'}`,
  ], { status: trackStatus, criteria: options.trackCriteria }));
  if (options.spec !== false) {
    write(repository, 'docs/specs/roadmap/t01-f01-foundation.md', spec('T01.F01', {
      status: options.specStatus,
      checked: options.specChecked,
    }));
  }
}

test('empty repositories and templates pass; live files require a master', () => {
  const repository = root();
  try {
    write(repository, 'docs/roadmaps/_track-template.md', '# Template\n');
    assert.equal(run(repository).status, 0);
    write(repository, 'docs/roadmaps/t01-orphan.md', '# Live\n');
    const result = run(repository);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /require a master roadmap/);
  } finally { rmSync(repository, { recursive: true, force: true }); }
});

test('a valid cross-track dependency graph passes', () => {
  const repository = root();
  try {
    write(repository, 'docs/roadmap.md', master([
      '- [ ] **T01 — First** — [Roadmap](roadmaps/t01-first.md) — Depends on: None',
      '- [ ] **T02 — Second** — [Roadmap](roadmaps/t02-second.md) — Depends on: None',
    ]));
    write(repository, 'docs/roadmaps/t01-first.md', track('T01', [
      '- [ ] **T01.F01 — First** — Depends on: T02.F01',
    ]));
    write(repository, 'docs/roadmaps/t02-second.md', track('T02', [
      '- [ ] **T02.F01 — Second** — Depends on: None',
    ]));
    write(repository, 'docs/specs/roadmap/t01-f01-first.md', spec('T01.F01'));
    write(repository, 'docs/specs/roadmap/t02-f01-second.md', spec('T02.F01'));
    const result = run(repository);
    assert.equal(result.status, 0, result.stderr);
  } finally { rmSync(repository, { recursive: true, force: true }); }
});

test('broken links, duplicate ownership, and orphan specs fail', () => {
  const repository = root();
  try {
    base(repository);
    let path = join(repository, 'docs/roadmap.md');
    writeFileSync(path, readFileSync(path, 'utf8').replace('t01-core.md', 't01-missing.md'));
    let result = run(repository);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /not linked|missing/);

    base(repository);
    path = join(repository, 'docs/roadmaps/t01-core.md');
    writeFileSync(path, readFileSync(path, 'utf8').replace(
      '## Executable Features',
      '## Executable Features\n\n- [ ] **T01.F01 — Duplicate** — Depends on: None',
    ));
    result = run(repository);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /duplicate feature ownership/);

    base(repository);
    write(repository, 'docs/specs/roadmap/t01-f99-orphan.md', spec('T01.F99'));
    result = run(repository);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /orphan feature spec/);

    rmSync(join(repository, 'docs/specs/roadmap/t01-f99-orphan.md'));
    write(repository, 'docs/specs/roadmap/t01-f01-copy.md', spec('T01.F01'));
    result = run(repository);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /multiple feature specs/);
  } finally { rmSync(repository, { recursive: true, force: true }); }
});

test('missing dependencies and dependency cycles fail', () => {
  const repository = root();
  try {
    base(repository, { dependency: 'T02.F01' });
    let result = run(repository);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /unknown feature dependency/);

    base(repository, { dependency: 'T01.F01' });
    result = run(repository);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /feature dependency cycle/);
  } finally { rmSync(repository, { recursive: true, force: true }); }
});

test('feature checkboxes and complete specs must agree', () => {
  const repository = root();
  try {
    base(repository, { featureChecked: true, specStatus: 'Planned' });
    let result = run(repository);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /feature checkbox and Complete spec must agree/);

    base(repository, { specStatus: 'Complete', specChecked: true });
    result = run(repository);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /feature checkbox and Complete spec must agree/);
  } finally { rmSync(repository, { recursive: true, force: true }); }
});

test('complete specs require completed implementation, verification, and success lists', () => {
  const repository = root();
  try {
    base(repository, { featureChecked: true, specStatus: 'Complete', specChecked: false });
    const result = run(repository);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /unchecked Implementation Tasks items/);
    assert.match(result.stderr, /unchecked Verification items/);
    assert.match(result.stderr, /unchecked Success Criteria items/);
  } finally { rmSync(repository, { recursive: true, force: true }); }
});

test('track and master completion rollups must agree', () => {
  const repository = root();
  try {
    base(repository, { trackStatus: 'Complete', trackChecked: false, featureChecked: true, specStatus: 'Complete', specChecked: true, trackCriteria: '- [x] Track complete.' });
    let result = run(repository);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /rollup must match/);

    base(repository, { masterStatus: 'Complete', trackStatus: 'Complete', featureChecked: true, specStatus: 'Complete', specChecked: true, trackCriteria: '- [x] Track complete.' });
    result = run(repository);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Final Success Criteria/);

    base(repository, { masterStatus: 'Complete', masterCriteria: '- [x] Program complete.', trackStatus: 'Complete', featureChecked: true, specStatus: 'Complete', specChecked: true, trackCriteria: '- [x] Track complete.' });
    result = run(repository);
    assert.equal(result.status, 0, result.stderr);
  } finally { rmSync(repository, { recursive: true, force: true }); }
});

test('unsupported arguments return a usage error', () => {
  const result = spawnSync(process.execPath, [checker, '--staged'], { encoding: 'utf8' });
  assert.equal(result.status, 2);
  assert.match(result.stderr, /Usage/);
});

test('a spec over the prose budget fails; tables and fenced blocks do not count', () => {
  const repository = root();
  try {
    base(repository);
    const filler = `${'Narrative prose that says how the work went. '.repeat(20)}\n\n`;
    write(repository, 'docs/specs/roadmap/t01-f01-foundation.md',
      `${spec('T01.F01')}\n${filler.repeat(20)}`);
    const over = run(repository);
    assert.equal(over.status, 1);
    assert.match(over.stderr, /prose budget/);

    const rows = `| a | b |\n| --- | --- |\n${'| data | data |\n'.repeat(900)}`;
    const fenced = `\`\`\`\n${'log line output\n'.repeat(900)}\`\`\`\n`;
    write(repository, 'docs/specs/roadmap/t01-f01-foundation.md',
      `${spec('T01.F01')}\n${rows}\n${fenced}`);
    assert.equal(run(repository).status, 0);
  } finally {
    rmSync(repository, { recursive: true, force: true });
  }
});

test('Notes for AI Agents accepts only labelled bullets', () => {
  const repository = root();
  try {
    base(repository);
    const notes = (body) => `${spec('T01.F01')}\n## Notes for AI Agents\n\n${body}\n`;
    write(repository, 'docs/specs/roadmap/t01-f01-foundation.md',
      notes('The orchestrator reviewed the first draft and then changed its mind.'));
    const bad = run(repository);
    assert.equal(bad.status, 1);
    assert.match(bad.stderr, /Notes for AI Agents/);

    write(repository, 'docs/specs/roadmap/t01-f01-foundation.md',
      notes('- Decision: the user accepted the measured cost on 2026-09-10.\n- Deferred: survivor at vm.rs:382 times out; see Notes.\n- Cost: 4 advisor consults, 0 P1.'));
    assert.equal(run(repository).status, 0);
  } finally {
    rmSync(repository, { recursive: true, force: true });
  }
});
