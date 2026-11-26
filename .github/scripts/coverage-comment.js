// @ts-check
const { readFile } = require("node:fs/promises");
const path = require("node:path");

const MARKER = "<!-- rari-coverage-diff -->";
const MAX_ROWS = 50;

/**
 * @typedef {{ count: number, covered: number }} Counts
 * @typedef {{ total: Counts, files: Map<string, Counts> }} Summary
 */

class InvalidSummaryError extends Error {
  constructor() {
    super("invalid coverage summary");
  }
}

/** @returns {Counts} */
function counts(lines) {
  const { count, covered } = lines ?? {};
  const isCount = (n) => Number.isInteger(n) && n >= 0 && n < 1e9;
  if (!isCount(count) || !isCount(covered) || covered > count) {
    throw new InvalidSummaryError();
  }
  return { count, covered };
}

/**
 * Validates an untrusted `cargo llvm-cov --json --summary-only` report.
 * @param {string} root Workspace path to strip from filenames.
 * @returns {Summary}
 */
function parseSummary(json, root) {
  const data = json?.data?.[0];
  const files = data?.files;
  if (!Array.isArray(files) || files.length > 10000) {
    throw new InvalidSummaryError();
  }
  return {
    total: counts(data.totals?.lines),
    files: new Map(
      files.map(({ filename, summary }) => {
        // PR and baseline runs share the runner workspace path.
        const name =
          typeof filename === "string" && filename.startsWith(root)
            ? filename.slice(root.length)
            : filename;
        if (
          typeof name !== "string" ||
          !/^[A-Za-z0-9_./-]{1,200}$/.test(name)
        ) {
          throw new InvalidSummaryError();
        }
        return [name, counts(summary?.lines)];
      }),
    ),
  };
}

/**
 * Line coverage in integer tenths of a percent, so equality means no change at 0.1 pp.
 * @param {Counts} counts
 */
function tenths({ count, covered }) {
  return count > 0 ? Math.round((covered * 1000) / count) : 0;
}

/** @param {number} n Non-negative tenths. */
function number(n) {
  return `${Math.floor(n / 10)}.${n % 10}`;
}

/** @param {number | undefined} n */
function pct(n) {
  return n === undefined ? "-" : `${number(n)}%`;
}

/** @param {number} n */
function delta(n) {
  return n < 0 ? `-${number(-n)} pp` : `+${number(n)} pp`;
}

/**
 * @param {{ base: Summary, pr: Summary, baseSha: string, reportUrl?: string }} options
 * @returns {string | undefined} The comment body, or `undefined` if coverage is unchanged.
 */
function render({ base, pr, baseSha, reportUrl }) {
  const baseTotal = tenths(base.total);
  const prTotal = tenths(pr.total);
  const baseFiles = new Map([...base.files].map(([k, v]) => [k, tenths(v)]));
  const prFiles = new Map([...pr.files].map(([k, v]) => [k, tenths(v)]));

  const rows = [...new Set([...baseFiles.keys(), ...prFiles.keys()])]
    .sort()
    .map((name) => ({ name, base: baseFiles.get(name), pr: prFiles.get(name) }))
    .filter((row) => row.base !== row.pr);
  // New and removed files first, then the largest changes (stable, so ties stay sorted by name).
  const rank = ({ base, pr }) =>
    base === undefined || pr === undefined ? -Infinity : -Math.abs(pr - base);
  rows.sort((a, b) => rank(a) - rank(b));

  if (baseTotal === prTotal && rows.length === 0) {
    return undefined;
  }

  const total =
    baseTotal === prTotal
      ? `${pct(prTotal)} of lines (unchanged)`
      : `${pct(baseTotal)} → ${pct(prTotal)} of lines (${delta(prTotal - baseTotal)})`;
  const lines = [
    MARKER,
    `**Test coverage:** ${total}, compared to \`main\` at ${baseSha.slice(0, 7)}`,
  ];

  if (rows.length > 0) {
    lines.push(
      "",
      `<details><summary>${rows.length} ${rows.length === 1 ? "file" : "files"} changed</summary>`,
      "",
      "| File | `main` | PR | Change |",
      "| --- | --: | --: | --: |",
      ...rows.slice(0, MAX_ROWS).map(({ name, base, pr }) => {
        const change =
          base === undefined
            ? "new"
            : pr === undefined
              ? "removed"
              : delta(pr - base);
        return `| \`${name}\` | ${pct(base)} | ${pct(pr)} | ${change} |`;
      }),
    );
    if (rows.length > MAX_ROWS) {
      lines.push("", `And ${rows.length - MAX_ROWS} more.`);
    }
    lines.push("", "</details>");
  }

  if (reportUrl) {
    lines.push("", `[Download report](${reportUrl}) (\`index.html\`)`);
  }

  return lines.join("\n");
}

/** @param {string} file */
async function readSummary(file, root) {
  let json;
  try {
    json = JSON.parse(await readFile(file, "utf8"));
  } catch {
    throw new InvalidSummaryError();
  }
  return parseSummary(json, root);
}

/**
 * Entry point for `actions/github-script`.
 * @param {{ github: any, context: any, core: any }} args
 */
async function run({ github, context, core }) {
  const { owner, repo } = context.repo;
  const { id: runId, head_sha: headSha } = context.payload.workflow_run;
  const { PR, BASE_SHA, SUMMARY_DIR, GITHUB_WORKSPACE } = process.env;

  if (!/^[0-9]+$/.test(PR ?? "")) {
    throw new Error(`Invalid PR number: ${PR}`);
  }
  const issueNumber = Number(PR);

  let body;
  if (BASE_SHA) {
    if (!/^[0-9a-f]{40}$/.test(BASE_SHA)) {
      throw new Error(`Invalid base SHA: ${BASE_SHA}`);
    }
    const root = `${GITHUB_WORKSPACE}/`;
    const { data: artifacts } =
      await github.rest.actions.listWorkflowRunArtifacts({
        owner,
        repo,
        run_id: runId,
        name: "coverage-report",
      });
    const report = artifacts.artifacts.find((a) => !a.expired);
    const summary = (dir) =>
      readSummary(path.join(SUMMARY_DIR, dir, "coverage-summary.json"), root);
    body = render({
      base: await summary("base"),
      pr: await summary("pr"),
      baseSha: BASE_SHA,
      reportUrl:
        report &&
        `${context.serverUrl}/${owner}/${repo}/actions/runs/${runId}/artifacts/${report.id}`,
    });
  }

  const comments = await github.paginate(github.rest.issues.listComments, {
    owner,
    repo,
    issue_number: issueNumber,
    per_page: 100,
  });
  const comment = comments.find(
    (c) => c.user?.login === "github-actions[bot]" && c.body?.includes(MARKER),
  );

  const { data: pull } = await github.rest.pulls.get({
    owner,
    repo,
    pull_number: issueNumber,
  });
  if (pull.state !== "open" || pull.head.sha !== headSha) {
    core.notice(`PR #${issueNumber} is closed or has a newer head; nothing to do.`);
    return;
  }

  if (!body) {
    if (!comment) {
      return;
    }
    // Without a summary, the comment would keep the numbers of an older head.
    const status = BASE_SHA ? "No change." : `Not available for ${headSha.slice(0, 7)}.`;
    body = `${MARKER}\n**Test coverage:** ${status}`;
  }

  if (comment) {
    await github.rest.issues.updateComment({ owner, repo, comment_id: comment.id, body });
  } else {
    await github.rest.issues.createComment({ owner, repo, issue_number: issueNumber, body });
  }
}

module.exports = { run, render, parseSummary };
