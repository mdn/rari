import { mkdir } from "fs/promises";
import { arch, platform } from "os";
import { join } from "path";

import { download, exists } from "./download.js";
import packageJson from "../package.json" with { type: "json" };

const VERSION = `v${packageJson.version}`;
const BIN_PATH = join(import.meta.dirname, "../binary");
const FORCE = JSON.parse(process.env.FORCE || "false");
const GITHUB_TOKEN = process.env.GITHUB_TOKEN;

const TARGET_LOOKUP = {
  arm64: {
    darwin: "aarch64-apple-darwin",
    linux: "aarch64-unknown-linux-musl",
    win32: "aarch64-pc-windows-msvc",
  },
  x64: {
    darwin: "x86_64-apple-darwin",
    linux: "x86_64-unknown-linux-musl",
    win32: "x86_64-pc-windows-msvc",
  },
};

async function getTarget() {
  const architecture = process.env.npm_config_arch || arch();

  const target = TARGET_LOOKUP[architecture]?.[platform()];
  if (!target) {
    throw new Error("Unknown platform: " + platform());
  }
  return target;
}

/**
 * This function is adapted from vscode-ripgrep (https://github.com/microsoft/vscode-ripgrep)
 * Copyright (c) Microsoft, licensed under the MIT License
 *
 */
async function main() {
  const binPathExists = await exists(BIN_PATH);
  if (!FORCE && binPathExists) {
    console.log(
      `${BIN_PATH} already exists, exiting use FORCE=true to force install`,
    );
    process.exit(0);
  }

  if (!binPathExists) {
    await mkdir(BIN_PATH);
  }

  const target = await getTarget();
  const options = {
    version: VERSION,
    token: GITHUB_TOKEN,
    target,
    destDir: BIN_PATH,
    force: FORCE,
  };
  try {
    await download(options);
  } catch (err) {
    console.error(`Downloading rari failed: ${err.stack}`);
    process.exit(1);
  }
}

await main();
