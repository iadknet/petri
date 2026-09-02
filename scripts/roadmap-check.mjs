#!/usr/bin/env node

import { execFileSync } from 'node:child_process';
import {
  existsSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  readdirSync,
  statSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join, relative, resolve } from 'node:path';

const TRACK_PATH = /^t(\d{2})-([a-z0-9]+(?:-[a-z0-9]+)*)\.md$/;
const SPEC_PATH = /^t(\d{2})-f(\d{2})-([a-z0-9]+(?:-[a-z0-9]+)*)\.md$/;
const ID = /^T(\d{2})$/;
const FEATURE_ID = /^T(\d{2})\.F(\d{2})$/;
const DATE = /^\d{4}-\d{2}-\d{2}$/;

function usage(message) {
  if (message) console.error(`roadmap-check: ${message}`);
  console.error('Usage: scripts/roadmap-check.mjs [--root <repository> | --staged]');
  process.exitCode = 2;
}

function sectionMap(text) {
  const lines = text.split(/\r?\n/);
  const sections = new Map();
  let current = null;
  for (const line of lines) {
    const heading = line.match(/^## (.+)$/);
    if (heading) {
      current = { heading: heading[1], lines: [] };
      sections.set(heading[1], current);
    } else if (current) {
      current.lines.push(line);
    }
  }
  return sections;
}

function metadataEntries(text, name) {
  const expression = new RegExp(`^\\*\\*${name}\\*\\*: ([^\\n\\r]+)$`, 'gm');
  return [...text.matchAll(expression)].map((match) => match[1]);
}

function metadata(text, name) {
  return metadataEntries(text, name)[0];
}

function checkMetadata(text, file, errors, name, allowed) {
  const values = metadataEntries(text, name);
  if (!values.length) {
    errors.push(`${file}: missing **${name}** metadata`);
    return undefined;
  }
  if (values.length > 1) errors.push(`${file}: duplicate **${name}** metadata (${values.length} entries)`);
  const value = values[0];
  if (allowed && !allowed.includes(value)) {
    errors.push(`${file}: invalid ${name} '${value}'`);
  }
  return value;
}

function checkDate(text, file, errors) {
  const value = checkMetadata(text, file, errors, 'Last updated');
  if (!value) return;
  if (!DATE.test(value)) errors.push(`${file}: invalid Last updated '${value}'`);
}

function checkHeadings(text, file, errors, required) {
  const sections = sectionMap(text);
  for (const heading of required) {
    if (!sections.has(heading)) errors.push(`${file}: missing heading '## ${heading}'`);
    else if (text.split(/\r?\n/).filter((line) => line === `## ${heading}`).length > 1) {
      errors.push(`${file}: duplicate heading '## ${heading}'`);
    }
  }
  return sections;
}

function checkCheckboxSection(section, file, errors, label) {
  if (!section) return;
  if (!section.lines.some((line) => /^\s*[-*+] \[[ xX]\]/.test(line))) {
    errors.push(`${file}: ${label} section must contain at least one checkbox`);
  }
}

function unchecked(section) {
  return section?.lines.some((line) => /^\s*[-*+] \[ \]/.test(line)) ?? false;
}

function checked(line) {
  return /^- \[[xX]\]/.test(line);
}

function dependencyList(value, kind, file, errors) {
  if (value === 'None') return [];
  const expression = kind === 'track' ? ID : FEATURE_ID;
  const values = value.split(',').map((item) => item.trim());
  if (values.some((item) => !expression.test(item))) {
    errors.push(`${file}: invalid ${kind} dependency list '${value}'`);
    return [];
  }
  if (new Set(values).size !== values.length) {
    errors.push(`${file}: duplicate ${kind} dependency`);
  }
  if (values.join(', ') !== value) {
    errors.push(`${file}: ${kind} dependencies must use ', ' separators`);
  }
  return values;
}

function findRows(section, file, kind, errors) {
  const rows = [];
  if (!section) return rows;
  for (const line of section.lines) {
    if (!line.startsWith('- [')) continue;
    const expression = kind === 'track'
      ? /^- \[([ xX])\] \*\*(T\d{2}) — (.+?)\*\* — \[Roadmap\]\((roadmaps\/[^)]+)\) — Depends on: (.+)$/
      : /^- \[([ xX])\] \*\*(T\d{2}\.F\d{2}) — (.+?)\*\* — Depends on: (.+)$/;
    const match = line.match(expression);
    if (!match) {
      errors.push(`${file}: malformed ${kind} row '${line}'`);
      continue;
    }
    const isChecked = match[1].toLowerCase() === 'x';
    const id = match[2];
    const title = match[3];
    const link = kind === 'track' ? match[4] : undefined;
    const depText = kind === 'track' ? match[5] : match[4];
    const deps = dependencyList(depText, kind, file, errors);
    rows.push({ id, title, link, deps, checked: isChecked, line });
  }
  return rows;
}

