import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const commandsDirectory = path.join(root, "src-tauri", "src", "commands");
const registryPath = path.join(root, "src-tauri", "src", "api", "invoke.rs");
const frontendBackendPath = path.join(
  root,
  "solid",
  "src",
  "services",
  "backend",
  "tauri-backend.service.ts",
);

function unique(values) {
  return [...new Set(values)].sort();
}

function duplicates(values) {
  const seen = new Set();
  const repeated = new Set();
  for (const value of values) {
    if (seen.has(value)) repeated.add(value);
    seen.add(value);
  }
  return [...repeated].sort();
}

const commandFiles = fs
  .readdirSync(commandsDirectory)
  .filter((name) => name.endsWith(".rs") && name !== "mod.rs")
  .map((name) => path.join(commandsDirectory, name));

const declaredCommands = [];
for (const file of commandFiles) {
  const source = fs.readFileSync(file, "utf8");
  for (const match of source.matchAll(
    /#\[tauri::command\]\s*(?:pub\s+)?(?:async\s+)?fn\s+([a-zA-Z0-9_]+)/g,
  )) {
    declaredCommands.push(match[1]);
  }
}

const registry = fs.readFileSync(registryPath, "utf8");
const registeredCommands = [
  ...registry.matchAll(/crate::commands::[a-z_]+::([a-zA-Z0-9_]+)/g),
].map((match) => match[1]);

const frontend = fs.readFileSync(frontendBackendPath, "utf8");
const invokedCommands = [
  ...frontend.matchAll(/this\.invoke(?:<[^>]+>)?\(\s*["']([^"']+)["']/g),
].map((match) => match[1]);

const declared = unique(declaredCommands);
const registered = unique(registeredCommands);
const invoked = unique(invokedCommands);

const problems = [];
for (const command of duplicates(declaredCommands))
  problems.push(`Comando Rust duplicado: ${command}`);
for (const command of duplicates(registeredCommands))
  problems.push(`Comando registrado duplicado: ${command}`);
for (const command of duplicates(invokedCommands))
  problems.push(`Invoke frontend duplicado: ${command}`);

for (const command of declared) {
  if (!registered.includes(command))
    problems.push(`Comando Rust no registrado: ${command}`);
}
for (const command of registered) {
  if (!declared.includes(command))
    problems.push(`Comando registrado sin #[tauri::command]: ${command}`);
}
for (const command of invoked) {
  if (!registered.includes(command))
    problems.push(`Frontend invoca comando no registrado: ${command}`);
}

if (problems.length > 0) {
  console.error(`Contrato IPC invalido:\n${problems.join("\n")}`);
  process.exitCode = 1;
} else {
  console.log(
    `Contrato IPC valido: ${declared.length} comandos Rust, ${registered.length} registrados, ${invoked.length} invocados desde frontend.`,
  );
}
