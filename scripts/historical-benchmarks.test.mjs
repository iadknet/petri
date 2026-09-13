import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import test from 'node:test';
import { inventory, selectReports, verifyRaw, verifyHistory, verifyPackage, exportSummaries } from './historical-benchmarks.mjs';

const fixture = JSON.parse(readFileSync(new URL('../crates/v3-cli/tests/fixtures/synthetic-full-benchmark-v1.json', import.meta.url)));

function history(t) {
  const root = mkdtempSync(join(tmpdir(), 'petri historical artifacts '));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const repo = join(root, 'repo');
  mkdirSync(repo);
  const git = (...args) => execFileSync('git', args, { cwd: repo, encoding: 'utf8',
    env: { ...process.env, GIT_AUTHOR_DATE: '2026-09-01T00:00:00Z', GIT_COMMITTER_DATE: '2026-09-01T00:00:00Z' } }).trim();
  git('init', '-b', 'main');
  git('config', 'user.name', 'History fixture');
  git('config', 'user.email', 'history@example.invalid');
  const put = (path, bytes) => {
    mkdirSync(dirname(join(repo, path)), { recursive: true });
    writeFileSync(join(repo, path), bytes);
  };
  const commit = (message) => { git('add', '.'); git('commit', '-qm', message); return git('rev-parse', 'HEAD'); };
  const report = (path, time = '2026-09-01T00:00:00Z', extra = {}) => {
    const value = structuredClone(fixture);
    value.environment.generated_at = time;
    Object.assign(value, extra);
    put(path, JSON.stringify(value));
    return value;
  };
  const scan = () => inventory(repo, git('rev-parse', 'HEAD'), join(root, 'historical', 'package'));
  return { repo, git, put, commit, report, scan };
}

function accept(manifest) {
  for (const blob of manifest.blobs) blob.conversion = { status: 'converted' };
  selectReports(manifest);
  return manifest;
}

test('census preserves deleted versions, a rename and alias, and separates a rerun at a reused path', async (t) => {
  const h = history(t);
  h.put('keep.txt', 'unrelated\n');
  h.report('old report.data');
  const first = h.commit('first measurement');
  h.git('mv', 'old report.data', 'renamed.json');
  h.commit('rename');
  h.put('alias.json', readFileSync(join(h.repo, 'renamed.json')));
  h.commit('alias');
  h.git('rm', 'alias.json');
  h.report('renamed.json', '2026-09-02T00:00:00Z');
  h.commit('separate rerun');
  h.git('rm', 'renamed.json');
  h.put('renamed.json', '{"configuration":true}');
  h.commit('unrelated path reuse');
  const manifest = accept(await h.scan());
  assert.equal(manifest.enumeration.commits, 5);
  assert.equal(manifest.reports.length, 2);
  assert.equal(manifest.blobs.length, 2);
  const original = manifest.reports.find(r => r.paths.includes('old report.data'));
  assert.deepEqual(original.paths, ['alias.json', 'old report.data', 'renamed.json']);
  assert.ok(original.versions.some(v => v.first_commits.includes(first)));
  assert.ok(manifest.reports.every(r => r.historical_only && r.selected_blob));
  assert.ok(manifest.exclusions.some(e => e.paths.includes('renamed.json') && e.reason === 'unrelated JSON'));
  assert.equal(manifest.unresolved.length, 0);
  verifyRaw(manifest);
  assert.throws(() => verifyHistory(manifest, h.repo, h.git('rev-parse', 'HEAD')), /raw blob remains/);
});

test('a later invalid version keeps its preceding valid version and malformed bytes remain unresolved', async (t) => {
  const h = history(t);
  h.report('report.json');
  h.commit('valid');
  const good = h.git('rev-parse', 'HEAD:report.json');
  h.report('report.json', undefined, { schema_version: 999 });
  h.commit('unsupported correction');
  h.put('docs/progress/features/broken.json', '{"schema_version":1,');
  h.commit('malformed candidate');
  const manifest = await h.scan();
  for (const blob of manifest.blobs) blob.conversion = {
    status: blob.git_blob === good ? 'converted' : 'unsupported', error: 'fixture rejection',
  };
  selectReports(manifest);
  const report = manifest.reports.find(r => r.paths.includes('report.json'));
  assert.equal(report.selected_blob, good);
  assert.equal(report.versions.length, 2);
  assert.match(report.selection_reason, /final valid/);
  assert.ok(manifest.unresolved.some(u => /incomplete measured identity/.test(u.reason)));
  const broken = manifest.blobs.find(b => b.paths.some(p => p.endsWith('broken.json')));
  assert.equal(readFileSync(broken.raw.path, 'utf8'), '{"schema_version":1,');
  writeFileSync(broken.raw.path, 'changed');
  assert.throws(() => verifyRaw(manifest), /raw integrity/);
});