function cycleErrors(graph, file, kind, errors) {
  const visiting = new Set();
  const visited = new Set();
  const stack = [];
  function visit(id) {
    if (visiting.has(id)) {
      const start = stack.indexOf(id);
      errors.push(`${file}: ${kind} dependency cycle: ${stack.slice(start).concat(id).join(' -> ')}`);
      return;
    }
    if (visited.has(id)) return;
    visiting.add(id);
    stack.push(id);
    for (const dependency of graph.get(id) ?? []) visit(dependency);
    stack.pop();
    visiting.delete(id);
    visited.add(id);
  }
  for (const id of graph.keys()) visit(id);
}

function canonicalTrackLink(link) {
  return link?.replace(/^roadmaps\//, '') ?? '';
}

function loadMarkdown(file) {
  try {
    return readFileSync(file, 'utf8');
  } catch {
    return undefined;
  }
}

function markdownFiles(directory) {
  if (!existsSync(directory) || !statSync(directory).isDirectory()) return [];
  const files = [];
  function visit(current, prefix = '') {
    for (const entry of readdirSync(current, { withFileTypes: true }).sort((left, right) => left.name.localeCompare(right.name))) {
      const relativePath = prefix ? join(prefix, entry.name) : entry.name;
      const fullPath = join(current, entry.name);
      if (entry.isDirectory()) {
        visit(fullPath, relativePath);
      } else if ((entry.isFile() || entry.isSymbolicLink()) && entry.name.endsWith('.md') && entry.name !== 'README.md' && !entry.name.startsWith('_')) {
        files.push(relativePath);
      }
    }
  }
  visit(directory);
  return files;
}

function validateRoot(root) {
  const errors = [];
  const masterPath = join(root, 'docs', 'roadmap.md');
  const tracksDir = join(root, 'docs', 'roadmaps');
  const specsDir = join(root, 'docs', 'specs', 'roadmap');
  const masterText = loadMarkdown(masterPath);
  const trackFiles = markdownFiles(tracksDir);
  const specFiles = markdownFiles(specsDir);

  if (masterText === undefined) {
    if (trackFiles.length || specFiles.length) {
      errors.push('docs/roadmap.md: live track or feature-spec files require a live master');
    }
    return errors;
  }

  const master = 'docs/roadmap.md';
  checkMetadata(masterText, master, errors, 'Status', ['Planning', 'Active', 'Complete']);
  checkDate(masterText, master, errors);
  const masterSections = checkHeadings(masterText, master, errors, [
    'Success Definition', 'Track Roadmaps', 'Final Success Criteria', 'Notes for AI Agents',
  ]);
  const masterRows = findRows(masterSections.get('Track Roadmaps'), master, 'track', errors);
  const trackById = new Map();
  const trackRowsById = new Map(masterRows.map((row) => [row.id, row]));
  const trackGraph = new Map();
  for (const row of masterRows) {
    if (trackById.has(row.id)) errors.push(`${master}: duplicate track ID ${row.id}`);
    trackById.set(row.id, row);
    trackGraph.set(row.id, row.deps);
    if (row.link && !TRACK_PATH.test(canonicalTrackLink(row.link))) {
      errors.push(`${master}: non-canonical track path '${row.link}'`);
    }
    for (const dependency of row.deps) {
      if (!ID.test(dependency)) errors.push(`${master}: invalid track dependency '${dependency}'`);
    }
  }
  for (const dependencyListValue of trackGraph.values()) {
    for (const dependency of dependencyListValue) {
      if (!trackById.has(dependency)) errors.push(`${master}: unknown track dependency ${dependency}`);
    }
  }
  cycleErrors(trackGraph, master, 'track', errors);

  const tracks = new Map();
  for (const path of trackFiles) {
    const file = `docs/roadmaps/${path}`;
    const pathMatch = path.match(TRACK_PATH);
    const text = loadMarkdown(join(tracksDir, path));
    if (!pathMatch) {
      errors.push(`${file}: non-canonical track path`);
      continue;
    }
    const id = `T${pathMatch[1]}`;
    const row = trackRowsById.get(id);
    if (!row) errors.push(`${file}: orphan track ${id} is not linked exactly once by the master`);
    else if (canonicalTrackLink(row.link) !== path) errors.push(`${file}: canonical path does not match master link`);
    if (tracks.has(id)) errors.push(`${file}: duplicate track ID ${id}`);
    tracks.set(id, { id, name: path, file, text, row, features: new Map(), featureRows: [], status: undefined });
  }
  for (const row of masterRows) {
    const linkedName = canonicalTrackLink(row.link);
    const matches = masterRows.filter((candidate) => canonicalTrackLink(candidate.link) === linkedName);
    if (matches.length !== 1) errors.push(`${master}: track link '${row.link}' must occur exactly once`);
    if (!tracks.has(row.id)) errors.push(`${master}: linked track ${row.id} is missing`);
  }

  for (const track of tracks.values()) {
    const { text, file, id, row } = track;
    const status = checkMetadata(text, file, errors, 'Status', ['Planned', 'In Progress', 'Complete']);
    track.status = status;
    checkDate(text, file, errors);
    const masterLink = checkMetadata(text, file, errors, 'Master');
    if (masterLink !== '[Program Roadmap](../roadmap.md)') {
      errors.push(`${file}: expected **Master**: [Program Roadmap](../roadmap.md)`);
    }
    const sections = checkHeadings(text, file, errors, [
      'Goal', 'Track Success Criteria', 'Executable Features', 'Notes for AI Agents',
    ]);
    const featureRows = findRows(sections.get('Executable Features'), file, 'feature', errors);
    track.featureRows = featureRows;
    const graph = new Map();
    for (const feature of featureRows) {
      if (!feature.id.startsWith(`${id}.`)) errors.push(`${file}: feature ${feature.id} does not belong to ${id}`);
      if (track.features.has(feature.id)) errors.push(`${file}: duplicate feature ID ${feature.id}`);
      track.features.set(feature.id, feature);
      graph.set(feature.id, feature.deps);
      for (const dependency of feature.deps) {
        if (!FEATURE_ID.test(dependency)) errors.push(`${file}: invalid feature dependency '${dependency}'`);
      }
    }
    // Build the complete feature graph after all tracks have been read below.
    track.featureGraph = graph;
    checkCheckboxSection(sections.get('Track Success Criteria'), file, errors, 'Track Success Criteria');
    const allFeaturesChecked = featureRows.length > 0 && featureRows.every((feature) => feature.checked);
    const trackSuccessChecked = !unchecked(sections.get('Track Success Criteria'));
    if (status === 'Planned' && (featureRows.some((feature) => feature.checked))) {
      errors.push(`${file}: Planned track cannot contain a checked feature`);
    }
    if (status === 'Complete') {
      if (!featureRows.length) errors.push(`${file}: Complete track must contain at least one feature`);
      if (!allFeaturesChecked || !trackSuccessChecked) errors.push(`${file}: Complete track requires all features and success criteria checked`);
      if (!row?.checked) errors.push(`${file}: Complete track requires its master rollup checked`);
    }
  }

  // Feature dependencies are global, while the feature rows remain owned by tracks.
  const allFeatures = new Map();
  for (const track of tracks.values()) {
    for (const feature of track.featureRows) {
      if (allFeatures.has(feature.id)) errors.push(`${track.file}: duplicate feature ownership ${feature.id}`);
      allFeatures.set(feature.id, { ...feature, track });
    }
  }
  const featureGraph = new Map([...allFeatures].map(([id, feature]) => [id, feature.deps]));
  for (const [id, deps] of featureGraph) {
    for (const dependency of deps) {
      if (!allFeatures.has(dependency)) errors.push(`feature ${id}: unknown dependency ${dependency}`);
    }
  }
  cycleErrors(featureGraph, 'roadmap feature graph', 'feature', errors);

  const specs = new Map();
  for (const path of specFiles) {
    const file = `docs/specs/roadmap/${path}`;
    const pathMatch = path.match(SPEC_PATH);
    const text = loadMarkdown(join(specsDir, path));
    if (!pathMatch) {
      errors.push(`${file}: non-canonical feature-spec path`);
      continue;
    }
    const id = `T${pathMatch[1]}.F${pathMatch[2]}`;
    const feature = allFeatures.get(id);
    if (!feature) errors.push(`${file}: orphan feature spec ${id}`);
    if (specs.has(id)) errors.push(`${file}: multiple specs for feature ${id}`);
    const status = checkMetadata(text, file, errors, 'Status', ['Planned', 'In Progress', 'Blocked', 'Complete']);
    checkDate(text, file, errors);
    const featureMetadata = checkMetadata(text, file, errors, 'Feature');
    if (featureMetadata !== id) errors.push(`${file}: **Feature** must be ${id}`);
    const trackMetadata = checkMetadata(text, file, errors, 'Track');
    if (feature?.track.row) {
      const expected = `[${feature.track.id} — ${feature.track.row.title}](../../roadmaps/${feature.track.name})`;
      if (trackMetadata !== expected) errors.push(`${file}: owning-track link must be exactly ${expected}`);
    } else if (feature) {
      errors.push(`${file}: owning track ${feature.track.id} is missing its master track row`);
    } else if (!trackMetadata) {
      errors.push(`${file}: missing **Track** metadata`);
    }
    const sections = checkHeadings(text, file, errors, [
      'Overview', 'Goal', 'Non-Goals', 'Inputs and Invariants', 'Implementation Tasks',
      'Verification', 'Success Criteria', 'Blocker', 'Deferred Review Findings', 'Notes for AI Agents',
    ]);
    checkCheckboxSection(sections.get('Implementation Tasks'), file, errors, 'Implementation Tasks');
    checkCheckboxSection(sections.get('Verification'), file, errors, 'Verification');
    checkCheckboxSection(sections.get('Success Criteria'), file, errors, 'Success Criteria');
    const blocker = sections.get('Blocker')?.lines.join('\n').trim() ?? '';
    if (status === 'Blocked') {
      if (feature?.checked) errors.push(`${file}: Blocked feature cannot be checked`);
      if (!blocker || /^None\.?$/i.test(blocker)) errors.push(`${file}: Blocked spec requires a concrete Blocker`);
      if (feature && feature.deps.some((dep) => !allFeatures.get(dep)?.checked)) errors.push(`${file}: Blocked spec dependencies must be checked`);
    }
    if (feature && ['In Progress', 'Blocked', 'Complete'].includes(status)) {
      for (const dependency of feature.deps) {
        if (!allFeatures.get(dependency)?.checked) errors.push(`${file}: ${status} spec requires checked dependency ${dependency}`);
      }
    }
    if (status === 'Complete') {
      if (!feature?.checked) errors.push(`${file}: Complete spec requires its feature checked`);
      if (unchecked(sections.get('Implementation Tasks')) || unchecked(sections.get('Verification')) || unchecked(sections.get('Success Criteria'))) {
        errors.push(`${file}: Complete spec cannot contain unchecked implementation, verification, or success items`);
      }
    }
    specs.set(id, { id, name: path, file, text, status, sections, feature });
  }

  for (const feature of allFeatures.values()) {
    const spec = specs.get(feature.id);
    if (feature.checked) {
      if (!spec || spec.status !== 'Complete') errors.push(`${feature.track.file}: checked feature ${feature.id} requires exactly one Complete spec`);
      for (const dependency of feature.deps) {
        if (!allFeatures.get(dependency)?.checked) errors.push(`${feature.track.file}: checked feature ${feature.id} has unchecked dependency ${dependency}`);
      }
    }
    if (spec && spec.feature !== feature) errors.push(`${spec.file}: feature/spec ownership mismatch for ${feature.id}`);
  }

  const masterStatus = metadata(masterText, 'Status');
  const masterRollupsChecked = masterRows.every((row) => row.checked);
  const masterSuccess = masterSections.get('Final Success Criteria');
  checkCheckboxSection(masterSuccess, master, errors, 'Final Success Criteria');
  const anyTrack = masterRows.length > 0;
  if (masterStatus === 'Planning' && masterRows.some((row) => row.checked)) errors.push(`${master}: Planning master cannot contain a checked rollup`);
  if (masterStatus === 'Active' && (!anyTrack || (masterRollupsChecked && !unchecked(masterSuccess)))) errors.push(`${master}: Active master must have a non-complete roadmap`);
  if (masterStatus === 'Complete') {
    if (!anyTrack) errors.push(`${master}: Complete master must contain at least one track`);
    if (!masterRollupsChecked || unchecked(masterSuccess)) errors.push(`${master}: Complete master requires all rollups and final success criteria checked`);
  }
  for (const row of masterRows) {
    const track = tracks.get(row.id);
    if (track && row.checked !== (track.status === 'Complete')) {
      errors.push(`${master}: rollup ${row.id} must be checked if and only if its track is Complete`);
    }
    for (const dependency of row.deps) {
      if (row.checked && !trackRowsById.get(dependency)?.checked) errors.push(`${master}: checked track ${row.id} has unchecked dependency ${dependency}`);
    }
  }

  return [...new Set(errors)].sort();
}

function stagedRoot() {
  try {
    execFileSync('git', ['rev-parse', '--git-dir'], { stdio: 'ignore' });
  } catch {
    return { root: null, noop: true };
  }
  const names = execFileSync('git', ['diff', '--cached', '--name-only', '--diff-filter=ACMRD'], { encoding: 'utf8' })
    .split(/\r?\n/).filter(Boolean);
  const relevant = names.some((name) => name === 'docs/roadmap.md' || name.startsWith('docs/roadmaps/') || name.startsWith('docs/specs/roadmap/'));
  if (!relevant) return { root: null, noop: true };
  const root = mkdtempSync(join(tmpdir(), 'petri-roadmap-staged.'));
  execFileSync('git', ['checkout-index', '--all', `--prefix=${root}/`], { stdio: 'ignore' });
  return { root, noop: false };
}

function main() {
  let root = resolve(process.cwd());
  let staged = false;
  for (let index = 2; index < process.argv.length; index += 1) {
    const argument = process.argv[index];
    if (argument === '--staged') {
      if (staged || root !== resolve(process.cwd())) return usage('options are mutually exclusive');
      staged = true;
    } else if (argument === '--root') {
      if (staged) return usage('--root cannot be combined with --staged; options are mutually exclusive');
      if (index + 1 >= process.argv.length) return usage('--root requires a path');
      root = resolve(process.argv[++index]);
    } else {
      return usage(`unknown option '${argument}'`);
    }
  }
  if (staged) {
    const materialized = stagedRoot();
    if (materialized.noop) {
      console.log('roadmap-check: no staged roadmap changes to validate');
      return;
    }
    root = materialized.root;
    try {
      const errors = validateRoot(root);
      if (errors.length) {
        errors.forEach((error) => console.error(`roadmap-check: ${error}`));
        process.exitCode = 1;
      } else console.log('roadmap-check: staged roadmap validation passed');
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
    return;
  }
  const errors = validateRoot(root);
  if (errors.length) {
    errors.forEach((error) => console.error(`roadmap-check: ${error}`));
    process.exitCode = 1;
  } else {
    const hasMaster = existsSync(join(root, 'docs', 'roadmap.md'));
    console.log(`roadmap-check: ${hasMaster ? 'validation passed' : 'no live roadmap files to validate'}`);
  }
}

main();
