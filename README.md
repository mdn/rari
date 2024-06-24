> [!Warning]
> This project is work in project and lacking most of its documentation.
> Anything might change and code will move a lot. We do not encourage using it yet.
> We'll have an official announcement before we migrate, so stay tuned.


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

### Configuation

Create a `.config.toml` in the current working directory.
Add the following:

```toml
content_root = "/<ABSOLUTE-PATH-TO-mdn/content>/files"
build_out_root = "/tmp/rari"
```

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
