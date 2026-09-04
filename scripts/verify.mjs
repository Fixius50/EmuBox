import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const frontendRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
);
const npmCli =
  process.env.npm_execpath ||
  path.join(
    path.dirname(process.execPath),
    "node_modules",
    "npm",
    "bin",
    "npm-cli.js",
  );
const npmCommand =
  process.platform === "win32" || Boolean(process.env.npm_execpath)
    ? process.execPath
    : "npm";
const reportsDirectory = path.join(frontendRoot, "reports", "verify");

const checks = [
  ["ESLint", ["run", "lint"]],
  ["Prettier", ["run", "format:check"]],
  ["TypeScript", ["run", "typecheck"]],
  ["Arquitectura", ["run", "arch:check"]],
  ["Texto", ["run", "quality:text"]],
  ["Tests", ["test"]],
];

const failedChecks = [];
mkdirSync(reportsDirectory, { recursive: true });

for (const [name, argumentsList] of checks) {
  console.log(`\n[verify] ${name}`);
  const result = spawnSync(
    npmCommand,
    npmCommand === process.execPath
      ? [npmCli, ...argumentsList]
      : argumentsList,
    { cwd: frontendRoot, shell: false, encoding: "utf8", timeout: 180000 },
  );
  const output = [result.stdout, result.stderr, result.error?.message]
    .filter(Boolean)
    .join("\n");
  const reportName = name.toLowerCase().replaceAll(" ", "-");
  const reportPath = path.join(reportsDirectory, `${reportName}.log`);
  writeFileSync(reportPath, output || "Sin salida.\n", "utf8");
  if (output) process.stdout.write(`${output}\n`);
  if (result.status !== 0) {
    failedChecks.push(name);
    console.error(`[verify] Informe: ${reportPath}`);
  }
}

if (failedChecks.length > 0) {
  console.error(`\n[verify] Fallaron: ${failedChecks.join(", ")}`);
  process.exitCode = 1;
} else {
  console.log("\n[verify] Todos los controles pasaron.");
}
