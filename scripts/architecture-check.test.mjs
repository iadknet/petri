import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import test from 'node:test';

const checker = join(process.cwd(), 'scripts', 'architecture-check.mjs');
const DOC = '# Architecture\n\n## 5 Known debts\n\n| Debt | Today | Paid by |\n| --- | --- | --- |\n| G7 | energy | T30.F02 |\n';

function root() {
  return mkdtempSync(join(tmpdir(), 'petri-architecture-check.'));
}

function write(repository, path, content) {
  const file = join(repository, path);
  mkdirSync(file.slice(0, file.lastIndexOf('/')), { recursive: true });
  writeFileSync(file, content);
}

function source(repository, path, content) {
  write(repository, `crates/v3-core/src/${path}`, content);
}

function run(repository, { debts = '', doc = DOC } = {}) {
  write(repository, 'docs/architecture.md', doc);
  write(repository, 'scripts/architecture-debts.txt', debts);
  return spawnSync(process.execPath, [checker, '--root', repository], { encoding: 'utf8' });
}

function withRoot(body) {
  const repository = root();
  try {
    body(repository);
  } finally {
    rmSync(repository, { recursive: true, force: true });
  }
}

test('downward imports pass', () => withRoot((repository) => {
  source(repository, 'simulation/tick.rs', 'use crate::runtime::mesh::run;\nuse crate::creature::genome::CreatureGenome;\n');
  source(repository, 'runtime/mesh.rs', 'use crate::sensors::perception::Snapshot;\nuse crate::contracts::Position;\n');
  source(repository, 'creature/genome/mod.rs', 'use crate::contracts::InputReference;\n');
  const result = run(repository);
  assert.equal(result.status, 0, result.stderr);
}));

test('an upward import fails with file, line and domains', () => withRoot((repository) => {
  source(repository, 'runtime/mesh.rs', 'use crate::contracts::Position;\nuse crate::simulation::energy_accounting::DeathCause;\n');
  const result = run(repository);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /runtime\/mesh\.rs:2: runtime imports simulation \(simulation::energy_accounting\)/);
}));

test('domains on one row never import each other', () => withRoot((repository) => {
  source(repository, 'mutation/topology/routing.rs', 'use crate::runtime::routing::resolve;\n');
  const result = run(repository);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /mutation imports runtime/);
}));

test('the named prohibitions hold even downward', () => withRoot((repository) => {
  source(repository, 'creature/genome/vote.rs', 'use crate::config::RuntimeConfig;\n');
  source(repository, 'sensors/typed_food.rs', 'use crate::creature::genome::CreatureGenome;\n');
  const result = run(repository);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /genome imports config/);
  assert.match(result.stderr, /sensors imports genome/);
}));

test('reader modules inside creature count as readers', () => withRoot((repository) => {
  source(repository, 'creature/state.rs', 'use crate::creature::sensor_census::census;\n');
  source(repository, 'creature/sensor_census.rs', 'use crate::creature::state::CreatureState;\n');
  const result = run(repository);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /creature\/state\.rs:1: creature imports readers/);
  assert.doesNotMatch(result.stderr, /sensor_census\.rs/);
}));

test('grouped imports are checked', () => withRoot((repository) => {
  source(repository, 'kernel/world.rs', 'use crate::{config::Config, creature::identity::splitmix64};\n');
  const result = run(repository);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /kernel imports creature \(creature::identity\)/);
  assert.doesNotMatch(result.stderr, /imports config/);
}));

test('test code and comments are ignored', () => withRoot((repository) => {
  source(repository, 'runtime/mesh.rs', [
    '//! See [`crate::simulation::Simulation`].',
    'use crate::contracts::Position; // not crate::simulation::x',
    '#[cfg(test)]',
    'mod tests {',
    '    use crate::simulation::run_tick;',
    '    fn f() { let _ = { 1 }; }',
    '}',
    '#[cfg(test)]',
    'mod pass_loop_tests;',
    'pub fn after() {}',
  ].join('\n'));
  source(repository, 'runtime/cycle_tests.rs', 'use crate::simulation::run_tick;\n');
  source(repository, 'runtime/tests/helpers.rs', 'use crate::simulation::run_tick;\n');
  source(repository, 'runtime/cgp/neutral_connection_tests/offspring.rs', 'use crate::mutation::x;\n');
  source(repository, 'runtime/test_support.rs', 'use crate::simulation::run_tick;\n');
  const result = run(repository);
  assert.equal(result.status, 0, result.stderr);
}));

test('code after a test module is still checked', () => withRoot((repository) => {
  source(repository, 'runtime/mod.rs', '#[cfg(test)]\nmod cycle_tests;\npub use crate::simulation::Simulation;\n');
  const result = run(repository);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /runtime\/mod\.rs:3: runtime imports simulation/);
}));

test('a listed debt passes; a stale or undocumented one fails', () => withRoot((repository) => {
  source(repository, 'runtime/mesh.rs', 'use crate::simulation::energy_accounting::DeathCause;\n');
  let result = run(repository, { debts: '# comment\nG7 runtime/mesh.rs simulation::energy_accounting\n' });
  assert.equal(result.status, 0, result.stderr);

  result = run(repository, { debts: 'G7 runtime/mesh.rs simulation::energy_accounting\nG7 runtime/vm.rs simulation::energy_accounting\n' });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /stale debt G7 runtime\/vm\.rs simulation::energy_accounting/);

  result = run(repository, { debts: 'G9 runtime/mesh.rs simulation::energy_accounting\n' });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /G9 is not a known debt in docs\/architecture\.md/);
}));

test('a listed debt covers only its module, not the whole domain', () => withRoot((repository) => {
  source(repository, 'runtime/mesh.rs', 'use crate::simulation::energy_accounting::DeathCause;\nuse crate::simulation::stats::Timings;\n');
  const result = run(repository, { debts: 'G7 runtime/mesh.rs simulation::energy_accounting\n' });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /mesh\.rs:2: runtime imports simulation \(simulation::stats\)/);
}));

test('an unknown top-level module fails until the architecture places it', () => withRoot((repository) => {
  source(repository, 'scratch/mod.rs', 'pub fn f() {}\n');
  const result = run(repository);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /scratch\/mod\.rs: module scratch has no domain/);
}));

test('a module its parent gates behind cfg(test) is skipped with its children', () => withRoot((repository) => {
  source(repository, 'runtime/mod.rs', '#[cfg(test)]\npub(crate) mod vote_support;\n#[cfg(test)]\nmod fixtures;\npub mod mesh;\n');
  source(repository, 'runtime/vote_support.rs', 'use crate::creature::state::GraphRuntimeState;\n');
  source(repository, 'runtime/fixtures/mod.rs', 'mod deep;\n');
  source(repository, 'runtime/fixtures/deep.rs', 'use crate::simulation::run_tick;\n');
  source(repository, 'runtime/mesh.rs', 'use crate::contracts::Position;\n');
  const result = run(repository);
  assert.equal(result.status, 0, result.stderr);
}));
