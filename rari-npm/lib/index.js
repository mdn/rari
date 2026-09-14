import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

const PLATFORM_PACKAGE = `@mdn/rari-${process.platform}-${process.arch}`;

function resolveRariBin() {
  if (process.env.RARI_BINARY_PATH) {
    return process.env.RARI_BINARY_PATH;
  }

  const binary = process.platform === "win32" ? "rari.exe" : "rari";
  try {
    return require.resolve(`${PLATFORM_PACKAGE}/bin/${binary}`);
  } catch (error) {
    throw new Error(
      `Could not find the rari binary for ${process.platform}-${process.arch}. ` +
        `Make sure ${PLATFORM_PACKAGE} is installed: it is an optional dependency ` +
        `of @mdn/rari, so it is missing when installing with --omit=optional, ` +
        `when the platform is unsupported, or when the lockfile was generated on ` +
        `another platform. Alternatively, set RARI_BINARY_PATH to a rari binary.`,
      { cause: error },
    );
  }
}

export const rariBin = resolveRariBin();
