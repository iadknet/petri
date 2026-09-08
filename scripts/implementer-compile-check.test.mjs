import assert from 'node:assert/strict';
import { chmodSync, existsSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import test from 'node:test';

const hook = join(process.cwd(), 'scripts', 'implementer-compile-check');

// A stand-in `cargo` on PATH records how it was invoked and behaves per
// CARGO_SHIM_MODE: pass (default), fail (41 lines then exit 101), or hang.
const shim = `#!/bin/sh
{ printf '%s\\n' "$*"; pwd; } >"$CARGO_SHIM_LOG"
case "\${CARGO_SHIM_MODE:-pass}" in
  fail)
    i=0
    while [ "$i" -lt 40 ]; do printf 'line %s\\n' "$i"; i=$((i + 1)); done
    printf 'error[E0308]: mismatched types\\n' >&2
    exit 101
    ;;
  hang) sleep 30; exit 0 ;;
  *) printf 'Finished\\n'; exit 0 ;;
esac
`;

function git(repository, ...args) {
  const result = spawnSync('git', ['-C', repository, ...args], { encoding: 'utf8' });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}

function repo() {
  // realpath: git reports the resolved root, and macOS puts tmpdir behind a symlink.
  const repository = realpathSync(mkdtempSync(join(tmpdir(), 'petri-compile-check.')));
  git(repository, 'init', '-q', '-b', 'main');
  mkdirSync(join(repository, 'bin'));
  writeFileSync(join(repository, 'bin', 'cargo'), shim);
  chmodSync(join(repository, 'bin', 'cargo'), 0o755);
  mkdirSync(join(repository, 'crates', 'core', 'src'), { recursive: true });
  writeFileSync(join(repository, 'Cargo.toml'), '[workspace]\nmembers = ["crates/core"]\n');
  writeFileSync(join(repository, 'crates', 'core', 'src', 'lib.rs'), 'pub fn one() -> u8 { 1 }\n');
  writeFileSync(join(repository, 'notes.md'), 'notes\n');
  return repository;
}

function run(repository, input, env = {}) {
  const shimLog = join(repository, 'cargo-shim.log');
  const result = spawnSync('sh', [hook], {
    input: typeof input === 'string' ? input : JSON.stringify(input),
    encoding: 'utf8',
    env: {
      ...process.env,
      PATH: `${join(repository, 'bin')}:${process.env.PATH}`,
      CARGO_SHIM_LOG: shimLog,
      ...env,
    },
  });
  result.shim = existsSync(shimLog) ? readFileSync(shimLog, 'utf8').split('\n') : null;
  return result;
}

function edit(repository, path, extra = {}) {
  return {
    hook_event_name: 'PostToolUse',
    tool_name: 'Edit',
    tool_input: { file_path: join(repository, path), old_string: 'a', new_string: 'b', ...extra },
    tool_response: { filePath: join(repository, path) },
  };
}

test('a Rust edit that compiles: exit 0 after cargo check in the repository root', () => {
  const repository = repo();
  try {
    const result = run(repository, edit(repository, 'crates/core/src/lib.rs'));
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stderr, '');
    assert.deepEqual(result.shim.slice(0, 2), ['check --workspace --all-targets', repository]);
  } finally { rmSync(repository, { recursive: true, force: true }); }
});

test('a Rust edit that fails to compile: exit 2 with the last 30 lines of output', () => {
  const repository = repo();
  try {
    const result = run(repository, edit(repository, 'crates/core/src/lib.rs'), { CARGO_SHIM_MODE: 'fail' });
    assert.equal(result.status, 2);
    assert.match(result.stderr, /implementer-compile-check: cargo check --workspace --all-targets failed \(exit 101\) after editing .*lib\.rs/);
    assert.match(result.stderr, /mismatched types/);
    assert.match(result.stderr, /^line 11$/m);
    assert.doesNotMatch(result.stderr, /^line 10$/m);
  } finally { rmSync(repository, { recursive: true, force: true }); }
});

test('a hanging cargo check is abandoned at the timeout with exit 2', () => {
  const repository = repo();
  try {
    const started = Date.now();
    const result = run(repository, edit(repository, 'crates/core/src/lib.rs'), {
      CARGO_SHIM_MODE: 'hang',
      PETRI_COMPILE_CHECK_TIMEOUT: '1',
    });
    assert.equal(result.status, 2);
    assert.match(result.stderr, /timed out after 1s/);
    assert.ok(Date.now() - started < 15000, 'hook returned promptly after the timeout');
  } finally { rmSync(repository, { recursive: true, force: true }); }
});

