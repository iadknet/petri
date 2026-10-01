import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import {
  calibrationStep,
  cellStats,
  chargeWait,
  checkRun,
  flushVerdict,
  intervalRank,
  median,
  pairCount,
  pairRatio,
  PRESETS,
  spread,
} from './telemetry-stats.mjs';

const script = fileURLToPath(new URL('./telemetry-stats.mjs', import.meta.url));
const range = (n, f) => Array.from({ length: n }, (_, i) => f(i));

test('median of odd and even samples, unsorted, with an infinite value', () => {
  assert.equal(median([3, 1, 2]), 2);
  assert.equal(median([4, 1, 3, 2]), 2.5);
  assert.equal(median([1, Infinity, 2]), 2);
  assert.equal(median([1, 2, Infinity, Infinity]), Infinity);
  assert.ok(Number.isNaN(median([])));
});

test('interval rank k is the largest with P(Binomial(n, 1/2) <= k) <= 0.05', () => {
  assert.equal(intervalRank(16), 4);
  assert.equal(intervalRank(24), 7);
  assert.equal(intervalRank(32), 10);
  // n = 6: P(X <= 0) = 1/64 <= 0.05 < P(X <= 1) = 7/64.
  assert.equal(intervalRank(6), 0);
  // n = 4: P(X <= 0) = 1/16 > 0.05, so no interval exists.
  assert.equal(intervalRank(4), -1);
  assert.equal(intervalRank(0), -1);
});

test('the interval bounds are the (k + 1)-th smallest and (n - k)-th smallest ratios', () => {
  const ratios = range(16, (i) => (100 + i) / 100).reverse();
  const cell = cellStats(ratios, 1.05);
  assert.equal(cell.n, 16);
  assert.equal(cell.k, 4);
  assert.equal(cell.lower, 1.04);
  assert.equal(cell.upper, 1.11);
  assert.equal(cell.median, (1.07 + 1.08) / 2);
  assert.equal(cell.verdict, 'inconclusive');
});

test('verdict: pass below, fail at or above, inconclusive across the ceiling', () => {
  const at = (value) => range(16, () => value);
  assert.equal(cellStats(at(1.0), 1.05).verdict, 'pass');
  assert.equal(cellStats(at(1.1), 1.05).verdict, 'fail');
  const straddle = [...range(8, () => 1.0), ...range(8, () => 1.1)];
  assert.equal(cellStats(straddle, 1.05).verdict, 'inconclusive');
});

test('the routine ceiling is 1.10 and compares strictly at the boundary', () => {
  const at = (value) => range(16, () => value);
  const overhead = readFileSync(new URL('./telemetry-overhead', import.meta.url), 'utf8');
  const ceiling = Number(overhead.match(/^ceiling=(\S+)$/m)?.[1]);
  assert.equal(ceiling, 1.1);
  // Strict: an upper bound of exactly the ceiling does not pass, and a lower
  // bound of exactly the ceiling fails.
  assert.equal(cellStats(at(1.1), ceiling).verdict, 'fail');
  const upperAt = [...range(11, () => 1.0), ...range(5, () => 1.1)];
  assert.equal(cellStats(upperAt, ceiling).upper, 1.1);
  assert.equal(cellStats(upperAt, ceiling).verdict, 'inconclusive');
  // Unrounded: 1.09999 passes although it prints as 1.1000.
  assert.equal(cellStats(at(1.09999), ceiling).verdict, 'pass');
});

test('a voided pair enters as an infinite ratio and only moves towards fail', () => {
  const clean = range(16, () => 1.0);
  const voided = [...range(12, () => 1.0), ...range(4, () => Infinity)];
  assert.equal(cellStats(clean, 1.05).verdict, 'pass');
  assert.equal(cellStats(voided, 1.05).n, 16);
  assert.equal(cellStats(voided, 1.05).upper, 1.0);
  assert.equal(cellStats(voided, 1.05).verdict, 'pass');
  const more = [...range(11, () => 1.0), ...range(5, () => Infinity)];
  assert.equal(cellStats(more, 1.05).upper, Infinity);
  assert.equal(cellStats(more, 1.05).verdict, 'inconclusive');
});

