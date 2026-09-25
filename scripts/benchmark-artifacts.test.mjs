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
    'globalThis.api = {loadAll, meshSums, artifactNotice, num, get, mutationEffectsView};})();', context);
  return context.api;
}

const COUNT_FIELDS = ['proposals', 'skipped', 'action_changed', 'action_dead', 'genome_identical',
  'unexecuted_edit', 'masked_before_selection', 'state_or_cost_only', 'unresolved',
  'silent_with_state_or_cost', 'consistency_violations'];

function effectsBlock() {
  const cohort = (totals, parents, requested) => ({
    cohort: 'drift', identity: 'fixture cohort', parents_requested: requested, parents_evaluated: parents.length,
    totals, parents,
    operators: [{ key: 'Graph.AlterGraphEdgeWeight', counts: totals }],
    targets: [{ key: 'knockout_contributing', counts: totals }],
    coverage: { parents_evaluated: parents.length, pairs_requested: 20, pairs_sampled: 8, differ_recorded: 2,
      differ_authored: 1, differ_sequence_ticks_1_4: 0, differ_sequence_ticks_5_32: 4, differ_any: 5,
      state_or_cost_only: 1, all_noop_parents: 1, all_noop_parents_acting: 1 },
  });
  const parent = (depth, size) => ({ index: 0, depth_or_generation: depth, genome_size: size, total_nodes: 4,
    reachable_nodes: 3, executed_nodes: 2, contributing_nodes: 1, all_noop: false, distinct_queues: 3,
    counts: [10, 2, 1, 0, 1, 2, 2, 1, 1, 3, 0] });
  return {
    version: 'mutation-effects-v1', battery_version: 'neighborhood-v1', count_fields: COUNT_FIELDS, queue_buckets: ['1', '2', '3-4', '5-8', '9+'],
    exposure: [{ panel: 'drift@2000', supply: 'pinned 97 units (drift chart)', parents: 20, parents_all_noop: 5,
      parents_one_queue: 6, distinct_queues_histogram: [6, 4, 4, 4, 2], births_total: 200, zero_requested: 50,
      requested_all_skipped: 10, event_bearing: 140, requested_events_total: 300, applied_events_total: 240,
      genome_identical: 14, from_actionless: { silent: 30, changed: 0, dead: 0, zero_applied: 20 },
      from_acting: { silent: 70, changed: 30, dead: 10, zero_applied: 40 } }],
    cohorts: [
      cohort([100, 20, 10, 2, 8, 20, 20, 10, 10, 25, 0], [parent(0, 97)], 1),
      cohort([200, 0, 20, 4, 16, 40, 40, 40, 40, 50, 1], [parent(2000, 300), parent(2000, 100)], 20),
      'extinct: no living creature at the terminal tick',
    ],
    coverage: { version: 'neighborhood-coverage-v1', recorded: { requested: 32, actual: 32 }, sequence_source: 'recorded',
      controls: [{ name: 'same_genome', passed: true, differing_groups: [] },
        { name: 'slow_integrator_after_tick_4', passed: false, differing_groups: ['sequence_ticks_5_32'] }] },
  };
}

test('mutation-effects view shows counts and ratios over their stored denominators', () => {
  const api = page({});
  const view = api.mutationEffectsView(effectsBlock());
  assert.equal(view.status, 'measured');
  assert.equal(view.batteryVersion, 'neighborhood-v1');
  const [row] = view.exposure;
  assert.equal(row.requestedPerBirth, '1.5');
  assert.equal(row.appliedPerBirth, '1.2');
  assert.equal(row.zeroApplied, '60 of 200 (30%)');
  assert.equal(row.identical, '14 of 140 (10%)');
  assert.equal(row.allNoop, '5 of 20 (25%)');
  assert.equal(row.oneQueue, '6 of 20 (30%)');
  assert.equal(row.fromActing, '150 of 200 (75%)');
  const [founder, drift, selected] = view.cohorts;
  assert.equal(founder.applied, '80');
  assert.deepEqual(founder.categories.map((c) => c[0]), COUNT_FIELDS.slice(2, 9));
  assert.equal(founder.categories[0][1], '10 of 80 (12.5%)');
  assert.equal(founder.unresolved, '10 of 80 (12.5%)');
  assert.equal(founder.actionEffects, '12 of 80 (15%)');
  assert.equal(founder.silentWithStateOrCost, '25 of 68 (36.76%)');
  assert.equal(drift.parents, '2 of 20 requested');
  assert.equal(drift.meanDepthOrGeneration, '2,000');
  assert.equal(drift.meanGenomeSize, '200');
  assert.equal(drift.consistencyViolations, '1');
  assert.equal(drift.coverage.pairs, '8 of 20 requested');
  assert.equal(drift.coverage.recorded, '2 of 8 (25%)');
  assert.equal(drift.coverage.late, '4 of 8 (50%)');
  assert.equal(drift.coverage.any, '5 of 8 (62.5%)');
  assert.equal(drift.coverage.noopActing, '1 of 1 (100%)');
  assert.deepEqual(Array.from(drift.operators[0]).slice(0, 4), ['Graph.AlterGraphEdgeWeight', '200', '0', '200']);
  assert.equal(selected.status, 'undefined: extinct: no living creature at the terminal tick');
  assert.equal(view.recorded, '32 of 32 requested');
  assert.deepEqual(Array.from(view.controls[1]), ['slow_integrator_after_tick_4', 'failed', 'sequence_ticks_5_32']);
});

