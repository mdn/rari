# rari on npm

> [!WARNING]
> This is still experimental and work in progress.

This exposes [rari](https://github.com/mdn/rari) in the npm world.

## Local development

### Building types from local Rust changes

From the repo root, build the binary and copy it into the npm package:

```bash
cargo build
cp target/debug/rari rari-npm/bin/rari
```

Then from `rari-npm/`, export the schema and regenerate the types:

```bash
npm run export-schema   # writes schema.json using the local binary
npm run generate-types  # generates lib/rari-types.d.ts from schema.json
```

### Creating a test package

After generating types, create a tarball from `rari-npm/`:

```bash
npm pack
```

This produces `mdn-rari-<version>.tgz`. To install it in another project:

```bash
npm install /path/to/rari/rari-npm/mdn-rari-*.tgz
```

The postinstall script will download the released binary matching the package version from GitHub. To use the locally-built binary instead, skip postinstall and copy the binary manually:

```bash
npm install --ignore-scripts /path/to/rari/rari-npm/mdn-rari-*.tgz
mkdir -p node_modules/@mdn/rari/bin
cp /path/to/rari/target/debug/rari node_modules/@mdn/rari/bin/rari
```

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
The tag must also contain the dispatch-enabled `publish-npm.yml`; older tags
cannot use this workflow. Already-published npm versions cannot be overwritten.
