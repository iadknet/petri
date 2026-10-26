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

test('a grouped import spread over several lines is checked', () => withRoot((repository) => {
  source(repository, 'runtime/mesh.rs', 'use crate::{\n    contracts::Position,\n    simulation::tick::Tick,\n};\n');
  const result = run(repository);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /runtime\/mesh\.rs:1: runtime imports simulation \(simulation::tick\)/);
}));

test('nested import groups are checked', () => withRoot((repository) => {
  source(repository, 'runtime/mesh.rs', 'use crate::{contracts::Position, simulation::{tick::Tick, world::World}};\n');
  const result = run(repository);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /runtime imports simulation \(simulation::tick\)/);
  assert.match(result.stderr, /runtime imports simulation \(simulation::world\)/);
}));

test('super and self paths resolve against the importing module', () => withRoot((repository) => {
  source(repository, 'creature/genome/mod.rs', 'use super::state::CreatureState;\nuse self::vote::Vote;\n');
  source(repository, 'creature/genome/vote.rs', 'fn f() { super::super::state::CreatureState::new(); }\n');
  const result = run(repository);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /creature\/genome\/mod\.rs:1: genome imports creature \(creature::state\)/);
  assert.match(result.stderr, /creature\/genome\/vote\.rs:1: genome imports creature \(creature::state\)/);
  assert.doesNotMatch(result.stderr, /mod\.rs:2/);
}));

test('pub(super) visibility is not an import', () => withRoot((repository) => {
  source(repository, 'creature/genome/vote.rs', 'pub(super) fn f() {}\npub(in crate::creature::genome) fn g() {}\n');
  const result = run(repository);
  assert.equal(result.status, 0, result.stderr);
}));

test('an import of a module no domain owns fails instead of passing', () => withRoot((repository) => {
  source(repository, 'sensors/perception.rs', 'use crate::reexported::CreatureState;\n');
  const result = run(repository);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /sensors\/perception\.rs:1: sensors imports crate::reexported, which no domain in docs\/architecture\.md section 3 owns/);
}));

test('lib.rs holds only module declarations, attributes and comments', () => withRoot((repository) => {
  source(repository, 'lib.rs', '//! The crate.\n#![forbid(unsafe_code)]\npub mod sensors;\n#[cfg(test)]\nmod tests;\n\n// note\n');
  const clean = run(repository);
  assert.equal(clean.status, 0, clean.stderr);
  source(repository, 'lib.rs', 'pub mod sensors;\npub use crate::sensors::Snapshot;\npub fn helper() {}\n');
  const result = run(repository);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /lib\.rs:2: lib\.rs holds only module declarations/);
  assert.match(result.stderr, /lib\.rs:3: lib\.rs holds only module declarations/);
}));

test('no crate depends on v3-lab', () => withRoot((repository) => {
  write(repository, 'crates/v3-lab/Cargo.toml', '[package]\nname = "v3-lab"\n\n[lib]\nname = "v3-lab"\n');
  write(repository, 'crates/v3-cli/Cargo.toml', '[package]\nname = "v3-cli"\n\n[dependencies]\nv3-core = { path = "../v3-core" }\n');
  const clean = run(repository);
  assert.equal(clean.status, 0, clean.stderr);
  write(repository, 'crates/v3-cli/Cargo.toml', '[package]\nname = "v3-cli"\n\n[dev-dependencies]\nv3-lab = { path = "../v3-lab" }\n');
  const result = run(repository);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /crates\/v3-cli\/Cargo\.toml:5: depends on v3-lab, which nothing may depend on/);
}));

test("the checker's dependency order matches the document's section 3", () => withRoot((repository) => {
  const order = (levels) => `# Architecture\n\n## 3 Core domains\n\n${levels}\n\n## 5 Known debts\n\n| Debt | Today | Paid by |\n| --- | --- | --- |\n| G7 | energy | T30.F02 |\n`;
  const matching = [
    '1. `contracts`', '2. `config`, energy ledger (`ledger/`)', '3. world (`kernel`)', '4. `patterns`',
    '5. genome (`genome/`)', '6. `sensors`', '7. `mutation`, `runtime`', '8. `creature`', '9. `simulation`',
    '10. readers (`neighborhood/`)',
  ].join('\n');
  const clean = run(repository, { doc: order(matching) });
  assert.equal(clean.status, 0, clean.stderr);
  const swapped = matching.replace('6. `sensors`', '6. `patterns`').replace('4. `patterns`', '4. `sensors`');
  const result = run(repository, { doc: order(swapped) });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /section 3 places sensors at level 4; the checker has level 6/);
}));
