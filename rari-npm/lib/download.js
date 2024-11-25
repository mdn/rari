import { platform, tmpdir } from "os";
import { getProxyForUrl } from "proxy-from-env";
import * as https from "https";
import { createWriteStream } from "fs";
import { unlink, access, constants, mkdir, chmod } from "fs/promises";
import path from "node:path";

import { x } from "tar";
import extract from "extract-zip";

import packageJson from "../package.json" with { type: "json" };

const tmpDir = path.join(tmpdir(), `rari-cache-${packageJson.version}`);
const isWindows = platform() === "win32";

const REPO = "mdn/rari";

/**
 *
 * @param {string | URL} url
 * @param {string | URL} [base]
 */
function URLparse(url, base) {
  if (URL?.parse) {
    return URL.parse(url, base);
  } else {
    try {
      return new URL(url, base);
    } catch {
      return null;
    }
  }
}

/**
 * This function is adapted from vscode-ripgrep (https://github.com/microsoft/vscode-ripgrep)
 * Copyright (c) Microsoft, licensed under the MIT License
 *
 * @param {string} url
 */
function isGithubUrl(url) {
  return URLparse(url)?.hostname === "api.github.com";
}

/**
 * @param {string} path
 */
export async function exists(path) {
  try {
    await access(path, constants.F_OK);
    return true;
  } catch {
    return false;
  }
}

/**
 * This function is adapted from vscode-ripgrep (https://github.com/microsoft/vscode-ripgrep)
 * Copyright (c) Microsoft, licensed under the MIT License
 *
 * @param {string} url
 * @param {import("fs").PathLike} dest
 * @param {any} opts
 */
export async function do_download(url, dest, opts) {
  const proxy = getProxyForUrl(URLparse(url));
  if (proxy !== "") {
    const HttpsProxyAgent = await import("https-proxy-agent");
    opts = {
      ...opts,
      agent: new HttpsProxyAgent.HttpsProxyAgent(proxy),
      proxy,
    };
  }

  if (opts.headers && opts.headers.authorization && !isGithubUrl(url)) {
    delete opts.headers.authorization;
  }

  return await new Promise((resolve, reject) => {
    console.log(`Download options: ${JSON.stringify(opts)}`);
    const outFile = createWriteStream(dest);

    https
      .get(url, opts, (response) => {
        console.log("statusCode: " + response.statusCode);
        if (response.statusCode === 302 && response.headers.location) {
          console.log("Following redirect to: " + response.headers.location);
          return do_download(response.headers.location, dest, opts).then(
            resolve,
            reject,
          );
        } else if (response.statusCode !== 200) {
          reject(new Error("Download failed with " + response.statusCode));
          return;
        }

        response.pipe(outFile);
        outFile.on("finish", () => {
          resolve(null);
        });
      })
      .on("error", async (err) => {
        await unlink(dest);
        reject(err);
      });
  });
}

/**
 * This function is adapted from vscode-ripgrep (https://github.com/microsoft/vscode-ripgrep)
 * Copyright (c) Microsoft, licensed under the MIT License
 *
 * @param {string} _url
 * @param {any} opts
 */
function get(_url, opts) {
  console.log(`GET ${_url}`);

  const proxy = getProxyForUrl(URLparse(_url));
  if (proxy !== "") {
    var HttpsProxyAgent = require("https-proxy-agent");
    opts = {
      ...opts,
      agent: new HttpsProxyAgent.HttpsProxyAgent(proxy),
    };
  }

  return new Promise((resolve, reject) => {
    let result = "";
    https.get(_url, opts, (response) => {
      if (response.statusCode !== 200) {
        reject(new Error(`Request (${_url}) failed: ${response.statusCode}`));
      }

      response.on("data", (d) => {
        result += d.toString();
      });

      response.on("end", () => {
        resolve(result);
      });

      response.on("error", (e) => {
        reject(e);
      });
    });
  });
}

/**
 * @param {string} repo
 * @param {string} tag
 */