test('too few pairs for an interval, or none, is inconclusive', () => {
  assert.equal(cellStats([1, 1, 1, 1], 1.05).verdict, 'inconclusive');
  assert.equal(cellStats([], 1.05).verdict, 'inconclusive');
  // n = 6 gives the range as its interval; a dense preset with every pair
  // over the ceiling reads fail ("exceeds the ceiling; opt-in").
  assert.equal(cellStats(range(6, () => 1.2), 1.05).verdict, 'fail');
});

test('pair ratio: telemetry over reference, flush subtracted on request, void infinite', () => {
  assert.equal(pairRatio('5000 4000 -'), 1.25);
  assert.equal(pairRatio('15000 5000 10000'), 3);
  assert.equal(pairRatio('15000 5000 10000', { subtractFlush: true }), 1);
  assert.equal(pairRatio('void'), Infinity);
  assert.equal(pairRatio('void', { subtractFlush: true }), Infinity);
});

test('spread and the pair table', () => {
  assert.equal(spread([100, 104, 102, 101, 103]), 4 / 102);
  assert.equal(pairCount(0.05), 16);
  assert.equal(pairCount(0.0501), 24);
  assert.equal(pairCount(0.1), 24);
  assert.equal(pairCount(0.2), 32);
  assert.equal(pairCount(0.2001), 0);
});

const endpoint = 'http://127.0.0.1:4318';
const startLine = (name, overrides = {}) => {
  const p = { ...PRESETS[name], ...overrides };
  return `telemetry: on endpoint=${overrides.endpoint ?? endpoint} preset=${name} interval_ms=${p.interval_ms} tick_traces=${p.tick_traces} windows=${p.windows} window_ticks=${p.window_ticks} window_interval_ms=${p.window_interval_ms} invocation=ab run=cd`;
};
const runLine = (fields) => {
  const all = { exported: 9, failed: 0, dropped: 0, abandoned: 0, bytes: 1000, self_time_us: 50, flush_ms: 7, snapshots: 5, traces: 5, windows: 1, recorded: 8, ...fields };
  return `telemetry: run=cd ${Object.entries(all).map(([k, v]) => `${k}=${v}`).join(' ')}`;
};

test('the preset table is the spec table', () => {
  assert.deepEqual(Object.keys(PRESETS), ['minimal', 'phases', 'standard', 'dense']);
  assert.deepEqual(PRESETS.minimal, { interval_ms: '1000', tick_traces: 'off', windows: 'off', window_ticks: '8', window_interval_ms: '10000' });
  assert.deepEqual(PRESETS.dense, { interval_ms: '250', tick_traces: 'on', windows: 'on', window_ticks: '16', window_interval_ms: '2000' });
});

test('a run whose start line disagrees with its preset or endpoint is voided', () => {
  const ok = checkRun(`${startLine('standard')}\n${runLine({})}\n`, { preset: 'standard', endpoint, exportCheck: false });
  assert.equal(ok.status, 'ok');
  assert.equal(ok.self_time_us, 50);
  assert.equal(ok.flush_ms, 7);
  assert.equal(ok.start, startLine('standard'));
  for (const stderr of [
    `${startLine('standard', { windows: 'off' })}\n${runLine({})}`,
    `${startLine('phases')}\n${runLine({})}`,
    `${startLine('standard', { endpoint: 'http://127.0.0.1:9' })}\n${runLine({})}`,
    runLine({}),
    startLine('standard'),
  ]) {
    assert.match(checkRun(stderr, { preset: 'standard', endpoint, exportCheck: false }).status, /^void:/, stderr);
  }
});

