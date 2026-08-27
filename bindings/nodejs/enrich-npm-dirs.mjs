import { readdirSync, readFileSync, writeFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';

const npmRoot = join(import.meta.dirname, 'npm');

// Derive the scope from package.json rather than hardcoding it, so the platform
// packages cannot drift from the package they belong to.
const { name } = JSON.parse(
  readFileSync(join(import.meta.dirname, 'package.json'), 'utf8'),
);
const scope = name.startsWith('@') ? name.split('/')[0] : '';
const base = scope ? `${scope}/zen-engine` : 'zen-engine';

for (const dir of readdirSync(npmRoot)) {
  const readmePath = join(npmRoot, dir, 'README.md');
  if (!existsSync(readmePath)) continue;
  const readme = readFileSync(readmePath, 'utf8');
  const target = readme.match(/This is the \*\*(.+?)\*\* binary/)?.[1] ?? dir;
  writeFileSync(
    readmePath,
    `# \`${base}-${dir}\`

This is the **${target}** binary for [\`${base}\`](https://www.npmjs.com/package/${base}), the Node.js rules engine from [phenixrizen/zen](https://github.com/phenixrizen/zen) — a maintained fork of \`gorules/zen\`.

- [GitHub](https://github.com/phenixrizen/zen)
`,
  );
}