test('mutation-effects view reads historical, zero-denominator and short-sample blocks honestly', () => {
  const api = page({});
  assert.equal(api.mutationEffectsView(undefined).status, 'not measured');
  assert.equal(api.mutationEffectsView('Undefined').status, 'undefined: Undefined');

  const zero = effectsBlock();
  zero.exposure[0] = { ...zero.exposure[0], births_total: 0, event_bearing: 0, parents: 0, requested_events_total: 0,
    applied_events_total: 0, zero_requested: 0, requested_all_skipped: 0, genome_identical: 0, parents_all_noop: 0,
    from_actionless: { silent: 0, changed: 0, dead: 0, zero_applied: 0 }, from_acting: { silent: 0, changed: 0, dead: 0, zero_applied: 0 } };
  zero.cohorts[0].totals = [5, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  zero.cohorts[0].coverage = { ...zero.cohorts[0].coverage, pairs_sampled: 0, differ_recorded: null, all_noop_parents: 0 };
  const view = api.mutationEffectsView(zero);
  assert.equal(view.exposure[0].requestedPerBirth, 'undefined');
  assert.equal(view.exposure[0].zeroApplied, 'undefined');
  assert.equal(view.exposure[0].allNoop, 'undefined');
  const founder = view.cohorts[0];
  assert.ok(founder.categories.every((c) => c[1] === 'undefined'));
  assert.equal(founder.silentWithStateOrCost, 'undefined');
  assert.equal(founder.coverage.recorded, 'undefined');
  assert.equal(founder.coverage.any, 'undefined');
  assert.equal(founder.coverage.noopActing, 'undefined');

  const short = effectsBlock();
  short.coverage.recorded = { requested: 32, actual: 2 };
  short.coverage.sequence_source = 'authored';
  short.cohorts[1].parents_evaluated = 1;
  short.cohorts[1].parents = short.cohorts[1].parents.slice(0, 1);
  const shortView = api.mutationEffectsView(short);
  assert.equal(shortView.recorded, '2 of 32 requested');
  assert.equal(shortView.sequenceSource, 'authored');
  assert.equal(shortView.cohorts[1].parents, '1 of 20 requested');
  const extinct = effectsBlock();
  extinct.coverage.recorded = 'extinct: no living creature at the terminal tick';
  assert.equal(api.mutationEffectsView(extinct).recorded, 'undefined: extinct: no living creature at the terminal tick');
});

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
  summary.summary_version = 2;
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
  assert.match(api.artifactNotice(summary), /Detailed observations are omitted; see `omitted_details` in the summary\./);
  assert.doesNotMatch(api.artifactNotice(summary), /per-proposal rows/, 'the notice does not list every omitted path');
  assert.match(api.artifactNotice(summary), /not a download or a current availability guarantee/);
  assert.match(api.artifactNotice(summary), /2026-09-13T10:00:00Z/);
  summary.deterministic.goal_indicators.mutational_neighborhood.evolved.per_seed[0].mesh_summary = null;
  assert.equal(api.meshSums(summary), null);
});

test('progress page rejects unknown summary versions instead of treating them as missing reports', async () => {
  for (const summary_version of [999, 1]) {
    const api = page({
      'benchmark-series.json': {gate:{closed:['docs/progress/features/unknown.json']}},
      'features/unknown.json': {kind:'petri-benchmark-summary', summary_version, deterministic:{}, environment:{}},
    });
    await assert.rejects(api.loadAll(), /summary version/);
  }
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