function getApiUrl(repo, tag) {
  return `https://api.github.com/repos/${repo}/releases/tags/${tag}`;
}

/**
 * This function is adapted from vscode-ripgrep (https://github.com/microsoft/vscode-ripgrep)
 * Copyright (c) Microsoft, licensed under the MIT License
 *
 * @param {{ force: boolean; token: string | undefined; version: string; }} opts
 * @param {string} assetName
 * @param {string} downloadFolder
 */
async function getAssetFromGithubApi(opts, assetName, downloadFolder) {
  const assetDownloadPath = path.join(downloadFolder, assetName);

  // We can just use the cached binary
  if (!opts.force && (await exists(assetDownloadPath))) {
    console.log("Using cached download: " + assetDownloadPath);
    return assetDownloadPath;
  }

  const downloadOpts = {
    headers: {
      "user-agent": "rari-npm",
    },
  };

  if (opts.token) {
    downloadOpts.headers.authorization = `token ${opts.token}`;
  }

  console.log(`Finding release for ${opts.version}`);
  const release = await get(getApiUrl(REPO, opts.version), downloadOpts);
  let jsonRelease;
  try {
    jsonRelease = JSON.parse(release);
  } catch (e) {
    throw new Error("Malformed API response: " + e.stack);
  }

  if (!jsonRelease.assets) {
    throw new Error("Bad API response: " + JSON.stringify(release));
  }

  const asset = jsonRelease.assets.find((a) => a.name === assetName);
  if (!asset) {
    throw new Error("Asset not found with name: " + assetName);
  }

  console.log(`Downloading from ${asset.url}`);
  console.log(`Downloading to ${assetDownloadPath}`);

  downloadOpts.headers.accept = "application/octet-stream";
  await do_download(asset.url, assetDownloadPath, downloadOpts);
}

/**
 * @param {string} packedFilePath
 * @param {string} destinationDir
 */
async function unpack(packedFilePath, destinationDir) {
  const rari_name = "rari";
  if (isWindows) {
    await extract(packedFilePath, { dir: destinationDir });
  } else {
    await x({ cwd: destinationDir, file: packedFilePath });
  }

  const expectedName = path.join(destinationDir, rari_name);
  if (await exists(expectedName)) {
    return expectedName;
  }

  if (await exists(expectedName + ".exe")) {
    return expectedName + ".exe";
  }

  throw new Error(
    `Expecting ${rari_name} or ${rari_name}.exe unzipped into ${destinationDir}, didn't find one.`,
  );
}

/**
 * This function is adapted from vscode-ripgrep (https://github.com/microsoft/vscode-ripgrep)
 * Copyright (c) Microsoft, licensed under the MIT License
 *
 * @param {{
     version: string;
     token: string | undefined;
     target: string;
     destDir: string;
     force: boolean;
 }} options
*/
export async function download(options) {
  if (!options.version) {
    return Promise.reject(new Error("Missing version"));
  }

  if (!options.target) {
    return Promise.reject(new Error("Missing target"));
  }

  const extension = isWindows ? ".zip" : ".tar.gz";
  const assetName = ["rari", options.target].join("-") + extension;

  if (!(await exists(tmpDir))) {
    await mkdir(tmpDir);
  }

  const assetDownloadPath = path.join(tmpDir, assetName);
  try {
    await getAssetFromGithubApi(options, assetName, tmpDir);
  } catch (e) {
    console.log("Deleting invalid download cache");
    try {
      await unlink(assetDownloadPath);
    } catch (e) {}

    throw e;
  }

  console.log(`Unpacking to ${options.destDir}`);
  try {
    const destinationPath = await unpack(assetDownloadPath, options.destDir);
    if (!isWindows) {
      await chmod(destinationPath, "755");
    }
  } catch (e) {
    console.log("Deleting invalid download");

    try {
      await unlink(assetDownloadPath);
    } catch (e) {}

    throw e;
  }
}
