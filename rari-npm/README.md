# rari on npm

> [!WARNING]
> This is still experimental and work in progress.

This exposes [rari](https://github.com/mdn/rari) in the npm world.

## Package layout

`@mdn/rari` contains only the JavaScript wrapper and the TypeScript types. The binary ships in one package per platform (e.g. `@mdn/rari-darwin-arm64`, `@mdn/rari-linux-x64`), listed as `optionalDependencies` and restricted via `os`/`cpu`, so npm installs only the one matching the current platform. The wrapper resolves the binary from that package at runtime.

Set `RARI_BINARY_PATH` to use a different binary, e.g. a local debug build.

## Local development

### Building types from local Rust changes

From the repo root, build the binary, then from `rari-npm/` export the schema and regenerate the types using it:

```bash
cargo build
cd rari-npm
npm ci --prefix tooling
RARI_BINARY_PATH=../target/debug/rari npm run export-schema  # writes schema.json
npm run generate-types  # generates lib/rari-types.d.ts from schema.json
```

### Creating test packages

After generating types, create the platform package manifests and a tarball from `rari-npm/`:

```bash
npm run prepare-platform-packages  # writes npm/<platform>-<arch>/
cp ../target/debug/rari npm/darwin-arm64/bin/rari  # pick your platform
(cd npm/darwin-arm64 && npm pack)
npm pack
```

To install them in another project:

```bash
npm install /path/to/rari/rari-npm/mdn-rari-*.tgz /path/to/rari/rari-npm/npm/darwin-arm64/mdn-rari-darwin-arm64-*.tgz
```

Alternatively, install only the main tarball and point `RARI_BINARY_PATH` at a local binary.

Type-generation dependencies live in the private `tooling/` package with its own
lockfile. Publishing installs only these dependencies; platform packages do not
need to exist on npm yet.

## Publishing

After all release binaries have been uploaded, the build workflow dispatches
`publish-npm.yml` on the release tag. Running on the tag ensures npm provenance
identifies the commit used to create the package.

To manually retry publishing, first run a dry run on the release tag:

```bash
gh workflow run publish-npm.yml --repo mdn/rari --ref v0.2.35 -f publish=false
```

Use `-f publish=true` to publish. The selected ref must be a tag matching the
version in `rari-npm/package.json`, and the release binaries must already exist.
The tag must also contain the dispatch-enabled `publish-npm.yml`; use recovery
below if it does not. Already-published npm versions cannot be overwritten.

### Recovering a failed release

If the publishing workflow needs a fix, create a branch from the release tag,
apply the workflow fix, commit it, and push the branch. Keep `rari-npm/`
unchanged, including its package version: recovery only covers fixes outside
`rari-npm/`, so a broken package script or file requires a new release instead.
For an older release, the branch must include the recovery-enabled
`publish-npm.yml`.

```bash
git switch -c recover-v0.2.35 v0.2.35
# Apply and commit the workflow fix.
git push origin recover-v0.2.35
gh workflow run publish-npm.yml --repo mdn/rari --ref recover-v0.2.35 \
  -f recovery_tag=v0.2.35 -f publish=false
```

After inspecting the dry run, repeat the dispatch with `-f publish=true`.
Recovery verifies that the release tag matches the package version, that the
branch descends from the tag, and that `rari-npm/` matches the tag exactly.
The release binaries must already exist. Provenance remains enabled and
identifies the recovery commit, including the workflow fix.

### Staging from a branch

Use staged publishing to test packaging and Trusted Publishing before cutting a
release. The selected branch must contain this workflow and the package changes
to test. `stage_tag` selects an existing release whose binaries will be reused;
it cannot be combined with `recovery_tag`.

```bash
gh workflow run publish-npm.yml --repo mdn/rari --ref 924-stage-npm-packages \
  -f stage_tag=v0.2.34 -f publish=false
```

After inspecting the dry run, repeat with `-f publish=true` to upload all seven
packages using `npm stage publish` with provenance. They remain unavailable to
normal installs until approved. This exercises npm authentication, which a dry
run does not test.

The workflow assigns `<version>-stage.<run-id>.<attempt>` to the wrapper, its
platform dependency pins, and the generated platform packages. Each attempt gets
a unique version, avoiding collisions with releases and pending stages. The
staged packages use the `staging` dist-tag if approved. Their binaries report the
original release version; provenance identifies the dispatched branch commit.

With an authenticated npm session, use the stage IDs from the workflow output to
download the wrapper and your platform package, then install both tarballs in a
temporary project and run the CLI:

```bash
npm stage download <wrapper-stage-id>
npm stage download <platform-stage-id>
npm install --ignore-scripts /path/to/wrapper.tgz /path/to/platform.tgz
npx --no-install rari --version
node --input-type=module -e 'import { rariBin } from "@mdn/rari"; console.log(rariBin)'
```

Installing both tarballs tests binary resolution without install scripts. It
does not test registry resolution of optional dependencies, because staged
packages are not publicly installable. Reject test stages when finished:

```bash
npm stage reject <stage-id>
```

Downloading and rejecting stages require your npm session, not the workflow's
OIDC credentials. Rejection requires 2FA. See the
[npm stage documentation](https://docs.npmjs.com/cli/commands/npm-stage/).
