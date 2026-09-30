#!/usr/bin/env node
// T21.F05's statistic and run checks for scripts/telemetry-overhead. No
// random draw: every result is a function of its input.
//
//   telemetry-stats.mjs cell CEILING FILE [--subtract-flush]
//     FILE holds one pair per line, `TEL_MS REF_MS FLUSH_MS` (FLUSH_MS may be
//     `-`) or `void`; prints `n= k= median= lower= upper= verdict=`.
//   telemetry-stats.mjs spread FILE
//     FILE holds one reference wall time per line; prints `spread= n=`.
//   telemetry-stats.mjs check PRESET ENDPOINT EXPORT_CHECK(0|1) < STDERR
//     prints `status= self_time_us= flush_ms= failed= dropped= abandoned=
//     bytes= snapshots= traces= windows=` then the start line on its own line.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

// The spec's preset table (invariant 3), as the start line prints it.
export const PRESETS = {
  minimal: { interval_ms: '1000', tick_traces: 'off', windows: 'off', window_ticks: '8', window_interval_ms: '10000' },
  phases: { interval_ms: '1000', tick_traces: 'on', windows: 'off', window_ticks: '8', window_interval_ms: '10000' },
  standard: { interval_ms: '1000', tick_traces: 'on', windows: 'on', window_ticks: '8', window_interval_ms: '10000' },
  dense: { interval_ms: '250', tick_traces: 'on', windows: 'on', window_ticks: '16', window_interval_ms: '2000' },
};

// Least snapshots and windows a healthy run of at least 4 s exports per
// preset (spec, Export check row); a preset without a signal exports none.
const MIN_COUNTS = {
  minimal: { snapshots: 0, windows: 0 },
  phases: { snapshots: 0, windows: 0 },
  standard: { snapshots: 4, windows: 1 },
  dense: { snapshots: 12, windows: 2 },
};

const sorted = (values) => [...values].sort((a, b) => a - b);

export function median(values) {
  const v = sorted(values);
  if (v.length === 0) return NaN;
  const mid = Math.floor(v.length / 2);
  return v.length % 2 ? v[mid] : (v[mid - 1] + v[mid]) / 2;
}

// The largest k with P(Binomial(n, 1/2) <= k) <= 0.05, exactly; -1 when none.
export function intervalRank(n) {
  const total = 1n << BigInt(n);
  let k = -1;
  let cumulative = 0n;
  let choose = 1n;
  for (let i = 0; i <= n; i += 1) {
    cumulative += choose;
    if (cumulative * 20n > total) break;
    k = i;
    choose = (choose * BigInt(n - i)) / BigInt(i + 1);
  }
  return k;
}

// The median and its 90% distribution-free interval: the (k + 1)-th and
// (n - k)-th smallest ratios. Pass when the upper bound is below the ceiling,
// fail when the lower bound is at or above it, else inconclusive; strict and
// unrounded.
export function cellStats(ratios, ceiling) {
  const v = sorted(ratios);
  const n = v.length;
  const k = intervalRank(n);
  if (k < 0) return { n, k, median: median(v), lower: NaN, upper: NaN, verdict: 'inconclusive' };
  const lower = v[k];
  const upper = v[n - k - 1];
  let verdict = 'inconclusive';
  if (upper < ceiling) verdict = 'pass';
  else if (lower >= ceiling) verdict = 'fail';
  return { n, k, median: median(v), lower, upper, verdict };
}

// A pair line's ratio; `void` is infinite. With subtractFlush the telemetry
// run's flush_ms is taken off its wall time (state s).
export function pairRatio(line, { subtractFlush = false } = {}) {
  const [tel, ref, flush] = line.trim().split(/\s+/);
  if (tel === 'void') return Infinity;
  const wall = subtractFlush ? Number(tel) - Number(flush) : Number(tel);
  return wall / Number(ref);
}

// (max - min) / median.
export function spread(values) {
  const v = sorted(values);
  return (v[v.length - 1] - v[0]) / median(v);
}

