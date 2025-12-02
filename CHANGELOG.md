# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.5](https://github.com/mdn/rari/compare/v0.2.4...v0.2.5) (2025-12-02)


### Miscellaneous

* **deps:** bump percent-encoding from 2.3.1 to 2.3.2 ([#404](https://github.com/mdn/rari/issues/404)) ([86610ce](https://github.com/mdn/rari/commit/86610ce622ed60f05ed2aadab5bfb28efb766401))
* **deps:** bump quote from 1.0.40 to 1.0.42 ([#402](https://github.com/mdn/rari/issues/402)) ([0fc0d30](https://github.com/mdn/rari/commit/0fc0d30868eb4a5a80c56a91662219b090a9093b))
* **deps:** bump rayon from 1.10.0 to 1.11.0 ([#403](https://github.com/mdn/rari/issues/403)) ([f1b26e5](https://github.com/mdn/rari/commit/f1b26e55b7843b56f5590193e1644c951548c413))
* **deps:** bump tower-http from 0.6.6 to 0.6.7 ([#406](https://github.com/mdn/rari/issues/406)) ([9304c98](https://github.com/mdn/rari/commit/9304c98844f7fa6350dbb03e03242265e3121031))

## [0.2.4](https://github.com/mdn/rari/compare/v0.2.3...v0.2.4) (2025-11-28)


### Bug Fixes

* **doc:** add tracing context to parallel read ([#399](https://github.com/mdn/rari/issues/399)) ([c9c4687](https://github.com/mdn/rari/commit/c9c46871bebea181cbfd0f9d901f0211e1f08d32))


### Miscellaneous

* **types:** remove legacy_live_samples_base_url from settings ([#400](https://github.com/mdn/rari/issues/400)) ([d93013f](https://github.com/mdn/rari/commit/d93013f65a9c2f58e97147516099a789a1e77abe))

## [0.2.3](https://github.com/mdn/rari/compare/v0.2.2...v0.2.3) (2025-11-27)


### Bug Fixes

* **sidebar:** skip missing pages ([#397](https://github.com/mdn/rari/issues/397)) ([acddbfc](https://github.com/mdn/rari/commit/acddbfc0250278293d016f1df315d94e4224b6c4))

## [0.2.2](https://github.com/mdn/rari/compare/v0.2.1...v0.2.2) (2025-11-20)


### Features

* **cli:** add --force-updates option ([#381](https://github.com/mdn/rari/issues/381)) ([4bbcf79](https://github.com/mdn/rari/commit/4bbcf79e06d2ef5def4dc55fe51c37c45d12bd95))
* **cssxref:** migrate to new CSS url structure ([#382](https://github.com/mdn/rari/issues/382)) ([0ec5233](https://github.com/mdn/rari/commit/0ec52334a58ad78695bcea31cfa5574703c7b00b))


### Bug Fixes

* **css-syntax:** use extended spec links if available ([#375](https://github.com/mdn/rari/issues/375)) ([df5f813](https://github.com/mdn/rari/commit/df5f8135f795160c8229e8f13fdb3cf0d91d59fd))
* **cssinfo:** follow the css reorg ([#385](https://github.com/mdn/rari/issues/385)) ([4b3f190](https://github.com/mdn/rari/commit/4b3f19057b6edf7d7ff51b6c71cd8441ed960e5d))
* **doc:** add tracing context to `build_doc()` ([#371](https://github.com/mdn/rari/issues/371)) ([ce04c82](https://github.com/mdn/rari/commit/ce04c823c8a2c56f3c35eda050fe79d349224cd5))
* **fix-flaws:** apply suggestions in order ([#378](https://github.com/mdn/rari/issues/378)) ([6ac3c03](https://github.com/mdn/rari/commit/6ac3c035df42777cb7478409a4742e29e5c2567a))
* **tools:** skip missing redirect files ([#379](https://github.com/mdn/rari/issues/379)) ([744c998](https://github.com/mdn/rari/commit/744c99826f114cc34daf59d9d85516efc9015bb0))

## [0.2.1](https://github.com/mdn/rari/compare/v0.2.0...v0.2.1) (2025-11-06)


### Bug Fixes

* **css-syntax:** distinguish formal syntax scopes ([#359](https://github.com/mdn/rari/issues/359)) ([7a96e37](https://github.com/mdn/rari/commit/7a96e37c8b7209df65e14fc55b931ad551a040e5))
* **tools/redirects:** remove entries where target page exists ([#352](https://github.com/mdn/rari/issues/352)) ([29efb99](https://github.com/mdn/rari/commit/29efb996fd19ba7e686d3028a45a358567a749f8))


### Miscellaneous

* **deps:** bump rust from 1.86 to 1.90 ([#360](https://github.com/mdn/rari/issues/360)) ([fd36371](https://github.com/mdn/rari/commit/fd363711f2b00d16efe18cf50f1a1c81b8665e2e))
* **rustfmt:** disable unstable features ([#363](https://github.com/mdn/rari/issues/363)) ([247311f](https://github.com/mdn/rari/commit/247311fcf4ce5b6a3a8537b4652f97a2b0aec86c))

## [0.2.0](https://github.com/mdn/rari/compare/v0.1.54...v0.2.0) (2025-10-31)


### ⚠ BREAKING CHANGES

* **notecard:** remove the support of the legacy note card format ([#350](https://github.com/mdn/rari/issues/350))

### Miscellaneous

* **css-syntax:** update to webref 8 ([#357](https://github.com/mdn/rari/issues/357)) ([31bfae8](https://github.com/mdn/rari/commit/31bfae867944c800ddff637cc8ec7a6d8456a0ba))
* **notecard:** remove the support of the legacy note card format ([#350](https://github.com/mdn/rari/issues/350)) ([482a324](https://github.com/mdn/rari/commit/482a32423fe904a02e0626018a6e72b5c91f8ff6))
* **npm:** migrate to Trusted Publishing ([#356](https://github.com/mdn/rari/issues/356)) ([c776eff](https://github.com/mdn/rari/commit/c776efff67117ecc42af88d81608ad8ae1920dfb))

## [0.1.54](https://github.com/mdn/rari/compare/v0.1.53...v0.1.54) (2025-10-27)


### Features

* **rari-cli:** print port at server startup ([#319](https://github.com/mdn/rari/issues/319)) ([565a054](https://github.com/mdn/rari/commit/565a054c6fa2ee06c76808029727f7cd70917918))


### Bug Fixes

* **build:** read popularities from data dir ([#337](https://github.com/mdn/rari/issues/337)) ([1c8568b](https://github.com/mdn/rari/commit/1c8568b78c5d606f545f996a064ecccd20eefe27))
* **md:** remove the leading space of the first paragraph in note card ([25036cb](https://github.com/mdn/rari/commit/25036cbfdb4e36bd9c8d7c01005bcd683f32927a))
* **notecard:** remove leading space of first paragraph in Chinese ([#323](https://github.com/mdn/rari/issues/323)) ([25036cb](https://github.com/mdn/rari/commit/25036cbfdb4e36bd9c8d7c01005bcd683f32927a))
* **sync-translated-content:** move adjacent assets correctly ([#335](https://github.com/mdn/rari/issues/335)) ([bad50ab](https://github.com/mdn/rari/commit/bad50ab6e5a7f0760f64c60dbcc4a4ed88a1bd51))


### Miscellaneous

* **css-syntax:** update ill cased links ([#331](https://github.com/mdn/rari/issues/331)) ([2b8bd2b](https://github.com/mdn/rari/commit/2b8bd2bce2c6361f387024231185ab6c9c7d9443))
* migrate GitHub team references ([#351](https://github.com/mdn/rari/issues/351)) ([da911df](https://github.com/mdn/rari/commit/da911dfb0ffbb6dd3a52b23f3dd542aa09aaf573))

## [0.1.53](https://github.com/mdn/rari/compare/v0.1.52...v0.1.53) (2025-10-15)


### Features

* **css-syntax:** migrate to @webref/css 7 ([#317](https://github.com/mdn/rari/issues/317)) ([b011b7d](https://github.com/mdn/rari/commit/b011b7dadd18251f05047c31a312edbc3bf88a8d))


### Bug Fixes

* **CODEOWNERS:** add Engineering for dependencies ([#328](https://github.com/mdn/rari/issues/328)) ([a5b5e04](https://github.com/mdn/rari/commit/a5b5e0460a37225e74a2408213eeed6496fa4aba))


### Miscellaneous

* **tools:** remove terminal colorization ([#333](https://github.com/mdn/rari/issues/333)) ([1a800dd](https://github.com/mdn/rari/commit/1a800dd96fd38064d23f7cc03d619f1e37b07999))
* update authors ([#26](https://github.com/mdn/rari/issues/26)) ([58279fc](https://github.com/mdn/rari/commit/58279fc9966b001541cddbb69a9fb46fc86a6507))

## [0.1.52](https://github.com/mdn/rari/compare/v0.1.51...v0.1.52) (2025-10-13)


### Bug Fixes

* **csssyntax:** update renamed CSS slug ([#325](https://github.com/mdn/rari/issues/325)) ([13ae24c](https://github.com/mdn/rari/commit/13ae24cdc345329377bfc9695f07de770d3851fa))
* **sync-translated-content:** handle renames to sync case changes from content ([#327](https://github.com/mdn/rari/issues/327)) ([f64385d](https://github.com/mdn/rari/commit/f64385dee9a90a26290ec2dea6f483d363dd92b4))


### Miscellaneous

* add rustfmt and clippy to linter action toolchain ([#321](https://github.com/mdn/rari/issues/321)) ([3e70d19](https://github.com/mdn/rari/commit/3e70d198d431a7c5c4fd4499c5ff2dd30ad0773c))

## [0.1.51](https://github.com/mdn/rari/compare/v0.1.50...v0.1.51) (2025-09-18)


### Miscellaneous

* **web-features:** update schema to match v3 upstream ([#312](https://github.com/mdn/rari/issues/312)) ([9cc6510](https://github.com/mdn/rari/commit/9cc65104edbd844e714fd96b1d3a5f471e0322e8))

## [0.1.50](https://github.com/mdn/rari/compare/v0.1.49...v0.1.50) (2025-09-17)


### Bug Fixes

* **npm:** generate types matching serialized objects ([#314](https://github.com/mdn/rari/issues/314)) ([df570cc](https://github.com/mdn/rari/commit/df570cc1501af1920530a7ef7f6e2190e8f6dd0a))

## [0.1.49](https://github.com/mdn/rari/compare/v0.1.48...v0.1.49) (2025-09-09)


### Bug Fixes

* **contributors:** add short_title to spotlight pages ([#297](https://github.com/mdn/rari/issues/297)) ([264c670](https://github.com/mdn/rari/commit/264c67012e1694cb2e10fae4a5309d16c7b24d02))
* **rari-npm:** handle exit with signal ([#308](https://github.com/mdn/rari/issues/308)) ([e452c96](https://github.com/mdn/rari/commit/e452c9655587da6737e5d8f88d3b995b8c2f754c))
* **sidebar:** guides links were wrapped in code element ([#293](https://github.com/mdn/rari/issues/293)) ([6935c7b](https://github.com/mdn/rari/commit/6935c7b766dfd50fedaf07d78312fc9db9ef26b6))
* **webref_css:** add @webref/css version requirement ([30bd2cc](https://github.com/mdn/rari/commit/30bd2cc8dc4b34dcc1b49866356ff5e090cb1e86))
* **webref_css:** pin @webref/css to v6 ([#305](https://github.com/mdn/rari/issues/305)) ([30bd2cc](https://github.com/mdn/rari/commit/30bd2cc8dc4b34dcc1b49866356ff5e090cb1e86))


### Miscellaneous

* **api:** apply locale passed to RariApi::link + make optional ([#287](https://github.com/mdn/rari/issues/287)) ([6c24ea1](https://github.com/mdn/rari/commit/6c24ea12cec78447b3704e3f7612ab9b9269b128))
* **deps:** bump slab from 0.4.10 to 0.4.11 ([#295](https://github.com/mdn/rari/issues/295)) ([eba3173](https://github.com/mdn/rari/commit/eba31735854093ebed17d2e79d0f45f2142a68c9))
* **deps:** bump tracing-subscriber from 0.3.19 to 0.3.20 ([#309](https://github.com/mdn/rari/issues/309)) ([952b3ae](https://github.com/mdn/rari/commit/952b3ae709ab539fb95d90048182d4792ab60c97))
* **github:** add CODEOWNERS ([#299](https://github.com/mdn/rari/issues/299)) ([f80229c](https://github.com/mdn/rari/commit/f80229c84e5f434427c11f7a58cd86a837ab07b0))
* **rari-deps:** handle GitHub API error ([#301](https://github.com/mdn/rari/issues/301)) ([0fb32df](https://github.com/mdn/rari/commit/0fb32df0ea2f72ce3807ce702214cedfffcde9ba))

## [0.1.48](https://github.com/mdn/rari/compare/v0.1.47...v0.1.48) (2025-08-11)


### Miscellaneous

* **clippy:** update rust toolchain, added elided lifetime specifiers ([#283](https://github.com/mdn/rari/issues/283)) ([6b3dbdd](https://github.com/mdn/rari/commit/6b3dbddceb0e2e98824ac36440342262bcc877c6))

## [0.1.47](https://github.com/mdn/rari/compare/v0.1.46...v0.1.47) (2025-08-07)


### Bug Fixes

* **404:** build for every locale ([#281](https://github.com/mdn/rari/issues/281)) ([2a99d06](https://github.com/mdn/rari/commit/2a99d0650506b106f72e63964fefc05c960da42a))
* **translations:** include current locale when doing build ([#280](https://github.com/mdn/rari/issues/280)) ([4be95c8](https://github.com/mdn/rari/commit/4be95c81af810a835716f9b3729794de885ec479))

## [0.1.46](https://github.com/mdn/rari/compare/v0.1.45...v0.1.46) (2025-07-10)


### Bug Fixes

* **blog:** add breadcrumbs to posts ([#271](https://github.com/mdn/rari/issues/271)) ([20d040c](https://github.com/mdn/rari/commit/20d040c88e777dec3af59a9f00559244efd79832))
* **blog:** filter unpublished posts immediately ([#264](https://github.com/mdn/rari/issues/264)) ([45b5bf5](https://github.com/mdn/rari/commit/45b5bf5b3ca3852ba47184ff313bf38be739552a))
* **blog:** return only single parent/breadcrumb item for index ([01caa83](https://github.com/mdn/rari/commit/01caa83d4f4c4a3b48569b2f322b125d6c04202c))


### Enhancements

* **page:** add other_translations to non-Doc pages ([#272](https://github.com/mdn/rari/issues/272)) ([9ae3a11](https://github.com/mdn/rari/commit/9ae3a11d6e2a1f73ef274bbf5ab0b60f242971d2))
* **pages:** support short-title for generic ([#274](https://github.com/mdn/rari/issues/274)) ([9627c9b](https://github.com/mdn/rari/commit/9627c9b6acb13b4666f7cf2ec4936320ff6100c3))
* **sidebars:** add `<wbr>` to long code elements ([#270](https://github.com/mdn/rari/issues/270)) ([8c5d8fc](https://github.com/mdn/rari/commit/8c5d8fc41876f6538f658a48b99ba7513ebe1d00))
* **spa:** refine {page,short}_title ([#269](https://github.com/mdn/rari/issues/269)) ([5a4b046](https://github.com/mdn/rari/commit/5a4b046297f1d6461dded4ebf1afb578c8eae907))


### Miscellaneous

* **clippy:** fix complaints about format strings ([b70a321](https://github.com/mdn/rari/commit/b70a32149099349cb9d901109671b3d3130c2b28))

## [0.1.45](https://github.com/mdn/rari/compare/v0.1.44...v0.1.45) (2025-06-25)


### Features

* **frontmatter:** support banners ([#260](https://github.com/mdn/rari/issues/260)) ([2a25e22](https://github.com/mdn/rari/commit/2a25e22c69dd23a1abb82169fd60f037da7bf414))
* **json:** update schemars and inline renderer ([#258](https://github.com/mdn/rari/issues/258)) ([0d2539c](https://github.com/mdn/rari/commit/0d2539c4aea9beb876a72c53a2d91b0daa2bbc59))
* **lsp:** support other locales in links ([0381de8](https://github.com/mdn/rari/commit/0381de814e6dfd01129d0ade9a1d61a4c44a64ff))
* **updates:** allow skip updates from env ([#256](https://github.com/mdn/rari/issues/256)) ([c5b03eb](https://github.com/mdn/rari/commit/c5b03eb1277dd49b7c24d757fc8fdd209e847245))


### Miscellaneous

* **deps:** bump brace-expansion from 2.0.1 to 2.0.2 in /rari-npm ([#259](https://github.com/mdn/rari/issues/259)) ([2cfb136](https://github.com/mdn/rari/commit/2cfb13653ca99f4ed489ec0c7a4eae2d8c65afeb))
* **deps:** update major deps ([47b9a5e](https://github.com/mdn/rari/commit/47b9a5e0745014e74855fb005ab60283bdd504f3))

## [0.1.44](https://github.com/mdn/rari/compare/v0.1.43...v0.1.44) (2025-06-20)


### Bug Fixes

* revert "chore(deps): update major deps" ([e69710e](https://github.com/mdn/rari/commit/e69710e3ad6223241d032572664cf73c35dc3aef))

## [0.1.43](https://github.com/mdn/rari/compare/v0.1.42...v0.1.43) (2025-06-19)


### Features

* **spas:** support short_title ([d5c48db](https://github.com/mdn/rari/commit/d5c48db5d8cc9c9ddd55723d97ce0f99598c450d))


### Bug Fixes

* **serve:** avoid fallback for json ([#251](https://github.com/mdn/rari/issues/251)) ([759cefe](https://github.com/mdn/rari/commit/759cefea69f1dbedb0fa7f022cfa34f74d4abd68))
* **serve:** return 404 for UrlError::InvalidUrl ([#249](https://github.com/mdn/rari/issues/249)) ([18f54ee](https://github.com/mdn/rari/commit/18f54eec42f316809148324604e47d942c7cdf10))
* short title for Observatory report page ([#252](https://github.com/mdn/rari/issues/252)) ([b679d74](https://github.com/mdn/rari/commit/b679d74339ab44442e8ad80faa7065da52956db6))
* **sidebar:** avoid empty span ([#253](https://github.com/mdn/rari/issues/253)) ([ba2e792](https://github.com/mdn/rari/commit/ba2e7928cb000f1303f0cb62f3e0ffaeb12032f0))


### Miscellaneous

* **deps:** update major deps ([743c721](https://github.com/mdn/rari/commit/743c7219039100d45ecab9a17a506ff1f07440dd))
* **deps:** update minor deps ([fdac0e0](https://github.com/mdn/rari/commit/fdac0e0a85a992469f4eaf240168c6b3f3e37db2))

## [0.1.42](https://github.com/mdn/rari/compare/v0.1.41...v0.1.42) (2025-06-17)


### Features

* **json:** add fm description to generic and contributor ([#242](https://github.com/mdn/rari/issues/242)) ([b608389](https://github.com/mdn/rari/commit/b60838996ef9b2b39c825997c21be7539d0077e9))
* **json:** add parents to more pages ([#243](https://github.com/mdn/rari/issues/243)) ([5453fe9](https://github.com/mdn/rari/commit/5453fe91b498e2880648618e5597e369e778ff8f))


### Bug Fixes

* **homepage:** fallback for no generic content set ([#245](https://github.com/mdn/rari/issues/245)) ([7766a9c](https://github.com/mdn/rari/commit/7766a9c75bb50db2affd1691b849bdfeb7fd618f))

## [0.1.41](https://github.com/mdn/rari/compare/v0.1.40...v0.1.41) (2025-06-16)


### Features

* **client:** add GitHub token support for API requests ([#239](https://github.com/mdn/rari/issues/239)) ([c44bd1e](https://github.com/mdn/rari/commit/c44bd1eb15209a33d7e8b115fe6fd7b47d46038a))
* **homepage:** use data from generic content ([#238](https://github.com/mdn/rari/issues/238)) ([2e7b94e](https://github.com/mdn/rari/commit/2e7b94e8edaf4b4a7cb77575150da25f262374c6))
* **sidebar:** support args in fm sidebar ([#237](https://github.com/mdn/rari/issues/237)) ([82b80b0](https://github.com/mdn/rari/commit/82b80b07c462d047648c2c36a242caa8163d5ba5))


### Enhancements

* **sidebar:** wrap unlinked summary text in span ([#235](https://github.com/mdn/rari/issues/235)) ([a804805](https://github.com/mdn/rari/commit/a8048052176d499952c6fc7a5948f0fca890f4ec))


### Miscellaneous

* **sidebar:** remove unused toggle class ([#234](https://github.com/mdn/rari/issues/234)) ([6e79039](https://github.com/mdn/rari/commit/6e79039c0ebef43a088eb9d2ff037db6e2d1ac8f))

## [0.1.40](https://github.com/mdn/rari/compare/v0.1.39...v0.1.40) (2025-06-12)


### Features

* **lsp:** add an experimental language server to rari ([#230](https://github.com/mdn/rari/issues/230)) ([219962a](https://github.com/mdn/rari/commit/219962a1137ed1b1e9606c182c3f294481a4730e))


### Bug Fixes

* **css-syntax:** trim trailing line breaks ([#231](https://github.com/mdn/rari/issues/231)) ([553e614](https://github.com/mdn/rari/commit/553e6141f0aa7847c11cd931c5cb3127725c3a46))

## [0.1.39](https://github.com/mdn/rari/compare/v0.1.38...v0.1.39) (2025-06-05)


### Features

* **parser:** move to tree-sitter-mdn ([#219](https://github.com/mdn/rari/issues/219)) ([444c6b4](https://github.com/mdn/rari/commit/444c6b442b028345f8e0009a1c2790235aaf08fe))


### Bug Fixes

* **release-please:** configure changelog sections ([f1c12e1](https://github.com/mdn/rari/commit/f1c12e18960dfcec811826e27e340debb7b17ac6))


### Enhancements

* **rari-doc:** use root short-title in pageTitle ([#220](https://github.com/mdn/rari/issues/220)) ([a98b0d2](https://github.com/mdn/rari/commit/a98b0d2b67c0d1c40d9b19a83fc210dbbeaf8a6d))
* **sidebar:** make natural sort the default ([#225](https://github.com/mdn/rari/issues/225)) ([31154b8](https://github.com/mdn/rari/commit/31154b8b4f0ef38c25b24534974cb1f027f698e5))
* **sidebars:** add guides/tutorials to APIRef ([#224](https://github.com/mdn/rari/issues/224)) ([3bd6e0f](https://github.com/mdn/rari/commit/3bd6e0f9a555a05c75e2898afb21ff8a4b7025f7))


### Miscellaneous

* **page-types:** add `firefox-release-notes-active` page type ([185ea22](https://github.com/mdn/rari/commit/185ea222502a65eaefee2aafc9d8a85fc36a48f8))

## [0.1.38](https://github.com/mdn/rari/compare/v0.1.37...v0.1.38) (2025-05-19)


### Features

* **blog:** add pagination support ([#217](https://github.com/mdn/rari/issues/217)) ([beb4f44](https://github.com/mdn/rari/commit/beb4f4402cef9deacd64b6c960f0e12543300817))
* **serve:** serve blog assets ([e583414](https://github.com/mdn/rari/commit/e58341471cafca30a94f5bc509a6e3ea2fa4bd91))


### Bug Fixes

* **css_info:** fix bad "initial value" link ([71dfa1c](https://github.com/mdn/rari/commit/71dfa1cc57aa56508c7becbe5528fbbab9c956a5))
* **macro:** fix bad "initial value" link ([#214](https://github.com/mdn/rari/issues/214)) ([71dfa1c](https://github.com/mdn/rari/commit/71dfa1cc57aa56508c7becbe5528fbbab9c956a5))
* **rust:** bump workspace version ([aa052e1](https://github.com/mdn/rari/commit/aa052e14ba8ec3001277960dbf5c24329d9a25a8))

## [0.1.37](https://github.com/mdn/rari/compare/v0.1.36...v0.1.37) (2025-05-14)


### Bug Fixes

* **baseline:** fix per key calculation ([#212](https://github.com/mdn/rari/issues/212)) ([c714f43](https://github.com/mdn/rari/commit/c714f43a4eb93aab74a18f87f74c69fa58719c7c))
* **html:** escape titles ([d220c4f](https://github.com/mdn/rari/commit/d220c4fc1c0695145bf418801db5d8e1482b1cbf))
* **macro:** update redirected URLs in the CSSInfo macro ([#211](https://github.com/mdn/rari/issues/211)) ([921e512](https://github.com/mdn/rari/commit/921e5124284a62300d1a82632e08cc3ea078785d))
* **md:** render DD in order ([#207](https://github.com/mdn/rari/issues/207)) ([28d5aae](https://github.com/mdn/rari/commit/28d5aae65fa27a8d68cd7fac0a0bfea315637020))

## [0.1.36](https://github.com/mdn/rari/compare/v0.1.35...v0.1.36) (2025-04-30)


### Features

* **baseline:** display baseline by bcd key ([#200](https://github.com/mdn/rari/issues/200)) ([83a402d](https://github.com/mdn/rari/commit/83a402dc748aaaff7626c64e6310887d2217a190))
* **md:** update to comrak 0.38 ([#204](https://github.com/mdn/rari/issues/204)) ([ed5327b](https://github.com/mdn/rari/commit/ed5327b84b1d900e54c9167bf59ba18c1bfc14f7))
* **sidebars:** allow webgl-extension-method to show up in ApiRef sidebar ([#195](https://github.com/mdn/rari/issues/195)) ([cda09bc](https://github.com/mdn/rari/commit/cda09bcddc264965bc759f998384fed169fba9ca))
* **sidebars:** sort list subpages by short title ([#205](https://github.com/mdn/rari/issues/205)) ([117563a](https://github.com/mdn/rari/commit/117563acc5e741d247196f5b924acef18be65e80))

## [0.1.35](https://github.com/mdn/rari/compare/v0.1.34...v0.1.35) (2025-04-15)


### Features

* **livesamples:** tag live sample code blocks ([#193](https://github.com/mdn/rari/issues/193)) ([4cbe2ae](https://github.com/mdn/rari/commit/4cbe2ae3acd60b50e3b96d4122be0c471cf79f18))


### Bug Fixes

* **typescript:** fix baseline types ([4ae7a0a](https://github.com/mdn/rari/commit/4ae7a0ab4993a623d87a173a5de26b19995d27b3))

## [0.1.34](https://github.com/mdn/rari/compare/v0.1.33...v0.1.34) (2025-04-14)


### Bug Fixes

* **csssyntax:** fix color() ([58b506e](https://github.com/mdn/rari/commit/58b506e7230e76fba6b39ffb7e7d0f46dffa57af))
* **npm:** export types in an esm-compatible way ([#191](https://github.com/mdn/rari/issues/191)) ([8f523d7](https://github.com/mdn/rari/commit/8f523d7ead40582a6e75a48fff707d2ee1259c82))

## [0.1.33](https://github.com/mdn/rari/compare/v0.1.32...v0.1.33) (2025-04-08)


### Features

* **issues:** support ignoring issues ([#184](https://github.com/mdn/rari/issues/184)) ([df7636c](https://github.com/mdn/rari/commit/df7636cac43c1906bbf37e45df2a72ca5b37a898))
* **serve:** serve docs assets ([#185](https://github.com/mdn/rari/issues/185)) ([3c971ad](https://github.com/mdn/rari/commit/3c971adec5c117f2140eecc6eb0f7a3f1018c3e6))


### Bug Fixes

* **csssyntax:** add sources for properties ([68df8aa](https://github.com/mdn/rari/commit/68df8aacefb8f92d89a2b06daec7c7411d5d3469))

## [0.1.32](https://github.com/mdn/rari/compare/v0.1.31...v0.1.32) (2025-04-02)


### Features

* **css-syntax:** add sources ([#181](https://github.com/mdn/rari/issues/181)) ([31c4f5f](https://github.com/mdn/rari/commit/31c4f5f6b38508facadccbed00bae4c1fc6b6f21))
* **json:** add renderer to index.json for fred ([#180](https://github.com/mdn/rari/issues/180)) ([a290438](https://github.com/mdn/rari/commit/a290438627c1022ae5c9f0d2da1720db46efe625))


### Bug Fixes

* **dt:** don't add links if the dt contains a link ([6d81769](https://github.com/mdn/rari/commit/6d817692fe4fb2e8d5eab3edf82c7cc7700c44c5))
* **json:** make generic content template fm optional ([fb89132](https://github.com/mdn/rari/commit/fb89132213c2564ef81751dd086ab753e86a6d3d))
* **templ:** fix webextapi sidebar ([67d40bc](https://github.com/mdn/rari/commit/67d40bcc7c1fb7fdbd641302b57b52fa69016f60))

## [0.1.31](https://github.com/mdn/rari/compare/v0.1.30...v0.1.31) (2025-03-25)


### Features

* **templ:** add css_ref_list ([#177](https://github.com/mdn/rari/issues/177)) ([83f8241](https://github.com/mdn/rari/commit/83f8241bb7ab1a6b6a0d5eaff9f80180cb0b0236))


### Bug Fixes

* **deps:** fix package.json support ([5141e36](https://github.com/mdn/rari/commit/5141e36a95995a2fa438c190238ffdbe58aa4dde))
* **templ:** render glossary marco better in GlossaryDisambiguation ([8de1517](https://github.com/mdn/rari/commit/8de15175064dc66b31fad65942c8d015e43eeb34))
* update paths for SVG/MathML section move ([#156](https://github.com/mdn/rari/issues/156)) ([2a80835](https://github.com/mdn/rari/commit/2a808351fa5a9267bfe5b3bf25530480d4e41130))

## [0.1.30](https://github.com/mdn/rari/compare/v0.1.29...v0.1.30) (2025-03-14)


### Features

* **content:** support fix-flaws command ([#158](https://github.com/mdn/rari/issues/158)) ([8e2ba1c](https://github.com/mdn/rari/commit/8e2ba1c868c6aded2b25088fe6c919ebd55c17b2))


### Bug Fixes

* **issues:** calculate correct positions ([#154](https://github.com/mdn/rari/issues/154)) ([b97a9c2](https://github.com/mdn/rari/commit/b97a9c214452d8f89ca4da7928ad6814add16a9e))
* **templ:** don't report ill cased as broken in templs ([#159](https://github.com/mdn/rari/issues/159)) ([0927f47](https://github.com/mdn/rari/commit/0927f4772b9bdf3a3312d9b86bfaa53883535c16))
* update paths for HTTP section move ([#155](https://github.com/mdn/rari/issues/155)) ([ca4e711](https://github.com/mdn/rari/commit/ca4e7118e5600e7984997268c5c271fa84b84fdc))

## [0.1.29](https://github.com/mdn/rari/compare/v0.1.28...v0.1.29) (2025-03-13)


### Bug Fixes

* **homepage:** use text content for summary on homepage ([#146](https://github.com/mdn/rari/issues/146)) ([57758b2](https://github.com/mdn/rari/commit/57758b22231422d86e12036a0ed461d5bf70cc41))
* **issue-template:** make `CONTRIBUTING.md` link absolute ([#143](https://github.com/mdn/rari/issues/143)) ([29f914f](https://github.com/mdn/rari/commit/29f914fea3d3690fc3fc8d2f7e74cbd13be98db9))
* **sidebars:** strip "_static" suffix ([#145](https://github.com/mdn/rari/issues/145)) ([d4b32e0](https://github.com/mdn/rari/commit/d4b32e007543dd4017c783ec99774bd7e747cd42))
* **templ:** allow empty string for embed live sample ([bd063f3](https://github.com/mdn/rari/commit/bd063f3a0206a4cddab878364746f4909f827038))

## [0.1.28](https://github.com/mdn/rari/compare/v0.1.27...v0.1.28) (2025-03-12)


### Bug Fixes

* **npm:** rename types to remove `Json` prefix ([#147](https://github.com/mdn/rari/issues/147)) ([81248d1](https://github.com/mdn/rari/commit/81248d1114222e569167e9aae5b3da59aca08e32))

## [0.1.27](https://github.com/mdn/rari/compare/v0.1.26...v0.1.27) (2025-03-10)


### Features

* **templ:** don't report ill cased links for macros ([42411f5](https://github.com/mdn/rari/commit/42411f5202bddb15f93e7a88bad9c75e26310594))
* update featured articles ([c6d8eda](https://github.com/mdn/rari/commit/c6d8eda836f6ac055bd3d5d06f6ec32e7c020b13))

## [0.1.26](https://github.com/mdn/rari/compare/v0.1.25...v0.1.26) (2025-02-26)


### Features

* **redirects:** vaildate to urls better ([4360e26](https://github.com/mdn/rari/commit/4360e26cf6df08aed3251e8947d526eb500e0433))


### Bug Fixes

* **macro:** fix some CSS redirects ([#136](https://github.com/mdn/rari/issues/136)) ([6f284f4](https://github.com/mdn/rari/commit/6f284f49f1a7421b7fba5eaf14e23af154c7170f))
* **macro:** update redirects in CSSInfo ([#133](https://github.com/mdn/rari/issues/133)) ([28b1a2c](https://github.com/mdn/rari/commit/28b1a2ca4d56b514fab0c5b35791d833c6e264e1))

## [0.1.25](https://github.com/mdn/rari/compare/v0.1.24...v0.1.25) (2025-02-14)


### Features

* **build:** support --file-list ([fe44b01](https://github.com/mdn/rari/commit/fe44b01b04803f3d363d18406ee3eb5e8427a96a))
* **deps:** support DEPS_DATA_DIR env var to set data dir ([8dce98a](https://github.com/mdn/rari/commit/8dce98ad628724435cd2efdbd4ca12a85a0aa10e))
* **templ:** support sandbox attr in embedlivesample ([dd5ac86](https://github.com/mdn/rari/commit/dd5ac867f5ffc55793ccc0837ec6f11282c6fa78))
* **title:** update root_doc_url ([d2930a0](https://github.com/mdn/rari/commit/d2930a06ee6c39b3b916fad8c22938f9f66e7c27))
* **tools:** remove redirects that reference to deleted docs ([#126](https://github.com/mdn/rari/issues/126)) ([7f99fc6](https://github.com/mdn/rari/commit/7f99fc6035979a26183e3d0277b1cdba83f4b767))


### Bug Fixes

* **npm:** use `import` to dynamically load esm modules ([#130](https://github.com/mdn/rari/issues/130)) ([9fcfe1a](https://github.com/mdn/rari/commit/9fcfe1a8a516e683e8e1c024a9787e79011dc529))
* **redirects:** check for actual doc instead of path ([f9be824](https://github.com/mdn/rari/commit/f9be8240700d9952d458c7248e07d76fa1b2e817))
* **templ:** fix grouping for in cssref sidebar ([da781c5](https://github.com/mdn/rari/commit/da781c5a8e44b214ffda498c4fa6076429e8d4cd))
* **templ:** htmlelement/htmlxref don't lowercase ([4e331b3](https://github.com/mdn/rari/commit/4e331b38cd5d8be134f4836c7e70a19ae6a631a3))
* **workflows:** assign explicit permissions ([#123](https://github.com/mdn/rari/issues/123)) ([e446222](https://github.com/mdn/rari/commit/e4462229eb73e67a0b2d8ea2850cd05bef46a276))
* **workflows:** pin 3rd party actions ([#124](https://github.com/mdn/rari/issues/124)) ([0edec41](https://github.com/mdn/rari/commit/0edec412c012c82d08383b71f5dd2b652fc82cbb))

## [0.1.24](https://github.com/mdn/rari/compare/v0.1.23...v0.1.24) (2025-02-06)


### Bug Fixes

* **interactive-example:** missing quote at start of height attribute ([#120](https://github.com/mdn/rari/issues/120)) ([7fda86e](https://github.com/mdn/rari/commit/7fda86e5021c5ed307b0227ff05e86eeb12d8336))

## [0.1.23](https://github.com/mdn/rari/compare/v0.1.22...v0.1.23) (2025-02-05)


### Features

* **macros:** add InteractiveExample macro ([#84](https://github.com/mdn/rari/issues/84)) ([ca2f1e3](https://github.com/mdn/rari/commit/ca2f1e327cf275749b13fd414b99a5badf4639fd))


### Bug Fixes

* **move:** error when target directory exists ([3ce09de](https://github.com/mdn/rari/commit/3ce09de9e57ec17ae168453d1d774013b6f124ac))
* **validate-redirects:** validate to urls correct ([7de48c0](https://github.com/mdn/rari/commit/7de48c0f366529df530a683df0527f2893e2aa6c))

## [0.1.22](https://github.com/mdn/rari/compare/v0.1.21...v0.1.22) (2025-02-03)


### Features

* **build:** fail build on slug folder mismatch ([0628bc6](https://github.com/mdn/rari/commit/0628bc6eec244751ef1f02394080e4e84a1ff96f))
* **deps:** support versioning  ([#116](https://github.com/mdn/rari/issues/116)) ([e1e7418](https://github.com/mdn/rari/commit/e1e741804e93d9c512f00844a7a9ec466a736c81))
* **issues:** issues for ill cased links ([#115](https://github.com/mdn/rari/issues/115)) ([d8b6c2b](https://github.com/mdn/rari/commit/d8b6c2b393c85c0cdcc354c6062e30a03887a766))


### Bug Fixes

* **templ:** don't parse incomplete macro tags ([85692ec](https://github.com/mdn/rari/commit/85692eca0599131b7d2d9155daf5af5b238453c0))

## [0.1.21](https://github.com/mdn/rari/compare/v0.1.20...v0.1.21) (2025-01-29)


### Features

* **cli:** use info! instead of println! ([797e299](https://github.com/mdn/rari/commit/797e29999c222eb4cd4e4d0d55fa468232b72001))
* **templ:** support full syntax for csssyntaxraw ([1d78439](https://github.com/mdn/rari/commit/1d7843962ba9d2ccdc15b88106a373e16b705a0f))
* **tools:** add validate-redirects and support locale arg ([4488aed](https://github.com/mdn/rari/commit/4488aedb5949d66e5b1fc6e7c72b206c1bf182b7))


### Bug Fixes

* **build:** canonicalize file arguments ([5e2cccf](https://github.com/mdn/rari/commit/5e2cccf14638aa5b389b73c969540daf85366f18)), closes [#98](https://github.com/mdn/rari/issues/98)
* **build:** don't try to read generic pages if not set ([508b26f](https://github.com/mdn/rari/commit/508b26f8073a5cc014d7df8054635f108062e7a4))
* **frontmatter:** status is always an array ([5b5206b](https://github.com/mdn/rari/commit/5b5206b655c8672b23d13e655ef39520af8d1d35))
* **tool:** make `status` frontmatter serialize like in yari ([#109](https://github.com/mdn/rari/issues/109)) ([c43860d](https://github.com/mdn/rari/commit/c43860d4351bb3192004590756f6137440d580b8))
* **yaml:** force double quotes for fm and sidebars ([9890c7d](https://github.com/mdn/rari/commit/9890c7d321b9cc3d91054549552df3b88d826c83))

## [0.1.20](https://github.com/mdn/rari/compare/v0.1.19...v0.1.20) (2025-01-24)


### Features

* **issues:** improve flaw compatibility ([#95](https://github.com/mdn/rari/issues/95)) ([8b1f018](https://github.com/mdn/rari/commit/8b1f0181af3d5d4c0b48583f5a53b1d467108c23))

## [0.1.19](https://github.com/mdn/rari/compare/v0.1.18...v0.1.19) (2025-01-22)


### Features

* **templ:** add csssyntaxraw  ([#92](https://github.com/mdn/rari/issues/92)) ([25808bd](https://github.com/mdn/rari/commit/25808bdb2fa525b0d524d61f1354d05229bacc1d))


### Bug Fixes

* **build/parser:** parse empty string args for macros as `None`s ([#88](https://github.com/mdn/rari/issues/88)) ([4f5751f](https://github.com/mdn/rari/commit/4f5751fa506cf87366bc3dc777725df095913e8a))
* **html:** trim the first empty line in `&lt;pre&gt;` tag ([#90](https://github.com/mdn/rari/issues/90)) ([95f142f](https://github.com/mdn/rari/commit/95f142fecaad38cb696dc9606b2b9fa615c1f084))

## [0.1.18](https://github.com/mdn/rari/compare/v0.1.17...v0.1.18) (2025-01-16)


### Bug Fixes

* **build:** fix html img src ([b5af77e](https://github.com/mdn/rari/commit/b5af77e60be879c423f442fad5e0477423e17222))

## [0.1.17](https://github.com/mdn/rari/compare/v0.1.16...v0.1.17) (2025-01-14)


### Bug Fixes

* **blog:** respect published and date front matter ([ba9743d](https://github.com/mdn/rari/commit/ba9743d079eb2c0dc5c799dd516f3d222c98dc92))

## [0.1.16](https://github.com/mdn/rari/compare/v0.1.15...v0.1.16) (2025-01-10)


### Bug Fixes

* **issues:** add empty flaws for --json-issues ([c6be69e](https://github.com/mdn/rari/commit/c6be69eb91dfe71e7e12554ef04b49c5fcd0bc83))

## [0.1.15](https://github.com/mdn/rari/compare/v0.1.14...v0.1.15) (2025-01-09)


### Features

* **content:** add inventory command ([#80](https://github.com/mdn/rari/issues/80)) ([bafc0f9](https://github.com/mdn/rari/commit/bafc0f97479cf7210a39cd74d2f41450b92aff2f)), closes [#75](https://github.com/mdn/rari/issues/75)
* **sidebars:** add support for depth and nested ([#78](https://github.com/mdn/rari/issues/78)) ([84b6358](https://github.com/mdn/rari/commit/84b6358c4fb6783e36ea59649103c69e4eac397b))


### Bug Fixes

* **l10n:** correct the prefix string for notecards in zh-TW locale ([#81](https://github.com/mdn/rari/issues/81)) ([432bb40](https://github.com/mdn/rari/commit/432bb40b4729c66b186323428c872c033df51589))
* typo ([#82](https://github.com/mdn/rari/issues/82)) ([892f96f](https://github.com/mdn/rari/commit/892f96fa3403da2f35dc44c8a43a61d49e9837ed))

## [0.1.14](https://github.com/mdn/rari/compare/v0.1.13...v0.1.14) (2025-01-07)


### Features

* **baseline:** compute asterisk ([#77](https://github.com/mdn/rari/issues/77)) ([c35af8e](https://github.com/mdn/rari/commit/c35af8e46f3c57fdd3ba98cf92e63f2dd2064660))
* **sidebars/jsref:** support temporal ([#72](https://github.com/mdn/rari/issues/72)) ([3d70866](https://github.com/mdn/rari/commit/3d708668719208d1cbfd0b34959a771ee29bc350))


### Bug Fixes

* **json:** don't use camelCase for featured articles ([512080d](https://github.com/mdn/rari/commit/512080d804381d3dbc2facb93907e00c5d3c4bb1))

## [0.1.13](https://github.com/mdn/rari/compare/v0.1.12...v0.1.13) (2025-01-01)


### Bug Fixes

* **templ:** support argument for cssyntax ([330acdc](https://github.com/mdn/rari/commit/330acdcfc7d803b5bcc4656a9dcc093dad0339ee))

## [0.1.12](https://github.com/mdn/rari/compare/v0.1.11...v0.1.12) (2024-12-30)


### Bug Fixes

* **l10n:** correct the repo name to translated content ([#70](https://github.com/mdn/rari/issues/70)) ([e3b7209](https://github.com/mdn/rari/commit/e3b72093e8e7b37883b22acb09da6042963f54ed))

## [0.1.11](https://github.com/mdn/rari/compare/v0.1.10...v0.1.11) (2024-12-23)


### Features

* **build:** add -n compatibility ([8f27022](https://github.com/mdn/rari/commit/8f27022ad40cfdb54562e4ff46c19fa724d9f88e))


### Bug Fixes

* **l10n:** add sizes to fallback imgs ([2eab7ea](https://github.com/mdn/rari/commit/2eab7eaa99d0f876b67a87055cb54ddb28e31ff7))

## [0.1.10](https://github.com/mdn/rari/compare/v0.1.9...v0.1.10) (2024-12-20)


### Features

* **issues:** support json_issues flag ([761cafb](https://github.com/mdn/rari/commit/761cafbcff59a1b308162382c8852c0d30aba5a4))


### Bug Fixes

* **l10n:** fix fallback handling for Page::exists ([d532969](https://github.com/mdn/rari/commit/d5329699daade5b0bd62f17cecb5e394179efff3))
* **rari-npm:** make install faster ([1465576](https://github.com/mdn/rari/commit/1465576f9c7b34c3a6f1050363962af549c2162d))
* **templ:** don't show duplicates in webextexamples ([d6fb9d8](https://github.com/mdn/rari/commit/d6fb9d8ce706736a29f50ce0a93668a629a5aeba))

## [0.1.9](https://github.com/mdn/rari/compare/v0.1.8...v0.1.9) (2024-12-19)


### Features

* **baseline:** hide banner if discouraged ([#64](https://github.com/mdn/rari/issues/64)) ([bdfdb23](https://github.com/mdn/rari/commit/bdfdb230d9c5509d26eea353dfd0a97d83c51ab5))
* **css-definition-syntax:** support boolean-expr ([#58](https://github.com/mdn/rari/issues/58)) ([18baff1](https://github.com/mdn/rari/commit/18baff174fb08c0cdb47675978113f7eb1d9a3f8))

## [0.1.8](https://github.com/mdn/rari/compare/v0.1.7...v0.1.8) (2024-12-19)


### Bug Fixes

* **l10n:** improve en-US fallback in sidebars ([7f91855](https://github.com/mdn/rari/commit/7f918556511b4a5edf8f33c0a099015fb12f1333))

## [0.1.7](https://github.com/mdn/rari/compare/v0.1.6...v0.1.7) (2024-12-18)


### Features

* **sidebar:** add consolidation to fmt-sidebars ([9624a86](https://github.com/mdn/rari/commit/9624a86f110eef730c1d3e945530ee77b3239bc9))

## [0.1.6](https://github.com/mdn/rari/compare/v0.1.5...v0.1.6) (2024-12-18)


### Features

* **blog:** create rss.xml ([1a4c917](https://github.com/mdn/rari/commit/1a4c9172059e0a38e31181ff4f9edb1d8fe54ac5))

## [0.1.5](https://github.com/mdn/rari/compare/v0.1.4...v0.1.5) (2024-12-17)


### Features

* **cli:** support env_file ([2dd18b7](https://github.com/mdn/rari/commit/2dd18b71a1d596dee94acab27c24275fbc258216))

## [0.1.4](https://github.com/mdn/rari/compare/v0.1.3...v0.1.4) (2024-12-16)


### Bug Fixes

* **links:** unify link code ([e547623](https://github.com/mdn/rari/commit/e547623438b2274a1a6c29d7ca4916b28a1a43f4))

## [0.1.3](https://github.com/mdn/rari/compare/v0.1.2...v0.1.3) (2024-12-15)


### Bug Fixes

* **baseline:** support invalid baseline json ([bde35ee](https://github.com/mdn/rari/commit/bde35ee532e88c2531cdaa2cde051476e9dacda1))
* **serve:** support other_translations ([0fcf322](https://github.com/mdn/rari/commit/0fcf322adb0a68e8444a2b3e1d5620affddbbec4))

## [0.1.2](https://github.com/mdn/rari/compare/v0.1.1...v0.1.2) (2024-12-11)


### Features

* **build:** write top level metadata.json ([bbb1112](https://github.com/mdn/rari/commit/bbb111269c78acc87d73a1bb0c2e3d378ab21bac))

## [0.1.1](https://github.com/mdn/rari/compare/v0.1.0...v0.1.1) (2024-12-10)


### Features

* **md:** custom html escape ([a265450](https://github.com/mdn/rari/commit/a265450a599b72fae8276db7430989f057572f2f))


### Bug Fixes

* **links:** fall back to en-us ([21a7f18](https://github.com/mdn/rari/commit/21a7f1887a8381e65a348b19b43806217d8068c5))
* **links:** improve fallback for link content ([cdf0993](https://github.com/mdn/rari/commit/cdf09930a5bfa09cca6e270bb185bbcffe11c94c))
* **serve:** don't cache by default ([5ccf670](https://github.com/mdn/rari/commit/5ccf670248eb0d234c2bc9ca1e3baa852a3230c9))

## [0.1.0](https://github.com/mdn/rari/compare/v0.0.26...v0.1.0) (2024-12-05)


### ⚠ BREAKING CHANGES

* **cli:** Rari only builds build basic components by default. Use --all for old behavior.

### Features

* **cli:** new cli args ([5f0f7e6](https://github.com/mdn/rari/commit/5f0f7e69147772d198bd893677cce6fd49c9ec33))
* **serve:** 404 for document not found ([3435feb](https://github.com/mdn/rari/commit/3435feb071af72742983214db17f2878e69f46e2))


### Bug Fixes

* **generic_content:** fix locale in sitemap ([48d540c](https://github.com/mdn/rari/commit/48d540ce1d39576484b6c0f3f13c50e52b6508e7))
* **sidebars:** support listsubpages with code ([e287d44](https://github.com/mdn/rari/commit/e287d44cb8430267f89513a5cbcd323bfb7c5c6d))

## [0.0.26](https://github.com/mdn/rari/compare/v0.0.25...v0.0.26) (2024-12-01)


### Features

* **issues:** support sidebar name ([fd4ca80](https://github.com/mdn/rari/commit/fd4ca8082eadc80f0ec879c49affc07ee122c889))


### Bug Fixes

* **css-syntax:** support debugging ([ca2377d](https://github.com/mdn/rari/commit/ca2377dd180477ecbce0228a4666ba941d297642))
* **popularities:** update if not existing on 1st ([3f3c3d3](https://github.com/mdn/rari/commit/3f3c3d3a5a3db9e79b0b35acffea5bf5beb979c6))

## [0.0.25](https://github.com/mdn/rari/compare/v0.0.24...v0.0.25) (2024-11-27)


### Features

* **content:** add sync-sidebars command ([a4d4686](https://github.com/mdn/rari/commit/a4d4686fb432e312d0889368b6d6c40216f54cc9))
* **popularities:** move popularities to deps ([8e4b4aa](https://github.com/mdn/rari/commit/8e4b4aab2d33e31eaacd90c23ca8f7fa4b5a1f27))
* **spas:** default values for SPAs ([1ef6e16](https://github.com/mdn/rari/commit/1ef6e16a47502d6fee50a9a79d275649381a6d94))


### Bug Fixes

* **contributors:** support missing contributors ([d408b52](https://github.com/mdn/rari/commit/d408b5242d6e03fcae9f2be24b2afac68faa8d6f))
* **rari-npm:** fix download tmp folder ([5207886](https://github.com/mdn/rari/commit/5207886329cbc0645c805e22b629edd016a2a476))
* **spas:** fix default ([5acc69b](https://github.com/mdn/rari/commit/5acc69b581d1708bf39c9ba192d95a00ac3837f0))

## [0.0.24](https://github.com/mdn/rari/compare/v0.0.23...v0.0.24) (2024-11-25)


### Features

* **rari-npm:** use bin folder ([be311ef](https://github.com/mdn/rari/commit/be311ef84e1edc7da2d4cb50cfddaff3aed58b9a))


### Bug Fixes

* **build:** don't display error for files in files ([f3d4d15](https://github.com/mdn/rari/commit/f3d4d15eca4c3bddda738cd0ca53a03589e5164f))

## [0.0.23](https://github.com/mdn/rari/compare/v0.0.22...v0.0.23) (2024-11-22)


### Features

* **generics:** use config for generic content and some spas ([a717537](https://github.com/mdn/rari/commit/a7175374dc087372dffca0ee517fe48fb5d276f5))


### Bug Fixes

* **rari-npm:** don't ignore types an schema ([811ca0e](https://github.com/mdn/rari/commit/811ca0ee0a5a45fda44be9532706759b059cce9b))
* **sitemaps:** fix xml ([857d3f2](https://github.com/mdn/rari/commit/857d3f236a06a987933c72cad1e2814150d2a8a8))

## [0.0.22](https://github.com/mdn/rari/compare/v0.0.21...v0.0.22) (2024-11-21)


### Features

* **npm:** export ts types, json schema ([#42](https://github.com/mdn/rari/issues/42)) ([242b078](https://github.com/mdn/rari/commit/242b078d430eee15ef51e0a41da73d7bc898c4c5))
* **sitemaps:** write xml sitemaps ([a450474](https://github.com/mdn/rari/commit/a45047430772e9830e7055253c02bc403680cc5c))
* **templ:** new embedlivesample iframe ([e5382ca](https://github.com/mdn/rari/commit/e5382ca43e150b94a4391b36e3b9494073355270))


### Bug Fixes

* **templ:** cssxref and jsxref double issue reporting ([c20851b](https://github.com/mdn/rari/commit/c20851bcfb2b9bd94eff1afdc24905efcec0fff3))

## [0.0.21](https://github.com/mdn/rari/compare/v0.0.20...v0.0.21) (2024-11-18)


### Features

* **build:** generate contributors.txt ([b37d92b](https://github.com/mdn/rari/commit/b37d92b5ba76176a7702efbce2d222079f31dd16))
* **serve:** support contributors.txt ([5dfe87a](https://github.com/mdn/rari/commit/5dfe87a57cf1a42002e43da32f2b9a1fb1631d26))

## [0.0.20](https://github.com/mdn/rari/compare/v0.0.19...v0.0.20) (2024-11-16)


### Features

* **issues:** add data-href to broken links ([02833be](https://github.com/mdn/rari/commit/02833be0e5a11c705c2a9f7285748ada68adf2c8))
* **serve:** fast local search-index ([7076a81](https://github.com/mdn/rari/commit/7076a81092303b413b898d7e81d017c132d8faa1))
* **sidebars:** support hash links ([26a7a13](https://github.com/mdn/rari/commit/26a7a13f0e47123302bee47386d084a676495a4e))

## [0.0.19](https://github.com/mdn/rari/compare/v0.0.18...v0.0.19) (2024-11-13)


### Features

* **issues:** add initial support for macro issues ([5e23b0f](https://github.com/mdn/rari/commit/5e23b0fb3424fb5bc7d89a5a6de38ba851750338))
* **templ:** livesamplelink ([a95f39b](https://github.com/mdn/rari/commit/a95f39b4b70a9dbe00bcff09f4740735e23ae4ab))

## [0.0.18](https://github.com/mdn/rari/compare/v0.0.17...v0.0.18) (2024-11-08)


### Bug Fixes

* **templ:** fix wrong en-us-only ([54d6359](https://github.com/mdn/rari/commit/54d6359c11264438fdc4e7d061c11a1daef8c8e6))

## [0.0.17](https://github.com/mdn/rari/compare/v0.0.16...v0.0.17) (2024-11-07)


### Features

* **diff:** update html template ([7432365](https://github.com/mdn/rari/commit/74323653ae2cb44b86cf03d725519c88ed6e0442))
* **html:** post process dts ([#34](https://github.com/mdn/rari/issues/34)) ([ef6fbd7](https://github.com/mdn/rari/commit/ef6fbd7321a4d3c3aa9ce96b73ddfe0a37e82461))


### Bug Fixes

* **cssinfo:** add warning on empty result ([bdcca17](https://github.com/mdn/rari/commit/bdcca17a667c9fe00910f584b040b1cdc3af6f7c))
* **frontmatter:** update fm_types ([53f3711](https://github.com/mdn/rari/commit/53f371149ffabbe5cdcf491e7443226cbf7638f2))
* **html:** split out prose after specification ([a0af5e8](https://github.com/mdn/rari/commit/a0af5e8569cd8d7beea8e2289ec79ad8b2fcaabf))
* **templ:** don't trim string args ([dbb6d42](https://github.com/mdn/rari/commit/dbb6d425d7d9946e6dce40f0a0528a5a722fa1b8))
* **templ:** escape closing curly braces ([ef70385](https://github.com/mdn/rari/commit/ef70385545513fc66ca237027e5f75a747d04457))
* **templ:** fix listsubpages order ([6175e8f](https://github.com/mdn/rari/commit/6175e8f9842b829889a7b9c01d2d311dfaba3a68))
* **templ:** natural sort for utf8 ([3e86604](https://github.com/mdn/rari/commit/3e8660491d7f88a594360aeca828b4bb087e860a))

## [0.0.16](https://github.com/mdn/rari/compare/v0.0.15...v0.0.16) (2024-10-31)


### Features

* **html:** no href for page-not-found ([1a0695b](https://github.com/mdn/rari/commit/1a0695b9db2c7a048d757023a3b2889e7b3e6605))
* **issues:** issue counter ([bf9984e](https://github.com/mdn/rari/commit/bf9984e374b9c34183e009b15e119c0418f9badb))


### Bug Fixes

* **html:** unify code tags in pre ([d66b941](https://github.com/mdn/rari/commit/d66b94114fa89b19c4b708643b061595e1f1ffec))
* **release-please:** update self package ([14c6b97](https://github.com/mdn/rari/commit/14c6b97d9409e6018997fba8e8710d9b0bc3891b))
* **templ:** fix summary and inheritancediagram ([c0890a4](https://github.com/mdn/rari/commit/c0890a4c1c6962a70099f57bc5dd2830e343d229))

## [0.0.15](https://github.com/mdn/rari/compare/v0.0.14...v0.0.15) (2024-10-25)


### Features

* **issues:** initial flaw compat ([dc0c131](https://github.com/mdn/rari/commit/dc0c131c911ba026e97e8c160e9117f2fc033aa5))
* **rari-doc:** write metadata.json ([6244c4b](https://github.com/mdn/rari/commit/6244c4b2bb74afe84282eb1d10bcc91fd0f231c8))


### Bug Fixes

* **rari-npm:** fix windows arm ([9bba930](https://github.com/mdn/rari/commit/9bba9307b9bf24e40471354ca048ce6365f1cb7e))
* **rari-npm:** publish action ([62ad708](https://github.com/mdn/rari/commit/62ad708bf1ee6ba3e2ec4bc026d2b15c797eb986))

## [0.0.14](https://github.com/mdn/rari/compare/v0.0.13...v0.0.14) (2024-10-24)


### Features

* **cli:** sync translated content ([#24](https://github.com/mdn/rari/issues/24)) ([a3e3e87](https://github.com/mdn/rari/commit/a3e3e871d78319cd0f85dde1cee638d088442215))
* **rari-npm:** add initial support for npm ([e6ba05b](https://github.com/mdn/rari/commit/e6ba05b20b253a98d2f6646aceff1d6e094030a1))
* **rari-npm:** add workflow ([9c7baa4](https://github.com/mdn/rari/commit/9c7baa488daafdec18239059471bc1aa6d73d9fe))
* **rari-npm:** include cli script ([ee9a1d9](https://github.com/mdn/rari/commit/ee9a1d9f832f3789757841dedea8237f29e1fcc2))
* **rari-npm:** rename package ([4376a82](https://github.com/mdn/rari/commit/4376a8257fb82514fc8e532d41e9608205ebc4f4))


### Bug Fixes

* **css-sytax-types:** support &lt; rust 1.83 ([45f66b1](https://github.com/mdn/rari/commit/45f66b1f92df05e2879c6ec0b5ce646763be2bf2))
* **rari-npm:** fix download and node &lt; 22 ([11ec9ee](https://github.com/mdn/rari/commit/11ec9ee7a7c4135565d37caa5e85ba50892594b8))
* **rari-npm:** use version from package.json ([cf07c78](https://github.com/mdn/rari/commit/cf07c7873a772cae4e3e9b582dca0fdc8b1608fc))

## [0.0.13](https://github.com/mdn/rari/compare/v0.0.12...v0.0.13) (2024-10-18)


### Features

* **cli:** add content add-redirect ([#21](https://github.com/mdn/rari/issues/21)) ([df26184](https://github.com/mdn/rari/commit/df26184b880f3131e4fba46a88a66ed305b03510))
* **release:** add aarch64-windows ([54c02fb](https://github.com/mdn/rari/commit/54c02fb10dfd9a6e8eb40e8728a73a55b4151733))
* **spa:** 404 spa ([53db274](https://github.com/mdn/rari/commit/53db27437eb3c93fe15cde063a3919a1500b5776))
* **templ:** new unicode escape delimiter ([5740118](https://github.com/mdn/rari/commit/5740118a9902ed9ef03a3cd5b0b4b19ba9e2afe2))


### Bug Fixes

* **api_ref:** stabilize sort ([0a41adc](https://github.com/mdn/rari/commit/0a41adc644f1d71e942c17256292df56225aa31e))
* **diff:** fix subpages and banners ([309f6bf](https://github.com/mdn/rari/commit/309f6bf5fc81b9dff64e67e87a3f091a51f4de3c))
* **generics:** support community page and fixes ([c955a66](https://github.com/mdn/rari/commit/c955a6615a3d4ae80c786d60d93ec275c80b09d9))

## [0.0.12](https://github.com/mdn/rari/compare/v0.0.11...v0.0.12) (2024-10-14)


### Features

* **cli:** content delete ([#20](https://github.com/mdn/rari/issues/20)) ([b92bdec](https://github.com/mdn/rari/commit/b92bdec8d81c1f446b542c5acc9f45432b7950ba))
* **locale:** add German ([#4](https://github.com/mdn/rari/issues/4)) ([ba457cf](https://github.com/mdn/rari/commit/ba457cffb07fb320ae1bad35f13dcf1dd0d41380))
* **serve:** use axum ([b8ae516](https://github.com/mdn/rari/commit/b8ae516ba85adca6a93792b6fc06132bda9d5709))


### Bug Fixes

* **diff:** fast diff and various fixes ([560f198](https://github.com/mdn/rari/commit/560f1981be2067b57f7b98e9d7d9190dbc8c72ef))
* **diff:** pretty html diff and fixes ([c32b5d8](https://github.com/mdn/rari/commit/c32b5d87ed96048f656cd8e77c3b49144d15e200))
* **history:** enable translated content history ([7821bd2](https://github.com/mdn/rari/commit/7821bd204fd3600481b17f9cf6224d4f1d929338))
* **html:** don't remove p's in li's ([#19](https://github.com/mdn/rari/issues/19)) ([6cf911a](https://github.com/mdn/rari/commit/6cf911a526303072117f1106b448c8d454c0d2ea))
* **templ:** corrects ids from templates ([d4398c8](https://github.com/mdn/rari/commit/d4398c8f523ca577e55f3868bd218220803c357b))
* **templ:** escapte titles ([a0cdc7a](https://github.com/mdn/rari/commit/a0cdc7a91923d6ca2f3fbd86490be6d7963d9dee))
* **templ:** fix delimiter usage ([ac5d606](https://github.com/mdn/rari/commit/ac5d6067d36ca0f3e18c4fb733fc31cbfa4d3db6))
* **templ:** unescape strings in parser ([0d6b6ec](https://github.com/mdn/rari/commit/0d6b6ece9373c2843425460fb34a3d84373ddc36))

## [0.0.11](https://github.com/mdn/rari/compare/v0.0.10...v0.0.11) (2024-10-02)


### Bug Fixes

* **blog:** copy author avatars ([ccd5231](https://github.com/mdn/rari/commit/ccd5231df56e4d3d07a3812de293828c4cc6b821))
* **shot_title:** derive short_title for tags ([28f461f](https://github.com/mdn/rari/commit/28f461fd30f7b10a7f9e2601c88da743e6518c69))

## [0.0.10](https://github.com/mdn/rari/compare/v0.0.9...v0.0.10) (2024-10-02)


### Bug Fixes

* **rewriter:** don't wrap eveything in em ([7fe90a7](https://github.com/mdn/rari/commit/7fe90a76a9d83f1b4826b29e1b2bb2c9f8abe00a))

## [0.0.9](https://github.com/mdn/rari/compare/v0.0.8...v0.0.9) (2024-10-02)


### Features

* **generics:** prepare support for community page ([05d39f2](https://github.com/mdn/rari/commit/05d39f2a238feb32fe8969ef4cf225405d6566d6))
* **issues:** add initial support for issues ([df326d5](https://github.com/mdn/rari/commit/df326d5cf456cbed6e53d870ca2a33291fc038ea))
* **locales:** filter translated locales ([9cf56b4](https://github.com/mdn/rari/commit/9cf56b4ce90957251c349e374176bb26a64367e9))
* **tools:** implement move command ([#1](https://github.com/mdn/rari/issues/1)) ([51e04dc](https://github.com/mdn/rari/commit/51e04dc383ca4ff2ce211f863a2e7f01043d242b))


### Bug Fixes

* **locales:** rename all to for generics and spas ([e1721d1](https://github.com/mdn/rari/commit/e1721d171f5b0316a833c8bd32d5f10c91b35476))

## [0.0.8](https://github.com/mdn/rari/compare/v0.0.7...v0.0.8) (2024-09-19)


### Bug Fixes

* **ci:** no default features for self_update ([5b67dd3](https://github.com/mdn/rari/commit/5b67dd311779daa29d2520aa8871e3837bc9b69f))

## [0.0.7](https://github.com/mdn/rari/compare/v0.0.6...v0.0.7) (2024-09-19)


### Bug Fixes

* **ci:** use rustls for self_update ([79259c5](https://github.com/mdn/rari/commit/79259c5a05a5961b490d29bb185a1b73654687d4))

## [0.0.6](https://github.com/mdn/rari/compare/v0.0.5...v0.0.6) (2024-09-19)


### Bug Fixes

* **ci:** split pr and release ([2d10174](https://github.com/mdn/rari/commit/2d10174a837414163c8a5aac3b17f12bd9a62730))
* **ci:** use PAT ([c5f211a](https://github.com/mdn/rari/commit/c5f211a708428a134d1ff10336521c07cb55a0a7))

## [0.0.5](https://github.com/mdn/rari/compare/v0.0.4...v0.0.5) (2024-09-18)


### Bug Fixes

* **ci:** use tag as trigger ([2a06ed1](https://github.com/mdn/rari/commit/2a06ed1bbd388376e1ab71d4a8e1531909f3ee14))

## [0.0.4](https://github.com/mdn/rari/compare/v0.0.3...v0.0.4) (2024-09-18)


### Bug Fixes

* **ci:** use published as trigger ([a0c7a9a](https://github.com/mdn/rari/commit/a0c7a9a7b158dae57c8238ad6bc134fb556821d7))

## [0.0.3](https://github.com/mdn/rari/compare/v0.0.2...v0.0.3) (2024-09-18)


### Bug Fixes

* **ci:** release build ([257e3bc](https://github.com/mdn/rari/commit/257e3bc99e08f0102b009d067fbcbce4974ba170))

## [0.0.2](https://github.com/mdn/rari/compare/v0.0.1...v0.0.2) (2024-09-18)


### Features

* **ci:** use release-please ([ded5373](https://github.com/mdn/rari/commit/ded5373d9b487d8714934fe0089ee81880309272))


### Bug Fixes

* **ci:** add release-please manifest ([ca1d06e](https://github.com/mdn/rari/commit/ca1d06ea0f11eef5680fbe554cea8686d2490630))
* **ci:** correct tag ([28b0d4d](https://github.com/mdn/rari/commit/28b0d4d2a329e8f4f4d1170127174911faad7c7d))
* **ci:** empty release-please  manifest ([2c6baba](https://github.com/mdn/rari/commit/2c6baba226db676b9f28436bca2bf56751ea4bfe))
* **ci:** fix release-please files ([323e26a](https://github.com/mdn/rari/commit/323e26a9012b7646a98c92ee584a7a852c11ed96))
* **ci:** release-please again ([2dc9b7f](https://github.com/mdn/rari/commit/2dc9b7fda4e4a3b4aceaa6158a9d26d68506a844))
* **ci:** release-please again ([c843529](https://github.com/mdn/rari/commit/c843529bf7a89573372fb934f53da8be245cb84e))
* **ci:** release-plz again ([a95bb03](https://github.com/mdn/rari/commit/a95bb03677735860226c03f5d20f674c4dfd0704))

## [Unreleased]

## [0.0.1](https://github.com/mdn/rari/releases/tag/v0.0.1) - 2024-09-18

### Added

- *(release-plz)* add release-plz and use lto
- *(redirects)* short cuts
- *(seach-index)* build search index
- *(homepage)* build homepage
- *(generic_pages)* add support for generic pages
- *(SPA)* basic SPA support
- *(spas)* initial support for spas
- *(translations)* use en-us front matter
- *(translations)* add other translations field
- *(sitemap)* build sitemap.txt
- *(tmpl)* css_ref
- *(baseline)* support new baseline
- *(templ)* apilistalpha
- *(templ)* add webextallexamples and listgroups
- *(templ)* webextallcompat
- *(templ)* addonsidebarmain
- *(templs)* more on subpages and fixes
- *(templs)* lots of sidebars
- *(templ)* more sidebars
- *(templ)* addonsidebar
- *(deps)* remove once_cell
- *(templs)* glossarydisambiguation
- *(templs)* add js prop attr and svginfo
- *(sidebar)* l10n support
- *(templs)* embeds
- *(templs)* banners and http links
- *(templs)* prev next menu
- *(templs)* default api sidebar
- *(templs)* embed gh live sample
- *(templs)* many
- *(templs)* more banners
- *(templs)* add securecontext
- *(templs)* svgattrs and seecompattable
- feat(templs) webextapiref
- *(apiref)* heavy lifting
- *(templ)* post process inline sidebar
- *(sidebars)* start supporting inline sidebars
- *(templ)* first half of cssinfo
- *(templ)* template before html conversion!
- *(templ)* jsref + fixes
- *(rari)* initial commit

### Fixed

- *(ci)* fix release-plz
- *(ci)* fix upload and release-plz
- *(homepage)* use hyData
- *(build)* orphaned and conflicting
- *(various)* fix issues from testing
- *(build)* locale and redirect fixes
- *(search-index)* path
- *(many)* fix path related issues
- *(ids)* fix fixing ids
- *(l10n)* fix trimming
- fix p and summary
- *(banners)* add p
- *(templ)* cssinfo at 99%
- *(ids)* more dl issues
- *(ids)* more on dl ids
- *(ids)* start with 2?!
- *(ci)* remove openssl-sys dependency
- *(ci)* remvoe codeql

### Other

- *(deps)* update all and move to workspace
- *(deps)* update dependencies
- *(errors)* add io error with context
- *(md)* update note cards
- *(deps)* major
- *(deps)* all minor
- *(comrak)* 0.26
- rename macros -> templs
- move templs
- add custom sorter
- apiref
- templ stats and performance
- quicklinks and fixes
- missing page support
- update dpes
- more templs
- prefix img src
- new template escape
- add parser
- move l10n to content
