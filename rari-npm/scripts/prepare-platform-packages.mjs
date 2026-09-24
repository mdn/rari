// Writes the manifests for the per-platform packages into npm/<suffix>/.
// The binaries themselves are added by the publish workflow.
import { copyFile, mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";

import packageJson from "../package.json" with { type: "json" };
import { TARGETS } from "./targets.mjs";

const root = join(import.meta.dirname, "..");
const outDir = join(root, "npm");

const DESCRIPTIONS = {
  darwin: "macOS",
  linux: "Linux",
  win32: "Windows",
};

for (const [suffix, target] of Object.entries(TARGETS)) {
  const [os, cpu] = suffix.split("-");
  const name = `@mdn/rari-${suffix}`;
  const dir = join(outDir, suffix);
  await mkdir(join(dir, "bin"), { recursive: true });

  const manifest = {
    name,
    version: packageJson.version,
    description: `The ${DESCRIPTIONS[os]} ${cpu} binary for rari (${target}).`,
    repository: packageJson.repository,
    homepage: packageJson.homepage,
    author: packageJson.author,
    license: packageJson.license,
    os: [os],
    cpu: [cpu],
  };
  await writeFile(
    join(dir, "package.json"),
    JSON.stringify(manifest, null, 2) + "\n",
  );

  await writeFile(
    join(dir, "README.md"),
    `# ${name}\n\nThe ${DESCRIPTIONS[os]} ${cpu} binary for [rari](https://github.com/mdn/rari).\n\nInstall [\`@mdn/rari\`](https://www.npmjs.com/package/@mdn/rari) instead of depending on this package directly.\n`,
  );

  for (const license of ["LICENSE", "LICENSE.MIT", "LICENSE.MPL-2.0"]) {
    await copyFile(join(root, license), join(dir, license));
  }

  console.log(`Prepared ${dir}`);
}
