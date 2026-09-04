import { mkdirSync, writeFileSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";
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

// Comprueba en disco (nunca contra la caché del servidor TS del editor) que las
// dependencias declaradas resuelven de verdad. Falla rápido y con un mensaje
// accionable en vez de dejar que aparezca luego como un críptico error de JSX.
function checkDependencies() {
  const manifest = JSON.parse(
    readFileSync(path.join(frontendRoot, "package.json"), "utf8"),
  );
  const packageNames = Object.keys({
    ...manifest.dependencies,
    ...manifest.devDependencies,
  });
  const require = createRequire(path.join(frontendRoot, "package.json"));
  const subpathsToCheck = {
    "solid-js": ["solid-js/jsx-runtime"],
  };

  const missing = [];
  for (const name of packageNames) {
    // Los paquetes @types/* solo contienen declaraciones: no tienen un punto de
    // entrada JS que require.resolve pueda encontrar, así que se valida en disco.
    if (name.startsWith("@types/")) {
      try {
        readFileSync(
          path.join(frontendRoot, "node_modules", name, "package.json"),
        );
      } catch {
        missing.push(name);
      }
      continue;
    }
    try {
      require.resolve(name);
    } catch {
      missing.push(name);
      continue;
    }
    for (const subpath of subpathsToCheck[name] || []) {
      try {
        require.resolve(subpath);
      } catch {
        missing.push(subpath);
      }
    }
  }

  return missing;
}

// TypeScript incluye implícitamente TODO node_modules/@types (directos y
// transitivos) cuando "types" no está fijado en tsconfig.json. Si alguno de
// esos paquetes quedó a medio instalar, tsc falla con "no se puede encontrar
// el archivo de definición de tipo para X". Se valida en disco, paquete por
// paquete, en vez de confiar en la caché del servidor TS del editor.
function checkImplicitTypeLibraries() {
  const typesDirectory = path.join(frontendRoot, "node_modules", "@types");
  let entries;
  try {
    entries = readdirSync(typesDirectory, { withFileTypes: true });
  } catch {
    return [];
  }

  const broken = [];
  for (const entry of entries) {
    if (!entry.isDirectory()) continue;
    const packageDir = path.join(typesDirectory, entry.name);
    let manifest;
    try {
      manifest = JSON.parse(
        readFileSync(path.join(packageDir, "package.json"), "utf8"),
      );
    } catch {
      broken.push(entry.name);
      continue;
    }
    const typesEntry = manifest.types || manifest.typings || "index.d.ts";
    try {
      readFileSync(path.join(packageDir, typesEntry));
    } catch {
      broken.push(entry.name);
    }
  }

  return broken;
}

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

console.log("\n[verify] Dependencias");
const missingDependencies = [
  ...checkDependencies(),
  ...checkImplicitTypeLibraries().map((name) => `@types/${name} (incompleto)`),
];
const dependenciesReportPath = path.join(reportsDirectory, "dependencias.log");
if (missingDependencies.length > 0) {
  const message = `No resuelven en node_modules: ${missingDependencies.join(", ")}.\nEjecuta "npm ci" para reinstalar de forma limpia y vuelve a intentarlo.\n`;
  writeFileSync(dependenciesReportPath, message, "utf8");
  process.stderr.write(message);
  failedChecks.push("Dependencias");
  console.error(`[verify] Informe: ${dependenciesReportPath}`);
} else {
  writeFileSync(
    dependenciesReportPath,
    "Todas las dependencias resuelven correctamente.\n",
    "utf8",
  );
  console.log("Todas las dependencias resuelven correctamente.");
}

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