test('merged ancestry is fully enumerated and incomparable versions block selection', async (t) => {
  const h = history(t);
  h.report('report.json');
  h.commit('base');
  h.git('checkout', '-qb', 'side');
  h.report('report.json', undefined, { fixture_branch: 'side' });
  const side = h.commit('side correction');
  h.git('checkout', '-q', 'main');
  h.report('report.json', undefined, { fixture_branch: 'main' });
  h.commit('main correction');
  h.git('merge', '-s', 'ours', '--no-ff', '-qm', 'merge retaining main', 'side');
  h.git('rm', 'report.json');
  h.commit('delete report');
  const manifest = await h.scan();
  for (const blob of manifest.blobs) blob.conversion = { status: 'converted' };
  // A side version and the main version both precede the merge; its retained
  // file resolves the choice through ancestry even after the later deletion.
  selectReports(manifest);
  assert.equal(manifest.enumeration.commits, 5);
  assert.equal(manifest.enumeration.merges, 1);
  assert.ok(manifest.reports[0].versions.some(v => v.first_commits.includes(side)));
  assert.ok(manifest.reports[0].selected_blob);
  const ambiguous = history(t);
  ambiguous.report('report.json');
  ambiguous.commit('base');
  ambiguous.git('checkout', '-qb', 'side');
  ambiguous.report('report.json', undefined, { fixture_branch: 'side' });
  ambiguous.commit('side correction');
  ambiguous.git('checkout', '-q', 'main');
  ambiguous.report('report.json', undefined, { fixture_branch: 'main' });
  ambiguous.commit('main correction');
  ambiguous.git('rm', 'report.json');
  ambiguous.commit('delete before merge');
  ambiguous.git('merge', '-s', 'ours', '--no-ff', '-qm', 'merge without retaining either correction', 'side');
  const unresolved = accept(await ambiguous.scan());
  assert.equal(unresolved.reports[0].selected_blob, null);
  assert.ok(unresolved.unresolved.some(u => /incomparable/.test(u.reason)));
});

test('equal metadata at unrelated paths does not join reports, and fixtures and NDJSON are excluded explicitly', async (t) => {
  const h = history(t);
  h.report('one.json', undefined, { unrelated: 'one' });
  h.report('two.json', undefined, { unrelated: 'two' });
  h.put('crates/example/tests/fixtures/report.json', JSON.stringify(fixture));
  h.put('docs/progress/sweeps/example/trace.ndjson', '{"protocol_version":"v3alpha1","event_type":"run_started"}\n{"tick":1,"population":2}\n');
  h.commit('unrelated reports and non-reports');
  const manifest = accept(await h.scan());
  assert.equal(manifest.reports.length, 2);
  assert.ok(manifest.exclusions.some(e => e.reason === 'test fixture'));
  assert.ok(manifest.exclusions.some(e => e.reason === 'simulation trace, not a benchmark report'));
  const census = JSON.parse(readFileSync(manifest.enumeration.census.path));
  assert.ok(census.trees[0].entries.some(e => e.type === 'tree' && e.path === 'crates/example/tests'));
  assert.equal(manifest.unresolved.length, 0);
});

test('independent mapped trees preserve unrelated bytes and modes and reject missing map entries', async (t) => {
  const source = history(t);
  source.report('docs/progress/features/run.json');
  source.put('keep.txt', 'first\n');
  const oldFirst = source.commit('first');
  source.put('keep.txt', 'second\n');
  const oldSecond = source.commit('second');
  const manifest = await source.scan();
  const candidate = history(t);
  candidate.put('keep.txt', 'first\n');
  const newFirst = candidate.commit('first');
  candidate.put('keep.txt', 'second\n');
  const newSecond = candidate.commit('second');
  const mapping = join(manifest.package_path, 'commit-map');
  writeFileSync(mapping, `old new\n${oldFirst} ${newFirst}\n${oldSecond} ${newSecond}\n`);
  assert.equal(verifyHistory(manifest, candidate.repo, newSecond, mapping).mapped_trees_checked, true);
  assert.throws(() => verifyHistory(manifest, candidate.repo, newFirst, mapping), /not reachable from candidate/);
  writeFileSync(mapping, `old new\n${oldSecond} ${newSecond}\n`);
  assert.throws(() => verifyHistory(manifest, candidate.repo, newSecond, mapping), /missing preserved commit/);
  candidate.git('update-index', '--chmod=+x', 'keep.txt');
  candidate.git('commit', '--amend', '--no-edit', '-q');
  const changed = candidate.git('rev-parse', 'HEAD');
  writeFileSync(mapping, `old new\n${oldFirst} ${newFirst}\n${oldSecond} ${changed}\n`);
  assert.throws(() => verifyHistory(manifest, candidate.repo, changed, mapping), /unrelated tree changed/);
});

