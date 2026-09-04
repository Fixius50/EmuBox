import { readFileSync, readdirSync, statSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const ignored = new Set(['.git', 'node_modules', 'dist', 'reports', 'target']);
const textExtensions = new Set(['.js', '.mjs', '.cjs', '.ts', '.tsx', '.json', '.md', '.css', '.html', '.sh']);
const suspicious = /(?:\uFFFD|\u00c3.|\u00c2.|\u00e2\u20ac[\u2122\u0153\u009d\u0093])/u;
const failures = [];

function visit(directory) {
  for (const entry of readdirSync(directory)) {
    if (ignored.has(entry)) continue;
    const filePath = path.join(directory, entry);
    const stats = statSync(filePath);
    if (stats.isDirectory()) {
      visit(filePath);
    } else if (textExtensions.has(path.extname(entry).toLowerCase())) {
      const content = readFileSync(filePath, 'utf8');
      if (suspicious.test(content)) failures.push(path.relative(root, filePath));
    }
  }
}

visit(root);
if (failures.length > 0) {
  console.error(`Texto sospechoso en: ${failures.join(', ')}`);
  process.exitCode = 1;
} else {
  console.log('No se detectó texto mal codificado.');
}