// The pair table: 16 at a spread <= 5%, 24 at <= 10%, 32 at <= 20%, else 0.
export function pairCount(value) {
  if (value <= 0.05) return 16;
  if (value <= 0.1) return 24;
  if (value <= 0.2) return 32;
  return 0;
}

function fields(line, prefix) {
  const out = {};
  for (const part of line.slice(prefix.length).split(' ')) {
    const at = part.indexOf('=');
    if (at > 0) out[part.slice(0, at)] = part.slice(at + 1);
  }
  return out;
}

const COUNTS = ['self_time_us', 'flush_ms', 'failed', 'dropped', 'abandoned', 'bytes', 'snapshots', 'traces', 'windows'];

// A telemetry run's stderr against its preset and endpoint; with exportCheck
// (state a), any failed, dropped or abandoned record or counts inconsistent
// with the preset void it. Run lines are summed, flush_ms taken at its most.
export function checkRun(stderr, { preset, endpoint, exportCheck }) {
  const lines = stderr.split('\n');
  const start = lines.find((line) => line.startsWith('telemetry: on ')) ?? '';
  const totals = Object.fromEntries(COUNTS.map((name) => [name, 0]));
  let runs = 0;
  for (const line of lines.filter((l) => l.startsWith('telemetry: run='))) {
    runs += 1;
    const got = fields(line, 'telemetry: ');
    for (const name of COUNTS) {
      const value = Number(got[name] ?? 0);
      totals[name] = name === 'flush_ms' ? Math.max(totals[name], value) : totals[name] + value;
    }
  }
  const result = (status) => ({ status, start, ...totals });
  if (!start) return result('void:no-start-line');
  if (runs === 0) return result('void:no-run-line');
  const got = fields(start, 'telemetry: on ');
  const expected = { endpoint, preset, ...PRESETS[preset] };
  for (const [name, value] of Object.entries(expected)) {
    if (got[name] !== value) return result(`void:settings-${name}`);
  }
  if (exportCheck) {
    for (const name of ['failed', 'dropped', 'abandoned']) {
      if (totals[name] > 0) return result(`void:export-${name}`);
    }
    const { snapshots, traces, windows } = totals;
    const settings = PRESETS[preset];
    const least = MIN_COUNTS[preset];
    const consistent =
      (settings.tick_traces === 'off' ? traces === 0 : traces === snapshots) &&
      (settings.windows === 'off' ? windows === 0 : windows >= least.windows) &&
      snapshots >= least.snapshots;
    if (!consistent) return result('void:export-counts');
  }
  return result('ok');
}

const fixed = (value) => (Number.isFinite(value) ? value.toFixed(4) : Number.isNaN(value) ? '-' : 'inf');
const lines = (file) => readFileSync(file, 'utf8').split('\n').filter((line) => line.trim());

function main(argv) {
  const [command, ...rest] = argv;
  if (command === 'cell') {
    const [ceiling, file, flag] = rest;
    const ratios = lines(file).map((line) => pairRatio(line, { subtractFlush: flag === '--subtract-flush' }));
    const cell = cellStats(ratios, Number(ceiling));
    console.log(`n=${cell.n} k=${cell.k} median=${fixed(cell.median)} lower=${fixed(cell.lower)} upper=${fixed(cell.upper)} verdict=${cell.verdict}`);
  } else if (command === 'spread') {
    const value = spread(lines(rest[0]).map(Number));
    console.log(`spread=${fixed(value)} n=${pairCount(value)}`);
  } else if (command === 'check') {
    const [preset, endpoint, exportCheck] = rest;
    if (!PRESETS[preset]) throw new Error(`unknown preset ${preset}`);
    const run = checkRun(readFileSync(0, 'utf8'), { preset, endpoint, exportCheck: exportCheck === '1' });
    console.log([`status=${run.status}`, ...COUNTS.map((name) => `${name}=${run[name]}`)].join(' '));
    console.log(run.start);
  } else {
    console.error('Usage: telemetry-stats.mjs cell CEILING FILE [--subtract-flush] | spread FILE | check PRESET ENDPOINT 0|1');
    process.exit(2);
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2));
}