test('the export check voids failures and counts inconsistent with the preset', () => {
  const check = (preset, fields) =>
    checkRun(`${startLine(preset)}\n${runLine(fields)}`, { preset, endpoint, exportCheck: true }).status;
  assert.equal(check('standard', {}), 'ok');
  assert.equal(check('dense', { snapshots: 20, traces: 20, windows: 3 }), 'ok');
  assert.equal(check('phases', { windows: 0 }), 'ok');
  assert.equal(check('minimal', { traces: 0, windows: 0 }), 'ok');
  for (const [preset, fields] of [
    ['standard', { failed: 1 }],
    ['standard', { dropped: 1 }],
    ['standard', { abandoned: 1 }],
    ['standard', { snapshots: 3, traces: 3 }],
    ['standard', { traces: 4 }],
    ['standard', { windows: 0 }],
    ['phases', { windows: 1 }],
    ['minimal', { traces: 1, windows: 0 }],
    ['minimal', { traces: 0, windows: 1 }],
    // dense: 250 ms snapshots and 2 s windows over a run of at least 4 s.
    ['dense', { snapshots: 11, traces: 11, windows: 3 }],
    ['dense', { snapshots: 20, traces: 20, windows: 1 }],
    ['dense', { snapshots: 20, traces: 19, windows: 3 }],
  ]) {
    assert.match(check(preset, fields), /^void:/, `${preset} ${JSON.stringify(fields)}`);
  }
  // Without the export check (states b and s) a failed export is expected.
  assert.equal(
    checkRun(`${startLine('standard')}\n${runLine({ failed: 9, abandoned: 3 })}`, { preset: 'standard', endpoint, exportCheck: false }).status,
    'ok',
  );
});

