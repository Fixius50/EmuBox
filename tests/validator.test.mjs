import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const config = ts.readConfigFile(
  path.join(root, "tsconfig.json"),
  ts.sys.readFile,
);
assert.equal(config.error, undefined);
const parsed = ts.parseJsonConfigFileContent(config.config, ts.sys, root);
assert.deepEqual(parsed.errors, []);
const expected = ts.sys.readDirectory(
  root,
  [".ts", ".tsx", ".mts", ".cts"],
  ["**/node_modules/**", "**/dist/**", "**/target/**"],
  [
    "*.ts",
    "*.mts",
    "*.cts",
    "solid/**/*",
    "tests/**/*",
    "data/**/*",
    "benchmarks/**/*",
  ],
);
const included = new Set(parsed.fileNames.map((file) => path.resolve(file)));
for (const file of expected)
  assert.ok(
    included.has(path.resolve(file)),
    `Archivo fuera de tsconfig: ${file}`,
  );
assert.ok(included.has(path.join(root, "solid/vite.config.ts")));
assert.ok(included.has(path.join(root, "solid/src/App.tsx")));
console.log(
  `Cobertura TypeScript: ${included.size} archivos incluidos; no depende del editor.`,
);

const directory = mkdtempSync(path.join(tmpdir(), "emubox-validator-"));
try {
  const closed = path.join(directory, "closed.ts");
  const jsx = path.join(directory, "closed.tsx");
  writeFileSync(closed, "export const title: string = 123;\n");
  writeFileSync(jsx, "export const View = () => <main />;\n");
  const program = ts.createProgram([closed, jsx], {
    noEmit: true,
    types: [],
    jsx: ts.JsxEmit.Preserve,
    jsxImportSource: "missing-jsx-runtime-fixture",
    module: ts.ModuleKind.ESNext,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
  });
  const diagnostics = ts.getPreEmitDiagnostics(program);
  assert.ok(
    diagnostics.some(
      (diagnostic) =>
        diagnostic.code === 2322 && diagnostic.file?.fileName === closed,
    ),
  );
  assert.ok(
    diagnostics.some(
      (diagnostic) =>
        diagnostic.code === 2875 && diagnostic.file?.fileName === jsx,
    ),
  );
  console.log(
    "Detectados error de tipos y jsx-runtime ausente en archivos nunca abiertos en VS Code.",
  );
} finally {
  rmSync(directory, { recursive: true, force: true });
}
