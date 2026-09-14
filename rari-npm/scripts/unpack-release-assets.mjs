// Unpacks the release archives (rari-<target>.tar.gz / .zip) from the given
// directory into npm/<suffix>/bin/, matching what prepare-platform-packages.mjs
// generated.
import { execFileSync } from "node:child_process";
import { access, chmod } from "node:fs/promises";
import { join } from "node:path";

import { TARGETS } from "./targets.mjs";

const assetsDir = process.argv[2];
if (!assetsDir) {
  console.error("Usage: unpack-release-assets.mjs <assets-dir>");
  process.exit(1);
}

const npmDir = join(import.meta.dirname, "..", "npm");

for (const [suffix, target] of Object.entries(TARGETS)) {
  const isWindows = suffix.startsWith("win32-");
  const archive = join(
    assetsDir,
    `rari-${target}.${isWindows ? "zip" : "tar.gz"}`,
  );
  const binDir = join(npmDir, suffix, "bin");
  const binary = join(binDir, isWindows ? "rari.exe" : "rari");

  if (isWindows) {
    execFileSync("unzip", ["-o", "-q", archive, "-d", binDir]);
  } else {
    execFileSync("tar", ["-xzf", archive, "-C", binDir]);
    await chmod(binary, 0o755);
  }

  await access(binary);
  console.log(`Unpacked ${archive} to ${binary}`);
}
