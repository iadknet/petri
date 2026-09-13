#!/usr/bin/env node
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { chmodSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import test from 'node:test';
import vm from 'node:vm';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const html = readFileSync(join(root, 'docs/progress/index.html'), 'utf8');

function page(reports) {
  const script = html.match(/<script>([\s\S]*?)<\/script>/)[1];
  const context = vm.createContext({
    fetch: async (path) => {
      assert.ok(Object.hasOwn(reports, path), `Unexpected fetch: ${path}`);
      return { ok: true, json: async () => reports[path] };
    },
  });
  vm.runInContext(script.slice(0, script.indexOf('  // ── Boot')) +
    'globalThis.api = {loadAll, meshSums, artifactNotice, num, get};})();', context);
  return context.api;
}

test('progress page reads mixed artifacts and retained mesh totals without opening raw', async () => {
  const full = {
    feature: 't01-f01', environment: {generated_at:'2026-09-01', git_revision:'abc'},
    deterministic: {profile:{name:'goal-v1'}, per_creature_tick:{vm_steps:'1.234567'}, goal_indicators:{
      population_persistence:{per_seed:[{seed:11,final_population:0}]},
      mutational_neighborhood:{evolved:{per_seed:[{seed:11,sampled_genomes:[{mesh_execution:{
        total_node_count:4,reachable_node_count:3,executed_node_count:2,knockout_count:1,
        route_varies_with_input:true,hop_cap_hits:0,executions_per_genome:80,
      }}]}]}},
    }},
  };
  const summary = structuredClone(full);
  summary.kind = 'petri-benchmark-summary';
  summary.summary_version = 1;
  summary.feature = 't01-f02';
  summary.raw = {path:'/never-fetch/local/raw.json', availability:'verified_local', verified_at:'2026-09-13T10:00:00Z'};
  summary.omitted_details = ['per-proposal rows', 'genomes'];
  summary.deterministic.goal_indicators.mutational_neighborhood.evolved.per_seed = [{seed:11,
    mesh_summary:{genomes:1,total:4,reachable:3,executed:2,knockout:1,routeVaries:1,capHits:0,execs:80}}];
  const api = page({
    'benchmark-series.json': {goal:{closed:['docs/progress/features/full.json','docs/progress/features/summary.json']}},
    'features/full.json': full, 'features/summary.json': summary,
  });
  const loaded = await api.loadAll();
  assert.equal(loaded.reportCount, 2);
  assert.equal(JSON.stringify(api.meshSums(full)), JSON.stringify(api.meshSums(summary)));
  assert.equal(api.get(summary, 'deterministic.per_creature_tick.vm_steps'), '1.234567');
  assert.equal(api.num(api.get(summary, 'deterministic.goal_indicators.population_persistence.per_seed.0.final_population')), 0);
  assert.match(api.artifactNotice(summary), /unavailable/i);
  assert.match(api.artifactNotice(summary), /2026-09-13T10:00:00Z/);
  summary.deterministic.goal_indicators.mutational_neighborhood.evolved.per_seed[0].mesh_summary = null;
  assert.equal(api.meshSums(summary), null);
});

test('progress page rejects unknown summary versions instead of treating them as missing reports', async () => {
  const api = page({
    'benchmark-series.json': {gate:{closed:['docs/progress/features/unknown.json']}},
    'features/unknown.json': {kind:'petri-benchmark-summary', summary_version:999},
  });
  await assert.rejects(api.loadAll(), /summary version/);
});

test('make forwards distinct optional paths with spaces through one existing preflight', () => {
  const dir = mkdtempSync(join(tmpdir(), 'petri make artifact '));
  try {
    mkdirSync(join(dir, 'scripts'));
    const wrapper = join(dir, 'scripts/bench-wait');
    writeFileSync(wrapper, '#!/bin/sh\nprintf "%s\\n" "$@"\n');
    chmodSync(wrapper, 0o755);
    const run = (...args) => execFileSync('make', ['--no-print-directory', '-f',join(root,'Makefile'),'bench',...args], {cwd:dir,encoding:'utf8'}).trim().split('\n');
    const args = run('PROFILE=goal','FEATURE=test-feature','OUT=raw dir/full.json','SUMMARY_OUT=summary dir/summary.json','BENCH_ARGS=--threads 1');
    assert.deepEqual(args, ['cargo','run','--release','-p','v3-cli','--','bench','--profile','goal','--feature','test-feature','--out','raw dir/full.json','--summary-out','summary dir/summary.json','--threads','1']);
    const defaults = run('PROFILE=sweep','FEATURE=test-feature','BENCH_ARGS=--width 8');
    assert.ok(!defaults.includes('--out'));
    assert.ok(defaults.includes('--feature'));
    const unlabelled = run('PROFILE=sweep','OUT=raw.json');
    assert.ok(!unlabelled.includes('--feature'));
    assert.throws(() => run('PROFILE=gate'), /FEATURE is required/);
    assert.throws(() => run('PROFILE=sweep'), /FEATURE or OUT is required/);
  } finally {
    rmSync(dir, {recursive:true,force:true});
  }
});
