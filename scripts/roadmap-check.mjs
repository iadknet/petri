#!/usr/bin/env node

import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { join, resolve } from 'node:path';

const TRACK_ID = /^T\d{2}$/;
const FEATURE_ID = /^T\d{2}\.F\d{2}$/;
const TRACK_FILE = /^t(\d{2})-[a-z0-9]+(?:-[a-z0-9]+)*\.md$/;
const SPEC_FILE = /^t(\d{2})-f(\d{2})-[a-z0-9]+(?:-[a-z0-9]+)*\.md$/;

function usage(message) {
  if (message) console.error(`roadmap-check: ${message}`);
  console.error('Usage: scripts/roadmap-check.mjs [--root <repository>]');
  process.exitCode = 2;
}

function read(file) {
  try {
    return readFileSync(file, 'utf8');
  } catch {
    return undefined;
  }
}

function metadata(text, name) {
  return text.match(new RegExp(`^\\*\\*${name}\\*\\*: ([^\\r\\n]+)$`, 'm'))?.[1];
}

function section(text, heading) {
  const escaped = heading.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  return text.match(new RegExp(`^## ${escaped}\\s*$([\\s\\S]*?)(?=^## |$(?![\\s\\S]))`, 'm'))?.[1];
}

function markdownFiles(directory, prefix = '') {
  if (!existsSync(directory)) return [];
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const relative = prefix ? `${prefix}/${entry.name}` : entry.name;
    if (entry.isDirectory()) files.push(...markdownFiles(join(directory, entry.name), relative));
    else if (entry.name.endsWith('.md') && entry.name !== 'README.md' && !entry.name.startsWith('_')) files.push(relative);
  }
  return files.sort();
}

function status(text, file, allowed, errors) {
  const value = metadata(text, 'Status');
  if (!allowed.includes(value)) errors.push(`${file}: missing or invalid Status`);
  return value;
}

function checked(mark) {
  return mark.toLowerCase() === 'x';
}

function trackRows(text, file, errors) {
  const body = section(text, 'Track Roadmaps');
  if (body === undefined) {
    errors.push(`${file}: missing Track Roadmaps section`);
    return [];
  }
  const rows = [];
  for (const line of body.split(/\r?\n/).filter((item) => item.startsWith('- ['))) {
    const match = line.match(/^- \[([ xX])\] \*\*(T\d{2}) — (.+?)\*\* — \[Roadmap\]\((roadmaps\/([^)]+))\) — Depends on: .+$/);
    if (!match) errors.push(`${file}: malformed track row '${line}'`);
    else rows.push({ checked: checked(match[1]), id: match[2], link: match[4], path: match[5] });
  }
  return rows;
}

function dependencies(value, file, errors) {
  if (value === 'None') return [];
  const values = value.split(',').map((item) => item.trim());
  if (!values.length || values.some((item) => !FEATURE_ID.test(item))) {
    errors.push(`${file}: invalid feature dependency list '${value}'`);
    return [];
  }
  return values;
}

function featureRows(text, file, trackId, errors) {
  const body = section(text, 'Executable Features');
  if (body === undefined) {
    errors.push(`${file}: missing Executable Features section`);
    return [];
  }
  const rows = [];
  for (const line of body.split(/\r?\n/).filter((item) => item.startsWith('- ['))) {
    const match = line.match(/^- \[([ xX])\] \*\*(T\d{2}\.F\d{2}) — (.+?)\*\* — Depends on: (.+)$/);
    if (!match) {
      errors.push(`${file}: malformed feature row '${line}'`);
      continue;
    }
    if (!match[2].startsWith(`${trackId}.`)) errors.push(`${file}: ${match[2]} does not belong to ${trackId}`);
    rows.push({ checked: checked(match[1]), id: match[2], deps: dependencies(match[4], file, errors) });
  }
  return rows;
}

function checklistComplete(text, heading, file, errors) {
  const body = section(text, heading);
  const boxes = body?.split(/\r?\n/).map((line) => line.match(/^\s*[-*+] \[([ xX])\]/)).filter(Boolean) ?? [];
  if (!boxes.length) errors.push(`${file}: Complete spec requires a ${heading} checklist`);
  else if (boxes.some((box) => !checked(box[1]))) errors.push(`${file}: Complete spec has unchecked ${heading} items`);
}

function criteriaComplete(text, heading, file, errors) {
  const body = section(text, heading);
  const boxes = body?.split(/\r?\n/).map((line) => line.match(/^\s*[-*+] \[([ xX])\]/)).filter(Boolean) ?? [];
  if (!boxes.length || boxes.some((box) => !checked(box[1]))) {
    errors.push(`${file}: Complete status requires all ${heading} items checked`);
  }
}

function findCycles(features, errors) {
  const visiting = new Set();
  const visited = new Set();
  const stack = [];
  function visit(id) {
    if (visiting.has(id)) {
      const start = stack.indexOf(id);
      errors.push(`docs/roadmap.md: feature dependency cycle: ${stack.slice(start).concat(id).join(' -> ')}`);
      return;
    }
    if (visited.has(id)) return;
    visiting.add(id);
    stack.push(id);
    for (const dependency of features.get(id)?.deps ?? []) visit(dependency);
    stack.pop();
    visiting.delete(id);
    visited.add(id);
  }
  for (const id of features.keys()) visit(id);
}

