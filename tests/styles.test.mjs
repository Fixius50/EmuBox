import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import test from "node:test";
import prettier from "prettier";
import * as cssPlugin from "prettier/plugins/postcss";

const directory = new URL("../solid/src/styles/", import.meta.url);
const files = readdirSync(directory).filter((name) => name.endsWith(".css"));
const runtimeTokens = new Set([
  "--category-index",
  "--row-offset",
  "--game-offset",
]);
const definedTokens = new Set(runtimeTokens);
const styles = [];

function walk(node, visit) {
  if (!node || typeof node !== "object") return;
  if (Array.isArray(node)) {
    for (const child of node) walk(child, visit);
    return;
  }
  visit(node);
  for (const key of ["nodes", "value", "group", "groups"])
    walk(node[key], visit);
}

for (const file of files) {
  const source = readFileSync(new URL(file, directory), "utf8");
  const { ast } = await prettier.__debug.parse(source, {
    parser: "css",
    plugins: [cssPlugin],
  });
  styles.push({ file, ast });
  walk(ast, (node) => {
    if (node.type === "css-decl" && node.prop.startsWith("--"))
      definedTokens.add(node.prop);
  });
}

for (const { file, ast } of styles) {
  test(`CSS: ${file} uses relative units, explicit transitions and defined tokens`, () => {
    assert.doesNotMatch(
      readFileSync(new URL(file, directory), "utf8"),
      /&-[A-Za-z0-9_-]/,
      `${file}: CSS nesting does not support Sass-style selector concatenation`,
    );
    walk(ast, (node) => {
      if (node.type === "value-number")
        assert.notEqual(node.unit, "px", `${file}: use relative units`);
      if (node.type === "css-decl" && node.prop === "letter-spacing") {
        assert.equal(
          node.value.text,
          "0",
          `${file}: keep letter spacing neutral`,
        );
      }
      if (
        node.type === "css-decl" &&
        ["transition", "transition-property"].includes(node.prop)
      ) {
        assert.doesNotMatch(
          node.value.text,
          /\ball\b/,
          `${file}: specify transition properties`,
        );
      }
      if (node.type === "value-func" && node.value === "var") {
        const [token, fallback] = node.group.groups;
        assert.ok(
          fallback || definedTokens.has(token.value),
          `${file}: undefined token ${token.value}`,
        );
      }
    });
  });
}

test("CSS: style entry imports each shared sheet once and motion policy last", () => {
  const entry = styles.find((style) => style.file === "index.css").ast;
  const imports = entry.nodes.filter(
    (node) => node.type === "css-atrule" && node.name === "import",
  );
  const paths = imports.map((node) => node.params.group.group.value);
  assert.equal(new Set(paths).size, paths.length);
  assert.equal(paths.at(-1), "./animations.css");
  assert.deepEqual(
    [...paths].sort(),
    files
      .filter(
        (file) => !["index.css", "development-preview.css"].includes(file),
      )
      .map((file) => `./${file}`)
      .sort(),
  );
});
