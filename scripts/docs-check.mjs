import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const files = [];
function walk(directory) {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const fullPath = path.join(directory, entry.name);
    if (entry.isDirectory()) walk(fullPath);
    else if (entry.name.endsWith('.md')) files.push(fullPath);
  }
}
walk(path.join(root, 'docs'));
files.push(path.join(root, 'README.md'), path.join(root, 'src-tauri/capabilities/README.md'));
const broken = [];
for (const file of files) {
  const source = fs.readFileSync(file, 'utf8');
  for (const match of source.matchAll(/\]\(([^\s)]+)(?:\s+"[^"]*")?\)/g)) {
    const target = match[1].split('#')[0];
    if (!target || /^[a-z]+:|^\//i.test(target)) continue;
    if (!fs.existsSync(path.resolve(path.dirname(file), decodeURIComponent(target)))) {
      broken.push(`${path.relative(root, file)} -> ${target}`);
    }
  }
}
if (broken.length) {
  console.error(`Enlaces locales rotos:\n${broken.join('\n')}`);
  process.exitCode = 1;
} else console.log(`Documentacion: ${files.length} Markdown con enlaces locales validos.`);