test('an incomplete effective profile is not a resolved measurement identity', async (t) => {
  const h = history(t);
  h.report('report.json', undefined, { deterministic: { profile: {} } });
  h.commit('incomplete profile');
  const manifest = await h.scan();
  assert.equal(manifest.reports[0].identity, null);
  assert.ok(manifest.unresolved.some(u => /incomplete measured identity/.test(u.reason)));
});

test('a historical reference preserves an otherwise unrecognizable deleted report candidate', async (t) => {
  const h = history(t);
  h.put('unlabelled-output', 'partial bytes with no envelope');
  h.put('docs/progress/benchmark-series.json', JSON.stringify({ gate: { closed: ['unlabelled-output'] } }));
  h.commit('old report reference');
  h.git('rm', 'unlabelled-output');
  h.put('docs/progress/benchmark-series.json', '{}');
  h.commit('reference and report removed');
  const manifest = await h.scan();
  assert.equal(manifest.blobs.length, 1);
  assert.equal(readFileSync(manifest.blobs[0].raw.path, 'utf8'), 'partial bytes with no envelope');
  assert.ok(manifest.unresolved.some(u => /incomplete measured identity/.test(u.reason)));
  const references = JSON.parse(readFileSync(manifest.enumeration.report_references.path));
  assert.ok(references.some(r => r.path === 'unlabelled-output' && r.history_paths.includes('unlabelled-output')));
});

test('a selected report cannot verify without its exported summary', async (t) => {
  const h = history(t);
  h.report('report.json');
  h.commit('stored measurement');
  const manifest = accept(await h.scan());
  assert.throws(() => verifyPackage(manifest), /missing summary/);
});

test('a historical root-level series preserves its baseline and closed references after deletion', async (t) => {
  const h = history(t);
  h.put('unlabelled-baseline', 'partial baseline bytes');
  h.put('unlabelled-closed', 'partial closed bytes');
  h.put('docs/progress/benchmark-series.json', JSON.stringify({
    epoch_baseline: 'unlabelled-baseline', closed: ['unlabelled-closed'],
  }));
  h.commit('old root-level series');
  h.git('rm', 'unlabelled-baseline', 'unlabelled-closed');
  h.put('docs/progress/benchmark-series.json', '{}');
  h.commit('old references and reports removed');

  const manifest = await h.scan();

  const references = JSON.parse(readFileSync(manifest.enumeration.report_references.path));
  assert.deepEqual(references.map(r => [r.pointer, r.path]), [
    ['/epoch_baseline', 'unlabelled-baseline'], ['/closed/0', 'unlabelled-closed'],
  ]);
  assert.equal(new Set(references.map(r => r.source_blob)).size, 1);
  assert.ok(references.every(r => r.occurrences.length === 1));
  assert.deepEqual(manifest.blobs.map(b => readFileSync(b.raw.path, 'utf8')).sort(),
    ['partial baseline bytes', 'partial closed bytes']);
  assert.equal(manifest.reports.length, 2);
  assert.ok(manifest.reports.every(r => r.historical_only && r.selected_blob === null));
  assert.equal(manifest.unresolved.length, 2);
});

test('unsupported raw-only reports need an explicit historical disposition', async (t) => {
  const h = history(t);
  h.report('report.json', undefined, { schema_version: 999 });
  h.commit('unsupported stored shape');
  const manifest = await h.scan();
  manifest.blobs[0].conversion = { status: 'unsupported', error: 'unsupported full report version 999' };
  selectReports(manifest);
  assert.throws(() => verifyPackage(manifest), /raw-only disposition/);
  manifest.reports[0].raw_only_disposition = {
    status: 'unsupported', reason: 'The stored fixture declares unsupported schema 999.', sources: ['fixture source commit'],
  };
  assert.equal(verifyPackage(manifest).verified_summaries, 0);
  assert.throws(() => exportSummaries(manifest), /main checkout/);
  h.git('checkout', '-qb', 'codex/t15-f02');
  h.put('report.json', 'user edit');
  assert.throws(() => exportSummaries(manifest), /checkout file changed/);
  assert.equal(readFileSync(join(h.repo, 'report.json'), 'utf8'), 'user edit');
  h.git('restore', 'report.json');
  const result = exportSummaries(manifest);
  assert.deepEqual(result.removed_raw_paths, ['report.json']);
  assert.equal(readFileSync(manifest.blobs[0].raw.path, 'utf8').includes('999'), true);
  assert.equal(JSON.parse(readFileSync(join(h.repo, 'docs/progress/historical-benchmark-manifest.json'))).reports[0].raw_only_disposition.status, 'unsupported');
});
