#!/usr/bin/env node

// Enforces docs/architecture.md on crates/v3-core: every `crate::`, `self::`
// and `super::` path points down section 3's dependency order into a domain it
// names and breaks neither named prohibition; lib.rs only declares modules;
// and, from section 2, no crate depends on v3-lab. Imports the code still owes are listed in
// scripts/architecture-debts.txt under a debt ID from section 5; an entry the
// code no longer needs, or whose ID the document no longer lists, fails, so
// paying a debt deletes it from both places.

import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { join, relative, resolve } from 'node:path';

// Section 3's order, bottom up; a domain imports only lower levels. The
// document's own list must match it (sectionOrderErrors).
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
// WORKAROUND(G26): four readers still live under `creature/`; T30.F07 moves
// them into `neighborhood/` and deletes this list.
const CREATURE_READERS = new Set(['plasticity_census', 'sensor_census', 'drives', 'ring_rules']);

function usage(message) {
  console.error(`architecture-check: ${message}`);
  console.error('Usage: scripts/architecture-check.mjs [--root DIR]');
  process.exitCode = 2;
}

// Domain of a module path given as segments under `crate::`.
function domainOf(segments) {
  const [first, second] = segments;
  // WORKAROUND(G24): the genome still lives under `creature/`; T30.F07 moves
  // it to `genome/` and deletes this case.
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
    // WORKAROUND(G25): the profiling binary lives in v3-core until T30.F07
    // moves it to v3-cli and deletes this skip.
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

// The use tree that starts at `start`: a path, then any balanced brace group.
function treeAt(text, start) {
  let end = start;
  let depth = 0;
  while (end < text.length) {
    const char = text[end];
    if (char === '{') depth += 1;
    else if (char === '}') {
      if (depth === 0) break;
      depth -= 1;
    } else if (depth === 0 && !/[\w:*]/.test(char)) break;
    end += 1;
  }
  return text.slice(start, end);
}

function splitTopLevel(text) {
  const parts = [];
  let depth = 0;
  let start = 0;
  for (let index = 0; index < text.length; index += 1) {
    if (text[index] === '{') depth += 1;
    else if (text[index] === '}') depth -= 1;
    else if (text[index] === ',' && depth === 0) {
      parts.push(text.slice(start, index));
      start = index + 1;
    }
  }
  parts.push(text.slice(start));
  return parts;
}

// Every path a use tree names, nested groups expanded.
function expandTree(tree) {
  const text = tree.trim();
  if (!text) return [];
  if (text.startsWith('{')) return splitTopLevel(text.slice(1, -1)).flatMap(expandTree);
  const match = text.match(/^([A-Za-z_]\w*(?:::[A-Za-z_]\w*)*)(?:::(\{[\s\S]*))?/);
  if (!match) return [];
  const head = match[1].split('::');
  const groups = match[2] ? expandTree(match[2]) : [];
  return groups.length ? groups.map((tail) => head.concat(tail)) : [head];
}

// Module paths a statement names, resolved to segments under `crate::`:
// `crate::` paths, and `self::` and `super::` paths relative to `module`.
function importedPaths(text, module) {
  const paths = [];
  for (const match of text.matchAll(/(?<![\w:])(crate|self|super(?:::super)*)::/g)) {
    const root = match[1];
    let base;
    if (root === 'crate') base = [];
    else if (root === 'self') base = module;
    else base = module.slice(0, module.length - root.split('::').length);
    for (const path of expandTree(treeAt(text, match.index + match[0].length))) {
      const segments = base.concat(path).filter((segment) => segment !== 'self');
      if (segments.length) paths.push(segments);
    }
  }
  return paths;
}

// Section 3's numbered list against LEVELS, so the document and the checker
// cannot drift apart. Each item names its domains by module path.
function sectionOrderErrors(text) {
  const section = text.split(/^## 3 Core domains$/m)[1]?.split(/^## /m)[0];
  if (!section) return [];
  const errors = [];
  for (const match of section.matchAll(/^(\d+)\. (.+)$/gm)) {
    const level = Number(match[1]);
    for (const part of match[2].split(', ')) {
      const name = part.match(/`([A-Za-z_]\w*)\/?`/)?.[1];
      const domain = name && domainOf([name]);
      if (!domain) {
        errors.push(`docs/architecture.md: section 3 level ${level} names no domain the checker knows ("${part}")`);
      } else if (LEVELS.get(domain) !== level) {
        errors.push(
          `docs/architecture.md: section 3 places ${domain} at level ${level}; the checker has level ${LEVELS.get(domain)}`,
        );
      }
    }
  }
  return errors;
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

// lib.rs only declares modules: an item or re-export there would sit outside
// every domain, where no import rule reaches it.
function libLines(sourceRoot) {
  const file = join(sourceRoot, 'lib.rs');
  if (!existsSync(file)) return [];
  const errors = [];
  for (const [index, raw] of readFileSync(file, 'utf8').split(/\r?\n/).entries()) {
    const line = raw.trim();
    if (
      line === '' ||
      line.startsWith('//') ||
      /^#!?\[.*\]$/.test(line) ||
      /^(pub(\([^)]*\))?\s+)?mod\s+[A-Za-z_]\w*\s*;$/.test(line)
    ) {
      continue;
    }
    errors.push(
      `crates/v3-core/src/lib.rs:${index + 1}: lib.rs holds only module declarations, attributes and comments`,
    );
  }
  return errors;
}

// Section 2: v3-lab is never depended on.
function labDependents(root) {
  const manifests = [join(root, 'Cargo.toml')];
  const crates = join(root, 'crates');
  if (existsSync(crates)) {
    for (const entry of readdirSync(crates, { withFileTypes: true })) {
      if (entry.isDirectory() && entry.name !== 'v3-lab') manifests.push(join(crates, entry.name, 'Cargo.toml'));
    }
  }
  const errors = [];
  for (const manifest of manifests) {
    if (!existsSync(manifest)) continue;
    for (const [index, line] of readFileSync(manifest, 'utf8').split(/\r?\n/).entries()) {
      if (/^\s*v3[-_]lab\s*=/.test(line) || /^\s*\[[^\]]*dependencies\.v3[-_]lab\]/.test(line)) {
        errors.push(`${relative(root, manifest)}:${index + 1}: depends on v3-lab, which nothing may depend on`);
      }
    }
  }
  return errors;
}

function validate(root) {
  const errors = [];
  const sourceRoot = join(root, 'crates', 'v3-core', 'src');
  const architecture = readFileSync(join(root, 'docs', 'architecture.md'), 'utf8');
  const documented = knownDebts(architecture);
  errors.push(...sectionOrderErrors(architecture));
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
    const lines = parsed.get(path);
    for (let index = 0; index < lines.length; index += 1) {
      const { number } = lines[index];
      // A use statement can span lines; join it up to its semicolon.
      let text = lines[index].line;
      if (/^\s*(pub(\([^)]*\))?\s+)?use\b/.test(text)) {
        while (!text.includes(';') && index + 1 < lines.length) {
          index += 1;
          text += ` ${lines[index].line}`;
        }
      }
      for (const segments of importedPaths(text, modulePath)) {
        const to = domainOf(segments);
        if (!to) {
          errors.push(
            `crates/v3-core/src/${path}:${number}: ${from} imports crate::${segments[0]}, ` +
              'which no domain in docs/architecture.md section 3 owns',
          );
          continue;
        }
        if (allowed(from, to)) continue;
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

  errors.push(...libLines(sourceRoot), ...labDependents(root));

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
