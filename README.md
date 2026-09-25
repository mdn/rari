# Welcome to `rari`

`rari` is the build system for [MDN](https://developer.mozilla.org).

`rari` is hosted by [MDN](https://github.com/mdn).

## Getting Started

To get up and running, follow these steps:

Make sure you have [Rust](https://www.rust-lang.org/) installed, otherwise go to [https://rustup.rs/](https://rustup.rs/).

Clone this repository and run:

```plain
cargo run -- --help
```

### Configuration

Create a `.config.toml` in the current working directory.
Add the following:

```toml
content_root = "/<ABSOLUTE-PATH-TO-mdn/content>/files"
build_out_root = "/tmp/rari"
```

To read a locale from a separate translated-content checkout, map it to that
checkout's `files` directory and GitHub repository name:

```toml
content_translated_root = "/<ABSOLUTE-PATH-TO-mdn/translated-content>/files"

[translated_content_sources.de]
root = "/<ABSOLUTE-PATH-TO-mdn/translated-content-de>/files"
repository = "translated-content-de"
```

The mapped root takes precedence for that locale. Other translated locales use
`content_translated_root`. A required mapped checkout must contain its locale
directory.

To include a recognized locale in default translated-content syncs and allow
its checkout to be absent, add it to `optional_translated_locales`. For the
German mapping above:

```toml
content_translated_root = "/<ABSOLUTE-PATH-TO-mdn/translated-content>/files"
optional_translated_locales = ["de"]

[translated_content_sources.de]
root = "/<ABSOLUTE-PATH-TO-mdn/translated-content-de>/files"
repository = "translated-content-de"
```

The setting defaults to an empty list. `OPTIONAL_TRANSLATED_LOCALES` accepts a
comma-separated list as an environment override. An absent optional locale is
skipped by `sync-translated-content` and `fix-flaws`; a present locale with
invalid content still fails. Existing translated locales remain selected by
default, and `additional_locales_for_generics_and_spas` retains its current
behavior.

## Contributing

For now we're aiming for a parity rewrite of [yari's](https://github.com/mdn/yari) `yarn build -n`. Which generates the `index.json`
for all docs. Until we reach that point the codebase will be unstable and may change at any point. Therefore we won't accept contributions for now.

<!--
Our project welcomes contributions from any member of our community.
To get started contributing, please see our [Contributor Guide](CONTRIBUTING.md).

-->

By participating in and contributing to our projects and discussions, you acknowledge that you have read and agree to our [Code of Conduct](CODE_OF_CONDUCT.md).

## Resources

For more information about `rari`, see the following resources:

To be updated...

<!-- [TODO: Add links to other helpful information (roadmap, docs, website, etc.)] -->

## Communications

If you have any questions, please reach out to us on [Discord](https://developer.mozilla.org/discord)

## License

This project is licensed under the [Mozilla Public License 2.0](LICENSE.md).
