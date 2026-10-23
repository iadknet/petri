#!/usr/bin/env node

// Enforces docs/architecture.md section 3 on crates/v3-core: every `crate::`
// import points down the dependency order and breaks none of the named
// prohibitions. Imports the code still owes are listed in
// scripts/architecture-debts.txt under a debt ID from section 5; an entry the
// code no longer needs, or whose ID the document no longer lists, fails, so
// paying a debt deletes it from both places.

import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { join, relative, resolve } from 'node:path';

// Section 3's order, bottom up; a domain imports only lower levels.
const LEVELS = new Map([
  ['contracts', 1],
  ['config', 2],
  ['ledger', 2],
  ['kernel', 3],
  ['patterns', 4],
  ['genome', 5],
  ['sensors', 6],
  ['mutation', 7],
  ['runtime', 7],
  ['creature', 8],
  ['simulation', 9],
  ['readers', 10],
]);
// Readers that live inside `creature/` today.
const CREATURE_READERS = new Set(['plasticity_census', 'sensor_census', 'drives', 'ring_rules']);

function usage(message) {
  console.error(`architecture-check: ${message}`);
  console.error('Usage: scripts/architecture-check.mjs [--root DIR]');
  process.exitCode = 2;
}

// Domain of a module path given as segments under `crate::`.
function domainOf(segments) {
  const [first, second] = segments;
  if (first === 'creature' && second === 'genome') return 'genome';
  if (first === 'creature' && CREATURE_READERS.has(second)) return 'readers';
  if (first === 'neighborhood') return 'readers';
  return LEVELS.has(first) ? first : undefined;
}

function allowed(from, to) {
  if (from === to) return true;
  if (from === 'genome') return to === 'contracts';
  if (from === 'sensors' && to === 'genome') return false;
  return LEVELS.get(from) > LEVELS.get(to);
}

function isTestPath(path) {
  return (
    /(^|\/)tests\//.test(path) ||
    /_tests\//.test(path) ||
    /tests?\.rs$/.test(path) ||
    /(^|\/)test_support\.rs$/.test(path) ||
    path.startsWith('bin/')
  );
}

function rustFiles(directory, prefix = '') {
  if (!existsSync(directory)) return [];
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = prefix ? `${prefix}/${entry.name}` : entry.name;
    if (entry.isDirectory()) files.push(...rustFiles(join(directory, entry.name), path));
    else if (entry.name.endsWith('.rs') && !isTestPath(path)) files.push(path);
  }
  return files.sort();
}

// Production lines with comments removed: an item under `#[cfg(test)]` is
// skipped (a `mod x;` line, or a braced block to its closing brace), and the
// names of test-gated `mod x;` declarations are returned so their files are
// skipped too.
function productionLines(text) {
  const lines = text.split(/\r?\n/);
  const kept = [];
  const testModules = [];
  let skipping = false;
  let depth = 0;
  let pendingTest = false;
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index].replace(/\/\/.*$/, '');
    const trimmed = line.trim();
    if (skipping) {
      depth += (line.match(/\{/g) ?? []).length - (line.match(/\}/g) ?? []).length;
      if (depth <= 0) skipping = false;
      continue;
    }
    if (/^#\[cfg\((all\()?test\b/.test(trimmed)) {
      pendingTest = true;
      continue;
    }
    if (pendingTest) {
      if (trimmed.startsWith('#[') || trimmed === '') continue;
      pendingTest = false;
      const declared = trimmed.match(/^(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_]\w*)\s*;/);
      if (declared) testModules.push(declared[1]);
      depth = (line.match(/\{/g) ?? []).length - (line.match(/\}/g) ?? []).length;
      if (depth > 0) skipping = true;
      continue;
    }
    kept.push({ number: index + 1, line });
  }
  return { kept, testModules };
}

function importedPaths(line) {
  const paths = [];
  for (const match of line.matchAll(/\bcrate::([A-Za-z_]\w*(?:::[A-Za-z_]\w*)*)/g)) {
    paths.push(match[1].split('::'));
  }
  for (const match of line.matchAll(/\bcrate::\{([^}]*)\}/g)) {
    for (const part of match[1].split(',')) {
      const path = part.trim().match(/^[A-Za-z_]\w*(?:::[A-Za-z_]\w*)*/)?.[0];
      if (path) paths.push(path.split('::'));
    }
  }
  return paths;
}

