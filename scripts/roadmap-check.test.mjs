import { execFileSync, spawnSync } from 'node:child_process';
import { cpSync, mkdtempSync, mkdirSync, readFileSync, renameSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import assert from 'node:assert/strict';

const checker = join(process.cwd(), 'scripts', 'roadmap-check.mjs');

function fixture() {
  const root = mkdtempSync(join(tmpdir(), 'petri-roadmap-test.'));
  mkdirSync(join(root, 'docs', 'roadmaps'), { recursive: true });
  mkdirSync(join(root, 'docs', 'specs', 'roadmap'), { recursive: true });
  return root;
}

function run(root, ...args) {
  return spawnSync(process.execPath, [checker, '--root', root, ...args], { encoding: 'utf8' });
}

function writeFixture(root, { masterStatus = 'Active', trackStatus = 'In Progress', featureChecked = false, specStatus = 'In Progress', specChecked = false, blocker = 'None.', dependency = 'None' } = {}) {
  writeFileSync(join(root, 'docs', 'roadmap.md'), `# Program\n\n**Status**: ${masterStatus}\n**Last updated**: 2026-09-01\n\n## Success Definition\n\nOutcome.\n\n## Track Roadmaps\n\n- [${trackStatus === 'Complete' ? 'x' : ' '}] **T01 — Core** — [Roadmap](roadmaps/t01-core.md) — Depends on: None\n\n## Final Success Criteria\n\n- [${masterStatus === 'Complete' ? 'x' : ' '}] Program complete.\n\n## Notes for AI Agents\n\nNotes.\n`);
  writeFileSync(join(root, 'docs', 'roadmaps', 't01-core.md'), `# T01 — Core\n\n**Status**: ${trackStatus}\n**Last updated**: 2026-09-01\n**Master**: [Program Roadmap](../roadmap.md)\n\n## Goal\n\nGoal.\n\n## Track Success Criteria\n\n- [${trackStatus === 'Complete' ? 'x' : ' '}] Track complete.\n\n## Executable Features\n\n- [${featureChecked ? 'x' : ' '}] **T01.F01 — Foundation** — Depends on: ${dependency}\n\n## Notes for AI Agents\n\nNotes.\n`);
  writeFileSync(join(root, 'docs', 'specs', 'roadmap', 't01-f01-foundation.md'), `# T01.F01 — Foundation\n\n**Status**: ${specStatus}\n**Last updated**: 2026-09-01\n**Feature**: T01.F01\n**Track**: [T01 — Core](../../roadmaps/t01-core.md)\n\n## Overview\n\nOverview.\n\n## Goal\n\nGoal.\n\n## Non-Goals\n\nNone.\n\n## Inputs and Invariants\n\nNone.\n\n## Implementation Tasks\n\n- [${specChecked ? 'x' : ' '}] Implement.\n\n## Verification\n\n- [${specChecked ? 'x' : ' '}] Verify.\n\n## Success Criteria\n\n- [${specChecked ? 'x' : ' '}] Succeed.\n\n## Blocker\n\n${blocker}\n\n## Deferred Review Findings\n\nNone.\n\n## Notes for AI Agents\n\nNotes.\n`);
}

test('templates-only empty scaffold passes', () => {
  const root = fixture();
  try {
    writeFileSync(join(root, 'docs', 'roadmaps', '_master-template.md'), '# template\n');
    assert.equal(run(root).status, 0);
    assert.match(run(root).stdout, /no live roadmap files/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('present empty master is validated instead of treated as absent', () => {
  const root = fixture();
  try {
    writeFileSync(join(root, 'docs', 'roadmap.md'), ' \n\n');
    const result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /docs\/roadmap\.md: missing \*\*Status\*\* metadata/);
    assert.match(result.stderr, /docs\/roadmap\.md: missing heading '## Success Definition'/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('live tracks without a master fail', () => {
  const root = fixture();
  try {
    writeFileSync(join(root, 'docs', 'roadmaps', 't01-core.md'), '# T01\n');
    const result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /require a live master/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('nested live track and feature-spec files are rejected deterministically', () => {
  const root = fixture();
  try {
    writeFixture(root);
    mkdirSync(join(root, 'docs', 'roadmaps', 'nested'), { recursive: true });
    cpSync(join(root, 'docs', 'roadmaps', 't01-core.md'), join(root, 'docs', 'roadmaps', 'nested', 't01-core.md'));
    let result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /docs\/roadmaps\/nested\/t01-core\.md: non-canonical track path/);

    writeFixture(root);
    mkdirSync(join(root, 'docs', 'specs', 'roadmap', 'nested'), { recursive: true });
    cpSync(join(root, 'docs', 'specs', 'roadmap', 't01-f01-foundation.md'), join(root, 'docs', 'specs', 'roadmap', 'nested', 't01-f01-foundation.md'));
    result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /docs\/specs\/roadmap\/nested\/t01-f01-foundation\.md: non-canonical feature-spec path/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('nested live files without a master still require a live master', () => {
  const root = fixture();
  try {
    mkdirSync(join(root, 'docs', 'roadmaps', 'nested'), { recursive: true });
    mkdirSync(join(root, 'docs', 'specs', 'roadmap', 'nested'), { recursive: true });
    writeFileSync(join(root, 'docs', 'roadmaps', 'nested', 't01-core.md'), '# T01 — Core\n');
    writeFileSync(join(root, 'docs', 'specs', 'roadmap', 'nested', 't01-f01-feature.md'), '# T01.F01 — Feature\n');
    const result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /live track or feature-spec files require a live master/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('active roadmap with an in-progress feature passes', () => {
  const root = fixture();
  try {
    writeFixture(root);
    const result = run(root);
    assert.equal(result.status, 0, result.stderr);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('complete closure requires checked feature, spec, and criteria', () => {
  const root = fixture();
  try {
    writeFixture(root, { masterStatus: 'Complete', trackStatus: 'Complete', featureChecked: true, specStatus: 'Complete', specChecked: true });
    const result = run(root);
    assert.equal(result.status, 0, result.stderr);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('blocked specs require a concrete blocker and remain unchecked', () => {
  const root = fixture();
  try {
    writeFixture(root, { specStatus: 'Blocked' });
    let result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /concrete Blocker/);
    writeFixture(root, { specStatus: 'Blocked', blocker: 'Waiting on API decision.' });
    result = run(root);
    assert.equal(result.status, 0, result.stderr);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('invalid dependencies and cycles are reported', () => {
  const root = fixture();
  try {
    writeFixture(root, { dependency: 'T01.F99' });
    const result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /unknown feature dependency|unknown dependency/);
    writeFixture(root, { dependency: 'T01.F01' });
    const cycle = run(root);
    assert.equal(cycle.status, 1);
    assert.match(cycle.stderr, /feature dependency cycle/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('cross-track dependencies resolve even when the dependency track is listed later', () => {
  const root = fixture();
  try {
    writeFixture(root, { trackStatus: 'Planned', specStatus: 'Planned', dependency: 'T02.F01' });
    const roadmap = join(root, 'docs', 'roadmap.md');
    writeFileSync(roadmap, readFileSync(roadmap, 'utf8').replace(
      '## Final Success Criteria',
      '- [ ] **T02 — Later** — [Roadmap](roadmaps/t02-later.md) — Depends on: None\n\n## Final Success Criteria',
    ));
    writeFileSync(join(root, 'docs', 'roadmaps', 't02-later.md'), `# T02 — Later\n\n**Status**: Planned\n**Last updated**: 2026-09-01\n**Master**: [Program Roadmap](../roadmap.md)\n\n## Goal\n\nGoal.\n\n## Track Success Criteria\n\n- [ ] Track complete.\n\n## Executable Features\n\n- [ ] **T02.F01 — Later feature** — Depends on: None\n\n## Notes for AI Agents\n\nNotes.\n`);
    const result = run(root);
    assert.equal(result.status, 0, result.stderr);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('missing links, duplicate IDs, and orphan specs are reported without crashing', () => {
  const root = fixture();
  try {
    writeFixture(root);
    writeFileSync(join(root, 'docs', 'roadmap.md'), readFileSync(join(root, 'docs', 'roadmap.md'), 'utf8')
      .replace('roadmaps/t01-core.md', 'roadmaps/t99-missing.md'));
    const missing = run(root);
    assert.equal(missing.status, 1);
    assert.match(missing.stderr, /missing|orphan track|canonical path/);

    rmSync(join(root, 'docs', 'specs', 'roadmap', 't01-f01-second.md'), { force: true });
    rmSync(join(root, 'docs', 'specs', 'roadmap', 't01-f99-orphan.md'), { force: true });
    writeFixture(root, { trackStatus: 'Planned', specStatus: 'Planned' });
    cpSync(join(root, 'docs', 'specs', 'roadmap', 't01-f01-foundation.md'), join(root, 'docs', 'specs', 'roadmap', 't01-f01-second.md'));
    const duplicate = run(root);
    assert.equal(duplicate.status, 1);
    assert.match(duplicate.stderr, /multiple specs/);

    writeFixture(root, { trackStatus: 'Planned', specStatus: 'Planned' });
    cpSync(join(root, 'docs', 'specs', 'roadmap', 't01-f01-foundation.md'), join(root, 'docs', 'specs', 'roadmap', 't01-f99-orphan.md'));
    const orphan = run(root);
    assert.equal(orphan.status, 1);
    assert.match(orphan.stderr, /orphan feature spec/);

    rmSync(join(root, 'docs', 'specs', 'roadmap', 't01-f01-second.md'), { force: true });
    rmSync(join(root, 'docs', 'specs', 'roadmap', 't01-f99-orphan.md'), { force: true });
    writeFixture(root, { trackStatus: 'Planned', specStatus: 'Planned' });
    const track = join(root, 'docs', 'roadmaps', 't01-core.md');
    writeFileSync(track, readFileSync(track, 'utf8').replace(
      '## Notes for AI Agents',
      '- [ ] **T01.F01 — Duplicate** — Depends on: None\n\n## Notes for AI Agents',
    ));
    const duplicateId = run(root);
    assert.equal(duplicateId.status, 1);
    assert.match(duplicateId.stderr, /duplicate feature ID|duplicate feature ownership/);

    writeFixture(root, { trackStatus: 'Planned', specStatus: 'Planned' });
    const missingRow = join(root, 'docs', 'roadmap.md');
    writeFileSync(missingRow, readFileSync(missingRow, 'utf8').replace(/- \[ \] \*\*T01 — Core\*\*.*\n/, ''));
    const missingTrackRow = run(root);
    assert.equal(missingTrackRow.status, 1);
    assert.match(missingTrackRow.stderr, /missing its master track row/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('premature completion and rollup drift are rejected', () => {
  const root = fixture();
  try {
    writeFixture(root, { specStatus: 'Complete', specChecked: true });
    let result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Complete spec requires its feature checked/);

    writeFixture(root, { trackStatus: 'Complete', featureChecked: false, specStatus: 'Planned' });
    result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Complete track requires/);

    writeFixture(root);
    const roadmap = join(root, 'docs', 'roadmap.md');
    writeFileSync(roadmap, readFileSync(roadmap, 'utf8').replace('- [ ] **T01 — Core**', '- [x] **T01 — Core**'));
    result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /rollup.*if and only if/);

    writeFixture(root, { masterStatus: 'Complete' });
    result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Complete master requires/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('canonical identity and owning-track links are enforced', () => {
  const root = fixture();
  try {
    writeFixture(root);
    const track = join(root, 'docs', 'roadmaps', 't01-core.md');
    writeFileSync(track, readFileSync(track, 'utf8').replace('**Master**: [Program Roadmap](../roadmap.md)', '**Master**: [Wrong](../roadmap.md)'));
    const result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /expected \*\*Master\*\*/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('staged roadmap additions are materialized and validated', () => {
  const root = mkdtempSync(join(tmpdir(), 'petri-roadmap-git-add.'));
  try {
    execFileSync('git', ['init', '-q'], { cwd: root });
    mkdirSync(join(root, 'docs'), { recursive: true });
    writeFileSync(join(root, 'docs', 'roadmap.md'), '# incomplete\n');
    execFileSync('git', ['add', 'docs/roadmap.md'], { cwd: root });
    const result = spawnSync(process.execPath, [checker, '--staged'], { cwd: root, encoding: 'utf8' });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /missing \*\*Status\*\* metadata/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('staged no-op does not inspect unstaged roadmap files', () => {
  const root = mkdtempSync(join(tmpdir(), 'petri-roadmap-git.'));
  try {
    execFileSync('git', ['init', '-q'], { cwd: root });
    writeFileSync(join(root, 'README.md'), 'ok\n');
    execFileSync('git', ['add', 'README.md'], { cwd: root });
    mkdirSync(join(root, 'docs', 'roadmaps'), { recursive: true });
    writeFileSync(join(root, 'docs', 'roadmap.md'), 'invalid unstaged roadmap\n');
    const result = spawnSync(process.execPath, [checker, '--staged'], { cwd: root, encoding: 'utf8' });
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /no staged roadmap changes/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('staged deletion and rename are validated from the index', () => {
  const root = fixture();
  try {
    execFileSync('git', ['init', '-q'], { cwd: root });
    writeFixture(root);
    execFileSync('git', ['add', 'docs'], { cwd: root });
    execFileSync('git', ['-c', 'user.email=test@example.com', '-c', 'user.name=Test', 'commit', '-qm', 'fixture'], { cwd: root });
    execFileSync('git', ['mv', 'docs/roadmaps/t01-core.md', 'docs/roadmaps/t01-renamed.md'], { cwd: root });
    let result = spawnSync(process.execPath, [checker, '--staged'], { cwd: root, encoding: 'utf8' });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /canonical path|missing/);

    execFileSync('git', ['restore', '--staged', '--worktree', 'docs/roadmaps/t01-renamed.md'], { cwd: root });
    execFileSync('git', ['rm', '-q', 'docs/roadmap.md'], { cwd: root });
    result = spawnSync(process.execPath, [checker, '--staged'], { cwd: root, encoding: 'utf8' });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /require a live master/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('valid staged add, delete, rename, and no-op states are accepted', () => {
  const root = fixture();
  try {
    writeFixture(root);
    execFileSync('git', ['init', '-q'], { cwd: root });
    execFileSync('git', ['add', 'docs'], { cwd: root });
    execFileSync('git', ['-c', 'user.email=test@example.com', '-c', 'user.name=Test', 'commit', '-qm', 'fixture'], { cwd: root });

    writeFileSync(join(root, 'docs', 'roadmaps', '_extra-template.md'), '# template\n');
    execFileSync('git', ['add', 'docs/roadmaps/_extra-template.md'], { cwd: root });
    let result = spawnSync(process.execPath, [checker, '--staged'], { cwd: root, encoding: 'utf8' });
    assert.equal(result.status, 0, result.stderr);

    execFileSync('git', ['rm', '-q', 'docs/specs/roadmap/t01-f01-foundation.md'], { cwd: root });
    result = spawnSync(process.execPath, [checker, '--staged'], { cwd: root, encoding: 'utf8' });
    assert.equal(result.status, 0, result.stderr);

    execFileSync('git', ['mv', 'docs/roadmaps/_extra-template.md', 'docs/roadmaps/_renamed-template.md'], { cwd: root });
    result = spawnSync(process.execPath, [checker, '--staged'], { cwd: root, encoding: 'utf8' });
    assert.equal(result.status, 0, result.stderr);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('track dependency cycles and track/feature prefix mismatches are rejected', () => {
  const root = fixture();
  try {
    writeFixture(root, { trackStatus: 'Planned', specStatus: 'Planned' });
    const roadmap = join(root, 'docs', 'roadmap.md');
    writeFileSync(roadmap, readFileSync(roadmap, 'utf8')
      .replace('roadmaps/t01-core.md) — Depends on: None', 'roadmaps/t01-core.md) — Depends on: T02')
      .replace('## Final Success Criteria', '- [ ] **T02 — Later** — [Roadmap](roadmaps/t02-later.md) — Depends on: T01\n\n## Final Success Criteria'));
    writeFileSync(join(root, 'docs', 'roadmaps', 't02-later.md'), `# T02 — Later\n\n**Status**: Planned\n**Last updated**: 2026-09-01\n**Master**: [Program Roadmap](../roadmap.md)\n\n## Goal\n\nGoal.\n\n## Track Success Criteria\n\n- [ ] Track complete.\n\n## Executable Features\n\n- [ ] **T02.F01 — Later feature** — Depends on: None\n\n## Notes for AI Agents\n\nNotes.\n`);
    let result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /track dependency cycle/);

    writeFixture(root, { trackStatus: 'Planned', specStatus: 'Planned' });
    const track = join(root, 'docs', 'roadmaps', 't01-core.md');
    writeFileSync(track, readFileSync(track, 'utf8').replace('**T01.F01 — Foundation**', '**T02.F01 — Wrong prefix**'));
    result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /does not belong to T01/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('complete specs reject ordinary and nested unchecked tasks', () => {
  const root = fixture();
  try {
    writeFixture(root, { masterStatus: 'Complete', trackStatus: 'Complete', featureChecked: true, specStatus: 'Complete', specChecked: true });
    const spec = join(root, 'docs', 'specs', 'roadmap', 't01-f01-foundation.md');
    writeFileSync(spec, readFileSync(spec, 'utf8').replace('- [x] Implement.', '- [ ] Implement.'));
    let result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /unchecked implementation/);

    writeFixture(root, { masterStatus: 'Complete', trackStatus: 'Complete', featureChecked: true, specStatus: 'Complete', specChecked: true });
    writeFileSync(spec, readFileSync(spec, 'utf8').replace('## Verification', '  - [ ] Nested task\n\n## Verification'));
    result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /unchecked implementation/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('checked blocked features and invalid Planning or Active states are rejected', () => {
  const root = fixture();
  try {
    writeFixture(root, { featureChecked: true, specStatus: 'Blocked', blocker: 'Waiting on dependency.' });
    let result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Blocked feature cannot be checked|requires exactly one Complete spec/);

    writeFixture(root, { masterStatus: 'Planning', trackStatus: 'Complete', featureChecked: true, specStatus: 'Complete', specChecked: true });
    result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Planning master cannot/);

    writeFixture(root, { masterStatus: 'Active', trackStatus: 'Planned' });
    rmSync(join(root, 'docs', 'roadmaps', 't01-core.md'));
    rmSync(join(root, 'docs', 'specs', 'roadmap', 't01-f01-foundation.md'));
    writeFileSync(join(root, 'docs', 'roadmap.md'), readFileSync(join(root, 'docs', 'roadmap.md'), 'utf8').replace(/- \[ \] \*\*T01 — Core\*\*.*\n/, ''));
    result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Active master must have/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('duplicate required metadata and malformed IDs, paths, links, and dates are rejected', () => {
  const root = fixture();
  try {
    writeFixture(root);
    const master = join(root, 'docs', 'roadmap.md');
    writeFileSync(master, readFileSync(master, 'utf8').replace('**Status**: Active', '**Status**: Active\n**Status**: Planning'));
    const track = join(root, 'docs', 'roadmaps', 't01-core.md');
    writeFileSync(track, readFileSync(track, 'utf8').replace('**Master**: [Program Roadmap](../roadmap.md)', '**Master**: [Program Roadmap](../roadmap.md)\n**Master**: [Other](../roadmap.md)'));
    const spec = join(root, 'docs', 'specs', 'roadmap', 't01-f01-foundation.md');
    writeFileSync(spec, readFileSync(spec, 'utf8').replace('**Feature**: T01.F01', '**Feature**: T01.F01\n**Feature**: T01.F02').replace('**Last updated**: 2026-09-01', '**Last updated**: not-a-date'));
    const result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /duplicate \*\*Status\*\* metadata/);
    assert.match(result.stderr, /duplicate \*\*Master\*\* metadata/);
    assert.match(result.stderr, /duplicate \*\*Feature\*\* metadata/);
    assert.match(result.stderr, /invalid Last updated/);

    writeFixture(root);
    writeFileSync(join(root, 'docs', 'roadmap.md'), readFileSync(join(root, 'docs', 'roadmap.md'), 'utf8').replace('**T01 — Core**', '**T1 — Core**'));
    const malformed = run(root);
    assert.equal(malformed.status, 1);
    assert.match(malformed.stderr, /malformed track row/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('noncanonical live track and feature-spec filenames are rejected', () => {
  const root = fixture();
  try {
    writeFixture(root);
    const track = join(root, 'docs', 'roadmaps', 't01-core.md');
    const noncanonicalTrack = join(root, 'docs', 'roadmaps', 'T01-core.md');
    renameSync(track, noncanonicalTrack);
    let result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /docs\/roadmaps\/T01-core\.md: non-canonical track path/);

    writeFixture(root);
    const spec = join(root, 'docs', 'specs', 'roadmap', 't01-f01-foundation.md');
    const noncanonicalSpec = join(root, 'docs', 'specs', 'roadmap', 't01-F01-foundation.md');
    renameSync(spec, noncanonicalSpec);
    result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /docs\/specs\/roadmap\/t01-F01-foundation\.md: non-canonical feature-spec path/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('feature spec Track metadata must match the exact owning title and path', () => {
  const root = fixture();
  try {
    writeFixture(root);
    const spec = join(root, 'docs', 'specs', 'roadmap', 't01-f01-foundation.md');
    writeFileSync(spec, readFileSync(spec, 'utf8').replace(
      '**Track**: [T01 — Core](../../roadmaps/t01-core.md)',
      '**Track**: [T01 — Wrong title](../../roadmaps/t01-wrong.md)',
    ));
    const result = run(root);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /owning-track link must be exactly \[T01 — Core\]\(\.\.\/\.\.\/roadmaps\/t01-core\.md\)/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('usage errors use exit status 2', () => {
  const result = spawnSync(process.execPath, [checker, '--staged', '--root', '.'], { encoding: 'utf8' });
  assert.equal(result.status, 2);
  assert.match(result.stderr, /mutually exclusive/);
});

test('real goal prompt contains the approved execution contract', () => {
  const prompt = readFileSync(join(process.cwd(), 'docs', 'roadmaps', '_goal-prompt-template.md'), 'utf8');
  for (const clause of [
    'track roadmaps linked from `docs/roadmap.md`',
    'only unchecked `TNN.FNN`',
    '`gpt-5.6-sol` xhigh planner',
    '`gpt-5.6-sol` high reviewer',
    '`gpt-5.6-luna` high implementer',
    'Only a P1 finding blocks',
    'P2 and P3 findings are advisory',
    'one readiness revision',
    'one post-review remediation pass',
    'integration branch `roadmap/complete`',
    'dedicated\nintegration worktree',
    'exactly one feature branch and one feature worktree',
    'latest commit of the integration branch',
    'integrate completed feature commits into the integration branch in dependency',
    'Verify the feature worktree is clean after\nintegration, then clean and remove it',
    'This prompt explicitly authorizes local\nbranch, worktree, and commit creation',
    'Do not push, open',
    'merge into the user\'s `main` branch',
    'focused spec verification',
    'viability-first',
    'post-review `make check`',
    'atomically and truthfully',
    'atomic,\ntruthful spec/feature/track/master status/date/commit updates',
    'clean integration branch',
    'On the clean integration branch, run the final `make check`',
  ]) assert.match(prompt, new RegExp(clause.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')), clause);
});