test('the command line reads pair lines and prints the cell', () => {
  const dir = mkdtempSync(join(tmpdir(), 'telemetry-stats-'));
  try {
    const file = join(dir, 'pairs');
    writeFileSync(file, `${range(15, () => '10000 10000 3000').join('\n')}\nvoid\n`);
    const cell = (...args) => execFileSync(process.execPath, [script, 'cell', '1.05', file, ...args], { encoding: 'utf8' }).trim();
    assert.equal(cell(), 'n=16 k=4 median=1.0000 lower=1.0000 upper=1.0000 verdict=pass');
    assert.equal(cell('--subtract-flush'), 'n=16 k=4 median=0.7000 lower=0.7000 upper=0.7000 verdict=pass');
    writeFileSync(file, '100\n104\n102\n101\n103\n');
    const out = execFileSync(process.execPath, [script, 'spread', file], { encoding: 'utf8' }).trim();
    assert.equal(out, 'spread=0.0392 n=16');
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('calibration: three scaling runs, then bisection, first in-band count is T1', () => {
  const rule = { start: 20, target: 5000, low: 4000, high: 6000, maxRuns: 8 };
  const step = (history) => calibrationStep(history, rule);
  assert.deepEqual(step([]), { next: 20 });
  // Attempt 1's history: 48 ticks at 5,939 ms is already in the band.
  assert.deepEqual(step([[20, 2083]]), { next: 48 });
  assert.deepEqual(step([[20, 2083], [48, 5939]]), { t1: 48 });
  assert.deepEqual(step([[20, 2083], [48, 5939], [40, 3895]]), { t1: 48 });
  // The band is inclusive at both ends.
  assert.deepEqual(step([[20, 1000], [100, 4000]]), { t1: 100 });
  assert.deepEqual(step([[20, 1000], [100, 6000]]), { t1: 100 });
  // Fewer than three runs keep scaling to the target even across the band.
  assert.deepEqual(step([[20, 1000], [100, 9000]]), { next: 56 });
  // Three runs all below: the last scales to the top of the band.
  assert.deepEqual(step([[20, 2083], [30, 3000], [40, 3895]]), { next: 62 });
  // Bracketed: bisect between the nearest counts below and above the band.
  assert.deepEqual(step([[20, 2083], [48, 6100], [40, 3895]]), { next: 44 });
  assert.deepEqual(step([[40, 3895], [62, 9500], [20, 2083]]), { next: 51 });
  assert.deepEqual(step([[40, 3895], [62, 9500], [20, 2083], [51, 7000]]), { next: 45 });
  assert.deepEqual(step([[40, 3895], [62, 9500], [20, 2083], [51, 3990]]), { next: 56 });
  // Three runs all above: no lower bracket to bisect from.
  assert.deepEqual(step([[20, 7000], [14, 6500], [11, 6200]]), { inconclusive: true });
  // A bisection count already measured means the bracket is exhausted.
  assert.deepEqual(step([[40, 3895], [41, 6500], [20, 2083]]), { inconclusive: true });
  // Eight runs outside the band.
  const eight = [[20, 1000], [100, 3000], [133, 3500], [200, 3900], [240, 3950], [290, 3990], [350, 3999], [420, 3999]];
  assert.deepEqual(step(eight.slice(0, 7)), { next: 525 });
  assert.deepEqual(step(eight), { inconclusive: true });
  // With maxRuns 3 (routine mode) there is no bisection.
  const routine = { start: 1000, target: 4000, low: 3000, high: 5000, maxRuns: 3 };
  assert.deepEqual(calibrationStep([[1000, 1000], [4000, 2900], [5517, 5100]], routine), { inconclusive: true });
});

test('the calibrate command prints the next count, T1 or inconclusive', () => {
  const dir = mkdtempSync(join(tmpdir(), 'telemetry-stats-'));
  try {
    const file = join(dir, 'history');
    const run = () => execFileSync(process.execPath, [script, 'calibrate', '20', '5000', '4000', '6000', '8', file], { encoding: 'utf8' }).trim();
    writeFileSync(file, '');
    assert.equal(run(), 'next=20');
    writeFileSync(file, '20 2083\n');
    assert.equal(run(), 'next=48');
    writeFileSync(file, '20 2083\n48 5939\n');
    assert.equal(run(), 't1=48');
    writeFileSync(file, '40 3895\n41 6500\n20 2083\n');
    assert.equal(run(), 'inconclusive');
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('the spread retry wait is charged, and is inconclusive when the cap cannot hold it', () => {
  assert.deepEqual(chargeWait(1000, 3600000, 60000), { usedMs: 61000 });
  assert.deepEqual(chargeWait(3540000, 3600000, 60000), { inconclusive: true });
  assert.deepEqual(chargeWait(3550000, 3600000, 60000), { inconclusive: true });
  assert.deepEqual(chargeWait(3539999, 3600000, 60000), { usedMs: 3599999 });
});

test('a flush above the bound fails the cell; at the bound or unread it does not', () => {
  assert.equal(flushVerdict('pass', '10050', 10050), 'pass');
  assert.equal(flushVerdict('pass', '10051', 10050), 'fail-flush-bound');
  assert.equal(flushVerdict('inconclusive', '10051', 10050), 'fail-flush-bound');
  assert.equal(flushVerdict('pass', '', 10050), 'pass');
  assert.equal(flushVerdict('inconclusive', '-', 10050), 'inconclusive');
});

test('the wait and flush commands print the charge and the bounded verdict', () => {
  const run = (...args) => execFileSync(process.execPath, [script, ...args], { encoding: 'utf8' }).trim();
  assert.equal(run('wait', '1000', '3600000', '60000'), 'used=61000');
  assert.equal(run('wait', '3540000', '3600000', '60000'), 'inconclusive');
  assert.equal(run('flush', 'pass', '10011', '10050'), 'pass');
  assert.equal(run('flush', 'pass', '10051', '10050'), 'fail-flush-bound');
  assert.equal(run('flush', 'pass', '', '10050'), 'pass');
});