function knownDebts(text) {
  return new Set([...(text ?? '').matchAll(/^\| (G\d+) \|/gm)].map((match) => match[1]));
}

function readDebts(text, errors) {
  const debts = [];
  for (const [index, raw] of (text ?? '').split(/\r?\n/).entries()) {
    const line = raw.trim();
    if (!line || line.startsWith('#')) continue;
    const fields = line.split(/\s+/);
    if (fields.length !== 3 || !/^G\d+$/.test(fields[0])) {
      errors.push(`scripts/architecture-debts.txt:${index + 1}: expected "G<n> <file> <module>"`);
      continue;
    }
    debts.push({ id: fields[0], file: fields[1], module: fields[2], used: false });
  }
  return debts;
}

function validate(root) {
  const errors = [];
  const sourceRoot = join(root, 'crates', 'v3-core', 'src');
  const documented = knownDebts(readFileSync(join(root, 'docs', 'architecture.md'), 'utf8'));
  const debtsFile = join(root, 'scripts', 'architecture-debts.txt');
  const debts = readDebts(existsSync(debtsFile) ? readFileSync(debtsFile, 'utf8') : '', errors);

  for (const debt of debts) {
    if (!documented.has(debt.id)) {
      errors.push(`scripts/architecture-debts.txt: ${debt.id} is not a known debt in docs/architecture.md`);
    }
  }

  const testDirs = [];
  const parsed = new Map();
  for (const path of rustFiles(sourceRoot)) {
    const result = productionLines(readFileSync(join(sourceRoot, path), 'utf8'));
    parsed.set(path, result.kept);
    const directory = path.replace(/(^|\/)(mod|lib)\.rs$/, '').replace(/\.rs$/, '');
    for (const name of result.testModules) testDirs.push(directory ? `${directory}/${name}` : name);
  }
  const testGated = (path) => {
    const module = path.replace(/(\/mod)?\.rs$/, '');
    return testDirs.some((dir) => module === dir || module.startsWith(`${dir}/`));
  };

  for (const path of rustFiles(sourceRoot)) {
    if (path === 'lib.rs' || testGated(path)) continue;
    const modulePath = path.replace(/(\/mod)?\.rs$/, '').split('/');
    const from = domainOf(modulePath);
    if (!from) {
      errors.push(`crates/v3-core/src/${path}: module ${modulePath[0]} has no domain in docs/architecture.md section 3`);
      continue;
    }
    for (const { number, line } of parsed.get(path)) {
      for (const segments of importedPaths(line)) {
        const to = domainOf(segments);
        if (!to || allowed(from, to)) continue;
        const module = segments.slice(0, 2).join('::');
        const debt = debts.find((entry) => entry.file === path && entry.module === module);
        if (debt) {
          debt.used = true;
          continue;
        }
        errors.push(`crates/v3-core/src/${path}:${number}: ${from} imports ${to} (${module})`);
      }
    }
  }

  for (const debt of debts) {
    if (!debt.used) {
      errors.push(
        `scripts/architecture-debts.txt: stale debt ${debt.id} ${debt.file} ${debt.module}: ` +
          'the code no longer needs it; delete the entry (and the debt in docs/architecture.md once paid)',
      );
    }
  }
  return errors;
}

let root = process.cwd();
const args = process.argv.slice(2);
if (args.length) {
  if (args.length !== 2 || args[0] !== '--root') usage('invalid arguments');
  else root = resolve(args[1]);
}

if (process.exitCode !== 2) {
  const errors = validate(root);
  if (errors.length) {
    for (const error of errors) console.error(`architecture-check: ${error}`);
    console.error(
      `architecture-check: ${errors.length} problem(s). Section 3 of docs/architecture.md ` +
        `defines the order; ${relative(root, join(root, 'scripts', 'architecture-debts.txt'))} lists owed imports.`,
    );
    process.exitCode = 1;
  } else {
    console.log('architecture-check: imports follow docs/architecture.md');
  }
}
