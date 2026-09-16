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