test('non-Rust edits never run cargo', () => {
  const repository = repo();
  try {
    const result = run(repository, edit(repository, 'notes.md'), { CARGO_SHIM_MODE: 'fail' });
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.shim, null);
  } finally { rmSync(repository, { recursive: true, force: true }); }
});

test('a "file_path" inside written content does not decide the file type', () => {
  const repository = repo();
  try {
    const decoy = JSON.stringify({ file_path: '/elsewhere/decoy.rs' });
    let result = run(repository, {
      hook_event_name: 'PostToolUse',
      tool_name: 'Write',
      tool_input: { file_path: join(repository, 'notes.md'), content: `${decoy}\n` },
    }, { CARGO_SHIM_MODE: 'fail' });
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.shim, null);

    result = run(repository, edit(repository, 'crates/core/src/lib.rs', { new_string: JSON.stringify({ file_path: 'x.md' }) }));
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.shim[0], 'check --workspace --all-targets');
  } finally { rmSync(repository, { recursive: true, force: true }); }
});

test('missing file_path, a file outside a Cargo repository, or a missing directory never blocks', () => {
  const repository = repo();
  try {
    assert.equal(run(repository, '{"hook_event_name":"PostToolUse"}', { CARGO_SHIM_MODE: 'fail' }).status, 0);
    assert.equal(run(repository, edit(repository, 'missing/dir/lib.rs'), { CARGO_SHIM_MODE: 'fail' }).status, 0);
    rmSync(join(repository, 'Cargo.toml'));
    const result = run(repository, edit(repository, 'crates/core/src/lib.rs'), { CARGO_SHIM_MODE: 'fail' });
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.shim, null);
  } finally { rmSync(repository, { recursive: true, force: true }); }
});

// Cooldown: fires are sequential (the harness waits for each hook), so a
// passing check suppresses re-runs for a short window. TMPDIR is pointed at
// the fixture so the marker is cleaned up with it.
function marker(repository) {
  const hash = spawnSync('sh', ['-c', 'printf %s "$1" | cksum | cut -d" " -f1', '_', repository], { encoding: 'utf8' }).stdout.trim();
  return join(repository, `petri-compile-check-${hash}.ok`);
}

function cooled(repository, env = {}) {
  rmSync(join(repository, 'cargo-shim.log'), { force: true });
  return run(repository, edit(repository, 'crates/core/src/lib.rs'), { TMPDIR: repository, ...env });
}

test('a passing check suppresses the next fire inside the cooldown', () => {
  const repository = repo();
  try {
    let result = cooled(repository);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.shim[0], 'check --workspace --all-targets');
    assert.ok(existsSync(marker(repository)), 'marker written after a pass');

    result = cooled(repository, { CARGO_SHIM_MODE: 'fail' });
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.shim, null, 'cargo not run inside the cooldown');
  } finally { rmSync(repository, { recursive: true, force: true }); }
});

test('a failing check clears the cooldown so the next fire runs', () => {
  const repository = repo();
  try {
    writeFileSync(marker(repository), `${Math.floor(Date.now() / 1000) - 100}\n`);
    let result = cooled(repository, { CARGO_SHIM_MODE: 'fail' });
    assert.equal(result.status, 2);
    assert.equal(result.shim[0], 'check --workspace --all-targets', 'an expired marker does not suppress');
    assert.ok(!existsSync(marker(repository)), 'marker removed after a failure');

    result = cooled(repository);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.shim[0], 'check --workspace --all-targets', 'runs again right after a failure');
  } finally { rmSync(repository, { recursive: true, force: true }); }
});

test('PETRI_COMPILE_CHECK_COOLDOWN=0 runs cargo on every fire', () => {
  const repository = repo();
  try {
    assert.equal(cooled(repository, { PETRI_COMPILE_CHECK_COOLDOWN: '0' }).status, 0);
    const result = cooled(repository, { PETRI_COMPILE_CHECK_COOLDOWN: '0', CARGO_SHIM_MODE: 'fail' });
    assert.equal(result.status, 2);
    assert.equal(result.shim[0], 'check --workspace --all-targets');
  } finally { rmSync(repository, { recursive: true, force: true }); }
});
