#!/usr/bin/env node
// T15.F02's frozen-history census and local evidence package. Never changes refs.
import { createHash } from 'node:crypto';
import { execFileSync, spawnSync } from 'node:child_process';
import { chmodSync, copyFileSync, existsSync, mkdirSync, readFileSync, statSync, unlinkSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const json = value => Buffer.from(`${JSON.stringify(value, null, 2)}\n`);
const canonical = value => JSON.stringify(value, function (key, item) {
  return item && typeof item === 'object' && !Array.isArray(item)
    ? Object.fromEntries(Object.entries(item).sort(([a], [b]) => a.localeCompare(b))) : item;
});
const git = (root, ...args) => execFileSync('git', args, { cwd: root, maxBuffer: 1024 ** 3 });
const textGit = (root, ...args) => git(root, ...args).toString('utf8').trim();
const key = entry => `${entry.path}\0${entry.oid}`;
// The committed summary version `v3-cli bench-summarize` writes and loads.
const SUMMARY_VERSION = 2;

function write(path, bytes, immutable = false) {
  mkdirSync(dirname(path), { recursive: true });
  if (immutable && existsSync(path)) {
    if (!readFileSync(path).equals(bytes)) throw new Error(`existing export differs: ${path}`);
    return;
  }
  writeFileSync(path, bytes, immutable ? { flag: 'wx' } : undefined);
}

function receipt(path) {
  const bytes = readFileSync(path);
  return { path, sha256: sha256(bytes), bytes: bytes.length };
}

function treeEntries(root, tree) {
  return git(root, 'ls-tree', '-r', '-t', '-z', '--full-tree', tree).toString('utf8').split('\0').filter(Boolean).map(row => {
    const tab = row.indexOf('\t');
    const [mode, type, oid] = row.slice(0, tab).split(' ');
    const path = row.slice(tab + 1);
    if (path.includes('\ufffd')) throw new Error(`non-UTF-8 Git path requires review in tree ${tree}`);
    return { path, mode, type, oid };
  });
}

function classify(bytes, paths) {
  const content = bytes.toString('utf8').trim();
  const reportLocation = paths.some(path => /^docs\/progress\/features\/.*\.json$/.test(path));
  const reportTokens = /"(?:schema_version|deterministic|per_creature_tick)"\s*:/.test(content);
  let value;
  try { value = JSON.parse(content); } catch {
    if (reportLocation || reportTokens && /^\s*\{/.test(content)) {
      return { candidate: true, evidence: 'unparsable content at report location or with report keys', identity: null };
    }
    const lines = content.split('\n').filter(Boolean);
    if (lines.length > 1 && lines.every(line => {
      try { const row = JSON.parse(line); return (typeof row.tick === 'number'
        || row.protocol_version === 'v3alpha1' && typeof row.event_type === 'string') && !row.deterministic; } catch { return false; }
    })) return { reason: 'simulation trace, not a benchmark report', evidence: 'NDJSON tick samples and run protocol events have no benchmark envelope' };
    return { reason: content.includes('\0') ? 'non-report binary' : 'non-report text', evidence: 'no JSON report envelope or report location' };
  }
  const references = [];
  if (Array.isArray(value?.comparison?.references)) value.comparison.references.forEach((reference, i) => {
    if (typeof reference.path === 'string') references.push({ pointer: `/comparison/references/${i}/path`, path: reference.path });
  });
  if (paths.some(path => path.endsWith('/benchmark-series.json'))) {
    for (const [prefix, series] of [['', value], ...Object.entries(value ?? {}).map(([name, series]) => [`/${name}`, series])]) {
      if (!series || Array.isArray(series) || typeof series !== 'object') continue;
      if (typeof series.epoch_baseline === 'string') references.push({ pointer: `${prefix}/epoch_baseline`, path: series.epoch_baseline });
      if (Array.isArray(series.closed)) series.closed.forEach((path, i) => references.push({ pointer: `${prefix}/closed/${i}`, path }));
    }
  }
  if (value?.kind === 'petri-benchmark-summary') return { reason: 'existing benchmark summary', evidence: `summary version ${value.summary_version}`, references };
  const shaped = value && typeof value === 'object' && !Array.isArray(value)
    && (value.deterministic || value.schema_version !== undefined && (value.environment || value.feature));
  if (!shaped) return { reason: 'unrelated JSON', evidence: `top-level keys: ${Object.keys(value ?? {}).slice(0, 20).join(', ')}`, references };
  const identity = typeof value.feature === 'string' && value.feature
    && typeof value.environment?.generated_at === 'string' && value.environment.generated_at
    && typeof value.environment?.git_revision === 'string' && value.environment.git_revision
    && value.deterministic?.profile && !Array.isArray(value.deterministic.profile)
    && ['name', 'world_width', 'world_height', 'founders', 'seeds', 'ticks', 'food_coverage']
      .every(field => Object.hasOwn(value.deterministic.profile, field))
    ? { feature: value.feature, generated_at: value.environment.generated_at,
      git_revision: value.environment.git_revision, profile: value.deterministic.profile } : null;
  return { candidate: true, evidence: 'JSON benchmark envelope', identity, references };
}

function ancestorTest(parents) {
  const cache = new Map();
  function ancestors(oid) {
    if (!cache.has(oid)) {
      const result = new Set();
      for (const parent of parents[oid] ?? []) {
        result.add(parent);
        for (const earlier of ancestors(parent)) result.add(earlier);
      }
      cache.set(oid, result);
    }
    return cache.get(oid);
  }
  return (old, newer) => old === newer || ancestors(newer).has(old);
}

// The frozen source's full commit/tree census, shared by both inventories.
function census(repository, source, packagePath) {
  if (textGit(repository, 'rev-parse', '--is-shallow-repository') !== 'false'
    || textGit(repository, 'for-each-ref', '--format=%(refname)', 'refs/replace')) throw new Error('complete history without replacement refs is required');
  source = textGit(repository, 'rev-parse', '--verify', `${source}^{commit}`);
  const rows = textGit(repository, 'log', '--format=%H %T %P', '--reverse', '--topo-order', source).split('\n');
  const commits = rows.map(row => { const [oid, tree, ...parents] = row.trim().split(' '); return { oid, tree, parents }; });
  const ancestry = Object.fromEntries(commits.map(c => [c.oid, c.parents]));
  const isAncestor = ancestorTest(ancestry);
  const trees = new Map();
  const associations = new Map();
  let associationCount = 0;
  for (const { tree } of commits) {
    if (trees.has(tree)) continue;
    const entries = treeEntries(repository, tree);
    trees.set(tree, entries);
    associationCount += entries.length;
    for (const entry of entries.filter(e => e.type === 'blob')) {
      if (!associations.has(entry.oid)) associations.set(entry.oid, new Set());
      associations.get(entry.oid).add(entry.path);
    }
  }
  const censusPath = join(packagePath, 'census.json');
  write(censusPath, json({ commits, trees: [...trees].map(([oid, entries]) => ({ oid, entries })) }), true);
  const reachable = new Set(textGit(repository, 'rev-list', '--objects', '--no-object-names', source).split('\n'));
  const enumerated = new Set([...commits.map(c => c.oid), ...trees.keys(), ...[...trees.values()].flat().filter(e => e.type !== 'commit').map(e => e.oid)]);
  if (reachable.size !== enumerated.size || [...reachable].some(oid => !enumerated.has(oid))) throw new Error('commit/tree census does not reconcile with reachable Git objects');
  const objectsPath = join(packagePath, 'reachable-objects.txt');
  write(objectsPath, Buffer.from(`${[...reachable].sort().join('\n')}\n`), true);
  const manifest = {
    format_version: 1, frozen_source: { repository, commit: source }, package_path: packagePath,
    verified_at: new Date().toISOString(),
    enumeration: { commits: commits.length, merges: commits.filter(c => c.parents.length > 1).length,
      root_trees: trees.size,
      trees: new Set([...trees.keys(), ...[...trees.values()].flat().filter(e => e.type === 'tree').map(e => e.oid)]).size,
      unique_blobs: associations.size, tree_entries: associationCount,
      commit_tree_entries: commits.reduce((sum, c) => sum + trees.get(c.tree).length, 0),
      blob_path_associations: [...associations.values()].reduce((sum, p) => sum + p.size, 0),
      census: receipt(censusPath), reachable_objects: receipt(objectsPath) },
    ancestry, blobs: [], reports: [], exclusions: [], unresolved: [],
  };
  return { source, commits, ancestry, isAncestor, trees, associations, associationCount, manifest };
}

export async function inventory(repository, source, packagePath) {
  repository = resolve(repository);
  packagePath = resolve(packagePath);
  if (existsSync(join(packagePath, 'manifest.json'))) throw new Error('package already contains an inventory');
  const scan = census(repository, source, packagePath);
  source = scan.source;
  const { commits, isAncestor, trees, associations, manifest } = scan;
  const candidateNodes = new Map();
  // One Git batch preserves every byte without a separate process per blob.
  // The frozen corpus is bounded; exceeding the buffer fails, never truncates.
  const contentBatch = execFileSync('git', ['cat-file', '--batch'], {
    cwd: repository, input: `${[...associations.keys()].join('\n')}\n`, maxBuffer: 1024 ** 3,
  });
  let offset = 0;
  const classified = new Map();
  for (const [oid, pathSet] of associations) {
    const paths = [...pathSet].sort();
    const headerEnd = contentBatch.indexOf(10, offset);
    const [returnedId, type, size] = contentBatch.toString('ascii', offset, headerEnd).split(' ');
    const end = headerEnd + 1 + Number(size);
    if (returnedId !== oid || type !== 'blob' || !Number.isSafeInteger(Number(size)) || contentBatch[end] !== 10) throw new Error(`invalid Git blob batch at ${oid}`);
    const bytes = contentBatch.subarray(headerEnd + 1, end);
    offset = end + 1;
    const classification = classify(bytes, paths);
    classified.set(oid, { bytes, classification });
  }
  if (offset !== contentBatch.length) throw new Error('unconsumed Git blob batch bytes');
  const referencedSources = new Set([...classified].filter(([, c]) => c.classification.references?.length).map(([oid]) => oid));
  const treeBlobs = new Map([...trees].map(([oid, entries]) => [oid, new Set(entries.filter(e => e.type === 'blob').map(e => e.oid))]));
  const commitTrees = new Map(commits.map(c => [c.oid, c.tree]));
  const historyPaths = [...new Set([...associations.values()].flatMap(paths => [...paths]))];
  const references = [];
  const referencedBlobs = new Set();
  for (const sourceBlob of referencedSources) {
    const introductions = commits.filter(c => treeBlobs.get(c.tree).has(sourceBlob)
      && !c.parents.some(p => treeBlobs.get(commitTrees.get(p)).has(sourceBlob)));
    for (const reference of classified.get(sourceBlob).classification.references) {
      if (typeof reference.path !== 'string') throw new Error(`invalid stored report reference in ${sourceBlob}`);
      const matches = historyPaths.filter(path => path === reference.path.replace(/^\.\//, '') || reference.path.endsWith(`/${path}`));
      const occurrences = introductions.flatMap(c => trees.get(c.tree)
        .filter(e => e.type === 'blob' && matches.includes(e.path))
        .map(e => ({ source_commit: c.oid, path: e.path, git_blob: e.oid })));
      for (const occurrence of occurrences) referencedBlobs.add(occurrence.git_blob);
      references.push({ source_blob: sourceBlob, ...reference, history_paths: matches, occurrences,
        evidence: occurrences.length ? 'target stored when reference source entered Git' : 'target absent from source introduction trees; no external raw availability assumed' });
    }
  }
  const referencesPath = join(packagePath, 'report-references.json');
  write(referencesPath, json(references), true);
  manifest.enumeration.report_references = receipt(referencesPath);
  for (const [oid, pathSet] of associations) {
    const paths = [...pathSet].sort();
    const { bytes, classification } = classified.get(oid);
    if (!classification.candidate && referencedBlobs.has(oid)
      && classification.reason !== 'existing benchmark summary') {
      classification.candidate = true;
      classification.identity = null;
      classification.evidence = `historical report reference targets ${classification.reason}`;
    }
    const fixturePaths = paths.filter(p => /(?:^|\/)tests?\/fixtures\//.test(p));
    const reportPaths = paths.filter(p => !fixturePaths.includes(p));
    if (classification.candidate && fixturePaths.length) {
      manifest.exclusions.push({ git_blob: oid, paths: fixturePaths, reason: 'test fixture', evidence: 'report-shaped bytes stored under a test fixture directory' });
    }
    if (classification.candidate && reportPaths.length) {
      const path = join(dirname(packagePath), 'blobs', `${oid}.json`);
      write(path, bytes, true);
      manifest.blobs.push({ git_blob: oid, paths: reportPaths, identity: classification.identity,
        classification: classification.evidence, raw: { ...receipt(path), availability: 'verified_local', verified_at: manifest.verified_at },
        conversion: { status: 'pending', error: null } });
      for (const path of reportPaths) candidateNodes.set(key({ path, oid }), { path, oid, identity: classification.identity, commits: [] });
      if (fixturePaths.length) manifest.unresolved.push({ blob: oid, reason: 'report blob is also retained by an excluded fixture' });
    } else if (!classification.candidate) {
      manifest.exclusions.push({ git_blob: oid, paths, reason: classification.reason, evidence: classification.evidence });
    }
  }
  const roots = new Map([...candidateNodes.keys()].map(k => [k, k]));
  function root(k) { while (roots.get(k) !== k) k = roots.get(k); return k; }
  function unite(a, b) { roots.set(root(a), root(b)); }
  const byCommit = new Map();
  for (const commit of commits) {
    const current = new Map();
    for (const entry of trees.get(commit.tree)) {
      const node = candidateNodes.get(key(entry));
      if (node) { node.commits.push(commit.oid); current.set(entry.path, node); }
    }
    for (const parent of commit.parents) {
      const previous = byCommit.get(parent);
      for (const [path, node] of current) {
        const old = previous?.get(path);
        if (node.identity && old?.identity && canonical(node.identity) === canonical(old.identity)) unite(key(node), key(old));
        if (!old) for (const prior of previous?.values() ?? []) {
          if (node.oid === prior.oid) unite(key(node), key(prior));
        }
      }
    }
    byCommit.set(commit.oid, current);
  }
  const groups = new Map();
  for (const [k, node] of candidateNodes) {
    const group = root(k);
    if (!groups.has(group)) groups.set(group, []);
    groups.get(group).push(node);
  }
  const occurrencesPath = join(packagePath, 'occurrences.json');
  write(occurrencesPath, json([...candidateNodes.values()].map(n => ({ path: n.path, git_blob: n.oid, commits: n.commits }))), true);
  manifest.enumeration.report_occurrences = receipt(occurrencesPath);
  for (const nodes of groups.values()) {
    const paths = [...new Set(nodes.map(n => n.path))].sort();
    const versionNodes = new Map();
    for (const node of nodes) {
      if (!versionNodes.has(node.oid)) versionNodes.set(node.oid, []);
      versionNodes.get(node.oid).push(node);
    }
    const versions = [...versionNodes].map(([oid, entries]) => {
      const occurrences = [...new Set(entries.flatMap(n => n.commits))].sort();
      return { git_blob: oid, paths: entries.map(n => n.path).sort(), occurrence_count: occurrences.length,
        occurrence_sha256: sha256(json(occurrences)),
        first_commits: occurrences.filter(c => !occurrences.some(d => c !== d && isAncestor(d, c))),
        last_commits: occurrences.filter(c => !occurrences.some(d => c !== d && isAncestor(c, d))) };
    });
    const tipPaths = nodes.filter(n => n.commits.includes(source)).map(n => ({ path: n.path, git_blob: n.oid }));
    manifest.reports.push({ logical_id: sha256(json(nodes.map(key).sort())).slice(0, 20),
      identity: nodes.find(n => n.identity)?.identity ?? null, paths, versions, tip_paths: tipPaths,
      historical_only: tipPaths.length === 0, selected_blob: null, selection_reason: 'conversion pending', summary: null });
  }
  manifest.reports.sort((a, b) => a.logical_id.localeCompare(b.logical_id));
  const filterPath = join(packagePath, 'filter-inputs.json');
  write(filterPath, json(manifest.blobs.flatMap(b => b.paths.map(path => ({ path, git_blob: b.git_blob })))), true);
  manifest.filter_inputs = receipt(filterPath);
  selectReports(manifest);
  write(join(packagePath, 'manifest.json'), json(manifest));
  return manifest;
}

// Large-file cleanup Phase B (docs/specs/large-file-cleanup-2026-09-24.md):
// the blob IDs over `minBytes` at `prefixes` anywhere in the ancestry of
// `source`, except blobs present in its own tip tree.
export function largeBlobIds(repository, source, { minBytes = 1024 ** 2, prefixes }) {
  const commits = textGit(repository, 'log', '--format=%T', source).split('\n');
  const tip = new Set(treeEntries(repository, commits[0]).map(e => e.oid));
  const sized = new Map();
  for (const tree of new Set(commits)) {
    for (const row of git(repository, 'ls-tree', '-r', '-l', '-z', '--full-tree', tree).toString('utf8').split('\0').filter(Boolean)) {
      const tab = row.indexOf('\t');
      const [, type, oid, size] = row.slice(0, tab).split(/ +/);
      const path = row.slice(tab + 1);
      if (type === 'blob' && Number(size) > minBytes && prefixes.some(p => path.startsWith(p)) && !tip.has(oid)) sized.set(oid, true);
    }
  }
  return [...sized.keys()].sort();
}

// An explicit blob-ID inventory: every path and commit of each listed blob,
// its bytes and SHA-256, a byte-identical copy beside the package, and one
// filter input per path so a filter callback turns each pair into a deletion.
// The manifest has the shape `verifyRaw` and `verifyHistory` read.
export function blobInventory(repository, source, packagePath, blobIds) {
  repository = resolve(repository);
  packagePath = resolve(packagePath);
  if (existsSync(join(packagePath, 'manifest.json'))) throw new Error('package already contains an inventory');
  const ids = [...new Set(blobIds)];
  if (!ids.length || ids.length !== blobIds.length || ids.some(id => !/^[0-9a-f]{40}$/.test(id))) {
    throw new Error('blob inventory needs distinct 40-hex blob IDs');
  }
  const scan = census(repository, source, packagePath);
  const { commits, trees, associations, manifest } = scan;
  const tip = new Set(trees.get(commits.find(c => c.oid === scan.source).tree).map(e => e.oid));
  for (const id of ids) {
    if (!associations.has(id)) throw new Error(`blob not in the frozen ancestry: ${id}`);
    if (tip.has(id)) throw new Error(`blob present at the frozen tip: ${id}`);
  }
  const contents = execFileSync('git', ['cat-file', '--batch'], { cwd: repository, input: `${ids.join('\n')}\n`, maxBuffer: 1024 ** 3 });
  let offset = 0;
  for (const id of ids) {
    const headerEnd = contents.indexOf(10, offset);
    const [returnedId, type, size] = contents.toString('ascii', offset, headerEnd).split(' ');
    const end = headerEnd + 1 + Number(size);
    if (returnedId !== id || type !== 'blob' || contents[end] !== 10) throw new Error(`invalid Git blob batch at ${id}`);
    const path = join(dirname(packagePath), 'blobs', id);
    write(path, contents.subarray(headerEnd + 1, end), true);
    offset = end + 1;
    const paths = [...associations.get(id)].sort();
    manifest.blobs.push({ git_blob: id, paths,
      occurrences: paths.map(p => ({ path: p, commits: commits.filter(c => trees.get(c.tree).some(e => e.path === p && e.oid === id)).map(c => c.oid) })),
      raw: { ...receipt(path), availability: 'verified_local', verified_at: manifest.verified_at } });
  }
  const filterPath = join(packagePath, 'filter-inputs.json');
  write(filterPath, json(manifest.blobs.flatMap(b => b.paths.map(path => ({ path, git_blob: b.git_blob })))), true);
  manifest.filter_inputs = receipt(filterPath);
  manifest.totals = { blobs: manifest.blobs.length, bytes: manifest.blobs.reduce((sum, b) => sum + b.raw.bytes, 0),
    filter_inputs: manifest.blobs.reduce((sum, b) => sum + b.paths.length, 0) };
  write(join(packagePath, 'manifest.json'), json(manifest));
  return manifest;
}

export function selectReports(manifest) {
  const isAncestor = ancestorTest(manifest.ancestry);
  manifest.unresolved = manifest.unresolved.filter(u => !u.logical_id);
  for (const report of manifest.reports) {
    report.selected_blob = null;
    if (!report.identity) {
      report.selection_reason = 'incomplete measured identity';
      manifest.unresolved.push({ logical_id: report.logical_id, reason: report.selection_reason });
      continue;
    }
    const valid = report.versions.filter(v => manifest.blobs.find(b => b.git_blob === v.git_blob).conversion.status === 'converted');
    if (!valid.length) { report.selection_reason = 'no valid converted version'; continue; }
    const tips = valid.filter(v => report.tip_paths.some(p => p.git_blob === v.git_blob));
    const final = tips.length ? tips : valid.filter(v => !valid.some(other => other !== v
      && v.last_commits.every(old => other.last_commits.some(newer => old !== newer && isAncestor(old, newer)))));
    if (final.length !== 1) {
      report.selection_reason = 'incomparable final valid versions require a recorded resolution';
      manifest.unresolved.push({ logical_id: report.logical_id, reason: report.selection_reason });
      continue;
    }
    report.selected_blob = final[0].git_blob;
    report.selection_reason = tips.length ? 'valid version present at frozen tip' : 'final valid historical version by ancestry';
  }
}

export function verifyRaw(manifest) {
  for (const blob of manifest.blobs) {
    const bytes = readFileSync(blob.raw.path);
    const oid = createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex');
    if (oid !== blob.git_blob || sha256(bytes) !== blob.raw.sha256 || bytes.length !== blob.raw.bytes) {
      throw new Error(`raw integrity failure: ${blob.git_blob}`);
    }
  }
  for (const exported of [manifest.enumeration.census, manifest.enumeration.reachable_objects,
    manifest.enumeration.report_occurrences, manifest.enumeration.report_references, manifest.filter_inputs].filter(Boolean)) {
    const actual = receipt(exported.path);
    if (actual.sha256 !== exported.sha256 || actual.bytes !== exported.bytes) throw new Error(`export integrity failure: ${exported.path}`);
  }
  return { raw_blobs: manifest.blobs.length, raw_bytes: manifest.blobs.reduce((sum, b) => sum + b.raw.bytes, 0) };
}

export function verifyPackage(manifest) {
  const totals = verifyRaw(manifest);
  const paths = new Set();
  for (const report of manifest.reports) {
    if (!report.selected_blob) {
      const disposition = report.raw_only_disposition;
      if (!disposition?.reason || !disposition.sources?.length || !['unparsable', 'unsupported'].includes(disposition.status)
        || report.versions.some(v => !manifest.blobs.find(b => b.git_blob === v.git_blob).conversion.error)) {
        throw new Error(`missing explicit raw-only disposition: ${report.logical_id}`);
      }
      continue;
    }
    if (!report.summary) throw new Error(`missing summary: ${report.logical_id}`);
    const actual = receipt(report.summary.path);
    if (actual.sha256 !== report.summary.sha256 || actual.bytes !== report.summary.bytes) throw new Error(`summary integrity failure: ${report.logical_id}`);
    if (paths.has(report.summary.repository_path)) throw new Error(`summary path collision: ${report.summary.repository_path}`);
    paths.add(report.summary.repository_path);
    const blob = manifest.blobs.find(b => b.git_blob === report.selected_blob);
    const summary = JSON.parse(readFileSync(report.summary.path));
    if (summary.kind !== 'petri-benchmark-summary' || summary.summary_version !== SUMMARY_VERSION
      || summary.raw.sha256 !== blob.raw.sha256 || summary.raw.bytes !== blob.raw.bytes || summary.raw.path !== blob.raw.path
      || !report.summary.repeat_identical
      || canonical({ feature: summary.feature, generated_at: summary.environment.generated_at,
        git_revision: summary.environment.git_revision, profile: summary.deterministic.profile }) !== canonical(report.identity)) {
      throw new Error(`summary provenance or identity mismatch: ${report.logical_id}`);
    }
    const provenance = blob.conversion.provenance;
    if (receipt(provenance.path).sha256 !== provenance.sha256
      || canonical(summary.conversion) !== canonical(JSON.parse(readFileSync(provenance.path)))) {
      throw new Error(`conversion provenance mismatch: ${report.logical_id}`);
    }
  }
  if (manifest.unresolved.length) throw new Error(`${manifest.unresolved.length} unresolved inventory decisions`);
  return { ...totals, verified_summaries: paths.size };
}

export function verifyHistory(manifest, repository, tip, commitMapPath) {
  const rawIds = new Set(manifest.blobs.map(b => b.git_blob));
  const commits = textGit(repository, 'log', '--format=%H %T', tip).split('\n').map(row => row.split(' '));
  const trees = new Map();
  for (const [, tree] of commits) {
    if (!trees.has(tree)) trees.set(tree, treeEntries(repository, tree));
    for (const entry of trees.get(tree)) if (rawIds.has(entry.oid)) throw new Error(`raw blob remains in rewritten main: ${entry.path} ${entry.oid}`);
  }
  if (commitMapPath) {
    const reachable = new Set(commits.map(([oid]) => oid));
    const mapping = new Map(readFileSync(commitMapPath, 'utf8').trim().split('\n').slice(1).map(row => row.trim().split(/\s+/)));
    const census = JSON.parse(readFileSync(manifest.enumeration.census.path));
    const originalTrees = new Map(census.trees.map(t => [t.oid, t.entries]));
    const filtered = new Set(JSON.parse(readFileSync(manifest.filter_inputs.path)).map(p => `${p.path}\0${p.git_blob}`));
    for (const commit of census.commits) {
      const mapped = mapping.get(commit.oid);
      if (!mapped || /^0+$/.test(mapped)) throw new Error(`missing preserved commit: ${commit.oid}`);
      if (!reachable.has(mapped)) throw new Error(`mapped commit is not reachable from candidate: ${commit.oid} -> ${mapped}`);
      const expected = originalTrees.get(commit.tree).filter(e => e.type !== 'tree' && !filtered.has(key(e)));
      const actual = treeEntries(repository, mapped).filter(e => e.type !== 'tree');
      if (canonical(expected) !== canonical(actual)) throw new Error(`unrelated tree changed at ${commit.oid} -> ${mapped}`);
      const parents = textGit(repository, 'show', '-s', '--format=%P', mapped).split(' ').filter(Boolean);
      if (canonical(parents) !== canonical(commit.parents.map(p => mapping.get(p)))) throw new Error(`commit topology changed at ${commit.oid}`);
      const format = '--format=%an%x00%ae%x00%at%x00%aI%x00%cn%x00%ce%x00%ct%x00%cI%x00%B';
      if (!git(repository, 'show', '-s', format, mapped).equals(git(manifest.frozen_source.repository, 'show', '-s', format, commit.oid))) {
        throw new Error(`commit metadata or message changed at ${commit.oid}`);
      }
    }
  }
  return { commits: commits.length, trees: trees.size, raw_blobs_absent: rawIds.size, mapped_trees_checked: Boolean(commitMapPath) };
}

export function convertReports(manifest, executable) {
  verifyRaw(manifest);
  const packagePath = manifest.package_path;
  const converter = join(packagePath, 'converter', 'v3-cli');
  if (!existsSync(converter)) {
    mkdirSync(dirname(converter), { recursive: true });
    copyFileSync(resolve(executable), converter);
    chmodSync(converter, statSync(executable).mode);
  } else if (sha256(readFileSync(converter)) !== sha256(readFileSync(executable))) throw new Error('converter differs from the preserved executable');
  manifest.converter = receipt(converter);
  const invoke = blob => {
    const output = join(packagePath, 'versions', `${blob.git_blob}.json`);
    const provenancePath = join(packagePath, 'provenance', `${blob.git_blob}.json`);
    const args = ['bench-summarize', '--input', blob.raw.path, '--out', output, '--provenance', provenancePath];
    const provenance = { verified_at: manifest.verified_at,
      converter: { executable: converter, arguments: args, working_directory: manifest.frozen_source.repository }, supplied_evidence: null };
    write(provenancePath, json(provenance), true);
    const run = spawnSync(converter, args, { cwd: manifest.frozen_source.repository, encoding: 'utf8', maxBuffer: 1024 ** 2 });
    if (run.error) throw run.error;
    return { output, provenance: receipt(provenancePath), exit: run.status, error: run.status === 0 ? null : run.stderr.trim() };
  };
  for (const blob of manifest.blobs) {
    const run = invoke(blob);
    blob.conversion = { status: run.exit === 0 ? 'converted' : blob.classification === 'JSON benchmark envelope' ? 'unsupported' : 'unparsable',
      exit: run.exit, error: run.error, provenance: run.provenance, summary: run.exit === 0 ? receipt(run.output) : null };
  }
  selectReports(manifest);
  for (const report of manifest.reports) {
    if (!report.selected_blob) continue;
    const blob = manifest.blobs.find(b => b.git_blob === report.selected_blob);
    const first = readFileSync(blob.conversion.summary.path);
    const repeat = invoke(blob);
    if (repeat.exit !== 0 || !first.equals(readFileSync(repeat.output))) throw new Error(`non-repeatable conversion: ${report.logical_id}`);
    const slug = report.identity.feature.replace(/[^a-zA-Z0-9-]/g, '-').slice(0, 80);
    const path = report.tip_paths.map(p => p.path).sort()[0]
      ?? `docs/progress/features/historical/${slug}-${report.logical_id}.json`;
    const exported = join(packagePath, 'summaries', path);
    write(exported, first, true);
    report.summary = { repository_path: path, ...receipt(exported), repeat_identical: true };
  }
  manifest.totals = { ...verifyRaw(manifest), logical_reports: manifest.reports.length,
    summaries: manifest.reports.filter(r => r.summary).length,
    summary_bytes: manifest.reports.reduce((sum, r) => sum + (r.summary?.bytes ?? 0), 0),
    raw_only_reports: manifest.reports.filter(r => !r.summary).length, unresolved: manifest.unresolved.length };
  write(join(packagePath, 'manifest.json'), json(manifest));
  return manifest.totals;
}

export function exportSummaries(manifest) {
  verifyPackage(manifest);
  const repository = manifest.frozen_source.repository;
  if (textGit(repository, 'branch', '--show-current') === 'main') throw new Error('the main checkout must not be changed before cutover');
  const original = new Map(manifest.reports.flatMap(r => r.tip_paths.map(p => [p.path, p.git_blob])));
  const outputs = new Map([...original.keys()].map(path => [path, null]));
  for (const report of manifest.reports.filter(r => r.summary)) outputs.set(report.summary.repository_path, readFileSync(report.summary.path));
  for (const [path, bytes] of outputs) {
    const target = resolve(repository, path);
    if (!target.startsWith(`${repository}/`)) throw new Error(`export path escapes checkout: ${path}`);
    if (!existsSync(target)) continue;
    const current = readFileSync(target);
    if (bytes?.equals(current)) continue;
    const oid = createHash('sha1').update(`blob ${current.length}\0`).update(current).digest('hex');
    if (oid !== original.get(path)) throw new Error(`checkout file changed: ${path}`);
  }
  const seriesPath = join(repository, 'docs/progress/benchmark-series.json');
  let seriesReferences = 0;
  if (existsSync(seriesPath)) {
    const series = JSON.parse(readFileSync(seriesPath));
    for (const section of Object.values(series)) {
      if (!section || Array.isArray(section) || typeof section !== 'object') continue;
      for (const path of [...(section.closed ?? []), ...(section.epoch_baseline ? [section.epoch_baseline] : [])]) {
        const bytes = outputs.has(path) ? outputs.get(path) : readFileSync(join(repository, path));
        if (!bytes) throw new Error(`series reference has no supported summary: ${path}`);
        const summary = JSON.parse(bytes);
        if (summary.kind !== 'petri-benchmark-summary' || summary.summary_version !== SUMMARY_VERSION) throw new Error(`series reference is not a supported summary: ${path}`);
        seriesReferences += 1;
      }
    }
  }
  const removed = [];
  for (const [path, bytes] of outputs) {
    const target = join(repository, path);
    if (bytes) write(target, bytes);
    else if (existsSync(target)) { unlinkSync(target); removed.push(path); }
  }
  write(join(repository, 'docs/progress/historical-benchmark-manifest.json'), json(manifest));
  return { exported_summaries: manifest.reports.filter(r => r.summary).length,
    removed_raw_paths: removed, verified_series_references: seriesReferences };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const [command, ...args] = process.argv.slice(2);
    let result;
    if (command === 'large-blobs' && args.length >= 2) {
      result = largeBlobIds(process.cwd(), args[0], { prefixes: args.slice(1) });
    } else if (command === 'inventory-blobs' && args.length === 3) {
      // BLOB_IDS is the JSON array `large-blobs` prints.
      const ids = JSON.parse(readFileSync(args[2], 'utf8'));
      if (!Array.isArray(ids)) throw new Error('BLOB_IDS must be the JSON array large-blobs prints');
      result = blobInventory(process.cwd(), args[0], args[1], ids).totals;
    } else if (command === 'inventory' && args.length === 2) {
      const manifest = await inventory(process.cwd(), args[0], args[1]);
      result = { enumeration: manifest.enumeration, candidate_blobs: manifest.blobs.length, reports: manifest.reports.length, unresolved: manifest.unresolved };
    } else if (['convert', 'verify', 'verify-history', 'export'].includes(command)) {
      const manifest = JSON.parse(readFileSync(join(resolve(args[0]), 'manifest.json')));
      if (command === 'convert' && args.length === 2) result = convertReports(manifest, args[1]);
      else if (command === 'verify' && args.length === 1) result = verifyPackage(manifest);
      else if (command === 'export' && args.length === 1) result = exportSummaries(manifest);
      else if (command === 'verify-history' && args.length >= 3 && args.length <= 4) result = verifyHistory(manifest, args[1], args[2], args[3]);
    }
    if (!result) throw new Error('usage: historical-benchmarks.mjs inventory SOURCE PACKAGE | large-blobs SOURCE PREFIX... | inventory-blobs SOURCE PACKAGE BLOB_IDS | convert PACKAGE EXECUTABLE | verify PACKAGE | export PACKAGE | verify-history PACKAGE REPOSITORY TIP [COMMIT_MAP]');
    console.log(JSON.stringify(result, null, 2));
  } catch (error) {
    console.error(`historical-benchmarks: ${error.message}`);
    process.exitCode = 1;
  }
}