function validate(root) {
  const errors = [];
  const masterFile = join(root, 'docs', 'roadmap.md');
  const tracksDir = join(root, 'docs', 'roadmaps');
  const specsDir = join(root, 'docs', 'specs', 'roadmap');
  const masterText = read(masterFile);
  const trackFiles = markdownFiles(tracksDir);
  const specFiles = markdownFiles(specsDir);

  if (masterText === undefined) {
    if (trackFiles.length || specFiles.length) errors.push('docs/roadmap.md: live tracks or specs require a master roadmap');
    return errors;
  }

  const masterStatus = status(masterText, 'docs/roadmap.md', ['Planning', 'Active', 'Complete'], errors);
  const rows = trackRows(masterText, 'docs/roadmap.md', errors);
  const rowsById = new Map();
  const paths = new Set();
  for (const row of rows) {
    if (!TRACK_ID.test(row.id) || rowsById.has(row.id)) errors.push(`docs/roadmap.md: duplicate or invalid track ID ${row.id}`);
    if (paths.has(row.path)) errors.push(`docs/roadmap.md: duplicate track link ${row.link}`);
    rowsById.set(row.id, row);
    paths.add(row.path);
  }

  const tracks = new Map();
  const features = new Map();
  for (const path of trackFiles) {
    const file = `docs/roadmaps/${path}`;
    const match = path.match(TRACK_FILE);
    if (!match) {
      errors.push(`${file}: non-canonical track path`);
      continue;
    }
    const id = `T${match[1]}`;
    const row = rowsById.get(id);
    if (!row || row.path !== path) errors.push(`${file}: not linked exactly once by its ${id} master row`);
    const text = read(join(tracksDir, path));
    const trackStatus = status(text, file, ['Planned', 'In Progress', 'Complete'], errors);
    const owned = featureRows(text, file, id, errors);
    if (tracks.has(id)) errors.push(`${file}: duplicate track ${id}`);
    tracks.set(id, { file, text, status: trackStatus, features: owned });
    for (const feature of owned) {
      if (features.has(feature.id)) errors.push(`${file}: duplicate feature ownership for ${feature.id}`);
      else features.set(feature.id, { ...feature, trackId: id });
    }
  }
  for (const row of rows) {
    const track = tracks.get(row.id);
    if (!track) errors.push(`docs/roadmap.md: linked track ${row.id} is missing`);
    else if (row.checked !== (track.status === 'Complete')) errors.push(`docs/roadmap.md: ${row.id} rollup must match the track's Complete status`);
  }

  for (const feature of features.values()) {
    for (const dependency of feature.deps) {
      if (!features.has(dependency)) errors.push(`${feature.id}: unknown feature dependency ${dependency}`);
    }
  }
  findCycles(features, errors);

  const specs = new Map();
  for (const path of specFiles) {
    const file = `docs/specs/roadmap/${path}`;
    const match = path.match(SPEC_FILE);
    if (!match) {
      errors.push(`${file}: non-canonical or nested feature-spec path`);
      continue;
    }
    const pathId = `T${match[1]}.F${match[2]}`;
    const text = read(join(specsDir, path));
    const featureId = metadata(text, 'Feature');
    const specStatus = status(text, file, ['Planned', 'In Progress', 'Blocked', 'Complete'], errors);
    if (featureId !== pathId) errors.push(`${file}: Feature metadata must match ${pathId}`);
    if (!features.has(pathId)) errors.push(`${file}: orphan feature spec for ${pathId}`);
    if (!specs.has(pathId)) specs.set(pathId, []);
    specs.get(pathId).push({ file, text, status: specStatus });
  }

  for (const feature of features.values()) {
    const ownedSpecs = specs.get(feature.id) ?? [];
    if (ownedSpecs.length > 1) errors.push(`${feature.id}: multiple feature specs`);
    const complete = ownedSpecs.length === 1 && ownedSpecs[0].status === 'Complete';
    if (feature.checked !== complete) errors.push(`${feature.id}: feature checkbox and Complete spec must agree`);
    if (complete) {
      for (const heading of ['Implementation Tasks', 'Verification', 'Success Criteria']) {
        checklistComplete(ownedSpecs[0].text, heading, ownedSpecs[0].file, errors);
      }
    }
  }

  for (const track of tracks.values()) {
    if (track.status !== 'Complete') continue;
    if (!track.features.length || track.features.some((feature) => !feature.checked)) {
      errors.push(`${track.file}: Complete track requires every feature checked`);
    }
    criteriaComplete(track.text, 'Track Success Criteria', track.file, errors);
  }
  if (masterStatus === 'Complete') {
    if (!rows.length || rows.some((row) => !row.checked)) errors.push('docs/roadmap.md: Complete master requires every track checked');
    criteriaComplete(masterText, 'Final Success Criteria', 'docs/roadmap.md', errors);
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
    for (const error of errors) console.error(`roadmap-check: ${error}`);
    process.exitCode = 1;
  } else {
    console.log(`roadmap-check: ${existsSync(join(root, 'docs', 'roadmap.md')) ? 'validation passed' : 'no live roadmap files to validate'}`);
  }
}
