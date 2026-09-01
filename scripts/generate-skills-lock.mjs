import { createHash } from 'node:crypto';
import { readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { join, relative } from 'node:path';
const root = '.agents/skills'; const files = {};
function walk(dir) { for (const name of readdirSync(dir).sort()) { const full = join(dir, name); const rel = relative(root, full); if (statSync(full).isDirectory()) walk(full); else files[rel] = createHash('sha256').update(readFileSync(full)).digest('hex'); } }
walk(root);
const sources = {
  'project-workflow': { repository: 'iadknet/new_app_scaffolding_skills', commit: '83ef0fb281a17c28c584a410b371d1afe7c6b37b', path: 'skills/agent-project-scaffold/assets/project-skills/project-workflow', license: 'unknown' },
  'prd-create': { repository: 'iadknet/new_app_scaffolding_skills', commit: '83ef0fb281a17c28c584a410b371d1afe7c6b37b', path: 'skills/agent-project-scaffold/assets/project-skills/prd-create', license: 'unknown' },
  'prd-review': { repository: 'iadknet/new_app_scaffolding_skills', commit: '83ef0fb281a17c28c584a410b371d1afe7c6b37b', path: 'skills/agent-project-scaffold/assets/project-skills/prd-review', license: 'unknown' },
  'prd-implement': { repository: 'iadknet/new_app_scaffolding_skills', commit: '83ef0fb281a17c28c584a410b371d1afe7c6b37b', path: 'skills/agent-project-scaffold/assets/project-skills/prd-implement', license: 'unknown' },
  rust: { repository: 'joshuadavidthomas/agent-skills', commit: '516dee7a422b90937b2958d11c03694154ab9c09', path: 'rust', license: 'unknown' },
  'react-best-practices': { repository: 'vercel-labs/agent-skills', commit: '063bee94c3f4df8453406c830b0a7df0f2860278', path: 'skills/react-best-practices', license: 'MIT' },
  'composition-patterns': { repository: 'vercel-labs/agent-skills', commit: '063bee94c3f4df8453406c830b0a7df0f2860278', path: 'skills/composition-patterns', license: 'MIT' },
};
const entries = Object.entries(files).map(([path, sha256]) => ({ path, sha256 }));
writeFileSync('.agents/skills.lock.json', JSON.stringify({ version: 1, sources, excluded_files: ['nested AGENTS.md and CLAUDE.md; gh installation metadata'], files: entries }, null, 2) + '\n');
