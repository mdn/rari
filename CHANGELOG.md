# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.14](https://github.com/mdn/rari/compare/v0.0.13...v0.0.14) (2024-10-18)


### Features

* **apiref:** heavy lifting ([b9ad1de](https://github.com/mdn/rari/commit/b9ad1decca0eb139e0de34d888c3c6f8e03bb170))
* **baseline:** support new baseline ([cd17a95](https://github.com/mdn/rari/commit/cd17a955b428038552e95ff2cd36ff7a6751f434))
* **ci:** use release-please ([ded5373](https://github.com/mdn/rari/commit/ded5373d9b487d8714934fe0089ee81880309272))
* **cli:** add content add-redirect ([#21](https://github.com/mdn/rari/issues/21)) ([df26184](https://github.com/mdn/rari/commit/df26184b880f3131e4fba46a88a66ed305b03510))
* **cli:** content delete ([#20](https://github.com/mdn/rari/issues/20)) ([b92bdec](https://github.com/mdn/rari/commit/b92bdec8d81c1f446b542c5acc9f45432b7950ba))
* **deps:** remove once_cell ([9942a57](https://github.com/mdn/rari/commit/9942a577756fe7ab44439b44ea928812fb9d950b))
* **generic_pages:** add support for generic pages ([971d92c](https://github.com/mdn/rari/commit/971d92cfc0a2b852b0afe88c2cb7f98944c53576))
* **generics:** prepare support for community page ([05d39f2](https://github.com/mdn/rari/commit/05d39f2a238feb32fe8969ef4cf225405d6566d6))
* **homepage:** build homepage ([56127ad](https://github.com/mdn/rari/commit/56127ad90c833e0141be8b87035c2d9a2ae3fca8))
* **issues:** add initial support for issues ([df326d5](https://github.com/mdn/rari/commit/df326d5cf456cbed6e53d870ca2a33291fc038ea))
* **locale:** add German ([#4](https://github.com/mdn/rari/issues/4)) ([ba457cf](https://github.com/mdn/rari/commit/ba457cffb07fb320ae1bad35f13dcf1dd0d41380))
* **locales:** filter translated locales ([9cf56b4](https://github.com/mdn/rari/commit/9cf56b4ce90957251c349e374176bb26a64367e9))
* **rari:** initial commit ([ef4894d](https://github.com/mdn/rari/commit/ef4894d4ea59363a928c958512a8843b62e122a0))
* **redirects:** short cuts ([fae9ba2](https://github.com/mdn/rari/commit/fae9ba2adf401e1b0b29ebc13eb56721076fdfad))
* **release-plz:** add release-plz and use lto ([f5742f3](https://github.com/mdn/rari/commit/f5742f30e58d7a57aa52b58aabdedb71ba4aa4a4))
* **release:** add aarch64-windows ([54c02fb](https://github.com/mdn/rari/commit/54c02fb10dfd9a6e8eb40e8728a73a55b4151733))
* **seach-index:** build search index ([8323644](https://github.com/mdn/rari/commit/8323644cc27258a214c6b0671780e8e8e3f17460))
* **serve:** use axum ([b8ae516](https://github.com/mdn/rari/commit/b8ae516ba85adca6a93792b6fc06132bda9d5709))
* **sidebar:** l10n support ([0b26aba](https://github.com/mdn/rari/commit/0b26aba841d6bea046e1b4454959fce069667471))
* **sidebars:** start supporting inline sidebars ([c9a7591](https://github.com/mdn/rari/commit/c9a7591c60205d1938d1b5c2d9f08f64b8c3b50c))
* **sitemap:** build sitemap.txt ([71adb36](https://github.com/mdn/rari/commit/71adb363f8e8d103bcb4a6b7442865f009d73fea))
* **spa:** 404 spa ([53db274](https://github.com/mdn/rari/commit/53db27437eb3c93fe15cde063a3919a1500b5776))
* **SPA:** basic SPA support ([a1ccbb1](https://github.com/mdn/rari/commit/a1ccbb19e79454a9b3bca83d8f545fe930160ad3))
* **spas:** initial support for spas ([526d841](https://github.com/mdn/rari/commit/526d841883978775768a609b2e7a42ffd6ce47ab))
* **templ:** add webextallexamples and listgroups ([728b5e3](https://github.com/mdn/rari/commit/728b5e34d69252643a6dc971acf42bb23425413b))
* **templ:** addonsidebar ([2667aad](https://github.com/mdn/rari/commit/2667aad3e035971334dbd3ed643ab22e19a669d9))
* **templ:** addonsidebarmain ([5c02fde](https://github.com/mdn/rari/commit/5c02fde5be0fe6935ecefe7dde57799476f196fa))
* **templ:** apilistalpha ([639a3e4](https://github.com/mdn/rari/commit/639a3e4640774404883da9c58a374e39e347b4d5))
* **templ:** first half of cssinfo ([b229209](https://github.com/mdn/rari/commit/b229209aebe559c60ee5efc8c523ca5f8c3cee1d))
* **templ:** jsref + fixes ([11feeb3](https://github.com/mdn/rari/commit/11feeb338fcf86823fae45f8524f03e5177ec746))
* **templ:** more sidebars ([fe7d2e8](https://github.com/mdn/rari/commit/fe7d2e88e5cf32d356a0ff84a442423fea691dd6))
* **templ:** new unicode escape delimiter ([5740118](https://github.com/mdn/rari/commit/5740118a9902ed9ef03a3cd5b0b4b19ba9e2afe2))
* **templ:** post process inline sidebar ([648becb](https://github.com/mdn/rari/commit/648becb8018d8cc4619300310cf0873f9c28f64f))
* **templs:** add js prop attr and svginfo ([56bf52a](https://github.com/mdn/rari/commit/56bf52a79d79c3fdff3788d7677971e011a59280))
* **templs:** add securecontext ([aac156e](https://github.com/mdn/rari/commit/aac156e4a999c6976c5e6f30a69dadaefbc0979f))
* **templs:** banners and http links ([794bc21](https://github.com/mdn/rari/commit/794bc2190483feadc3a6ed56502e70e4c975b697))
* **templs:** default api sidebar ([c64eff1](https://github.com/mdn/rari/commit/c64eff1ae7e3e8e2fd33cc3d650ab87eaf82a8ac))
* **templs:** embed gh live sample ([008587f](https://github.com/mdn/rari/commit/008587ff31daf9624500b77915cc08e623e71781))
* **templs:** embeds ([b150984](https://github.com/mdn/rari/commit/b15098403815c7580c7855f4b1d688974ff888c1))
* **templs:** glossarydisambiguation ([18d57ba](https://github.com/mdn/rari/commit/18d57ba2ccef4acf3246a87c9a70e461f8352462))
* **templs:** lots of sidebars ([4675517](https://github.com/mdn/rari/commit/467551703420788ae486969ebb840097e24c1da9))
* **templs:** many ([0919d7d](https://github.com/mdn/rari/commit/0919d7dda60a6a0015a68964b0b8c58de82e25b0))
* **templs:** more banners ([bc2a857](https://github.com/mdn/rari/commit/bc2a8573ac63da4e4ebee07b86082f076624dc41))
* **templs:** more on subpages and fixes ([792c407](https://github.com/mdn/rari/commit/792c40734bc457bd01e21fa211010ab1d3c190a8))
* **templs:** prev next menu ([c75d3bf](https://github.com/mdn/rari/commit/c75d3bf44e6bc7e16c63b91cc97bdfea83cfc968))
* **templs:** svgattrs and seecompattable ([42fd8a9](https://github.com/mdn/rari/commit/42fd8a9a2e50ea53f49f2721eb5e5e93632d1a78))
* **templ:** template before html conversion! ([46abf2b](https://github.com/mdn/rari/commit/46abf2b764afed0793330f678640758acfd6d950))
* **templ:** webextallcompat ([fc799b8](https://github.com/mdn/rari/commit/fc799b83112f57296130fd8cd1faaef0c389d103))
* **tmpl:** css_ref ([1468878](https://github.com/mdn/rari/commit/1468878a6b1b744c5ed8ec88ee1bdc11e85f6da4))
* **tools:** implement move command ([#1](https://github.com/mdn/rari/issues/1)) ([51e04dc](https://github.com/mdn/rari/commit/51e04dc383ca4ff2ce211f863a2e7f01043d242b))
* **translations:** add other translations field ([10c4805](https://github.com/mdn/rari/commit/10c48050a15459bd160736556152494228c9043b))
* **translations:** use en-us front matter ([2b84868](https://github.com/mdn/rari/commit/2b84868050bbe9c21ac504fbc9feb808e63d27b9))


### Bug Fixes

* **api_ref:** stabilize sort ([0a41adc](https://github.com/mdn/rari/commit/0a41adc644f1d71e942c17256292df56225aa31e))
* **banners:** add p ([ac7c8c0](https://github.com/mdn/rari/commit/ac7c8c0b6dc103a46347fde152330b646a9cd831))
* **blog:** copy author avatars ([ccd5231](https://github.com/mdn/rari/commit/ccd5231df56e4d3d07a3812de293828c4cc6b821))
* **build:** locale and redirect fixes ([126d6ac](https://github.com/mdn/rari/commit/126d6acf29b84b4b3c0ed48855b880b33b1f65aa))
* **build:** orphaned and conflicting ([a353b88](https://github.com/mdn/rari/commit/a353b88aee2ea792cdc19c0372d29dd6af7b35dd))
* **ci:** add release-please manifest ([ca1d06e](https://github.com/mdn/rari/commit/ca1d06ea0f11eef5680fbe554cea8686d2490630))
* **ci:** correct tag ([28b0d4d](https://github.com/mdn/rari/commit/28b0d4d2a329e8f4f4d1170127174911faad7c7d))
* **ci:** empty release-please  manifest ([2c6baba](https://github.com/mdn/rari/commit/2c6baba226db676b9f28436bca2bf56751ea4bfe))
* **ci:** fix release-please files ([323e26a](https://github.com/mdn/rari/commit/323e26a9012b7646a98c92ee584a7a852c11ed96))
* **ci:** fix release-plz ([f1ed561](https://github.com/mdn/rari/commit/f1ed561ffdaf53853e3282d4880c10b7253c956e))
* **ci:** fix upload and release-plz ([18bd40a](https://github.com/mdn/rari/commit/18bd40a381b51c3f1985463baa08f5087d7f6a5e))
* **ci:** no default features for self_update ([5b67dd3](https://github.com/mdn/rari/commit/5b67dd311779daa29d2520aa8871e3837bc9b69f))
* **ci:** release build ([257e3bc](https://github.com/mdn/rari/commit/257e3bc99e08f0102b009d067fbcbce4974ba170))
* **ci:** release-please again ([2dc9b7f](https://github.com/mdn/rari/commit/2dc9b7fda4e4a3b4aceaa6158a9d26d68506a844))
* **ci:** release-please again ([c843529](https://github.com/mdn/rari/commit/c843529bf7a89573372fb934f53da8be245cb84e))
* **ci:** release-plz again ([a95bb03](https://github.com/mdn/rari/commit/a95bb03677735860226c03f5d20f674c4dfd0704))
* **ci:** remove openssl-sys dependency ([c307ad8](https://github.com/mdn/rari/commit/c307ad88cbd8c7c9255c587577b48c1f0d6b4c9e))
* **ci:** remvoe codeql ([1787026](https://github.com/mdn/rari/commit/178702649ed5e8d68f2cbf0a2165cef34ba730c8))
* **ci:** split pr and release ([2d10174](https://github.com/mdn/rari/commit/2d10174a837414163c8a5aac3b17f12bd9a62730))
* **ci:** use PAT ([c5f211a](https://github.com/mdn/rari/commit/c5f211a708428a134d1ff10336521c07cb55a0a7))
* **ci:** use published as trigger ([a0c7a9a](https://github.com/mdn/rari/commit/a0c7a9a7b158dae57c8238ad6bc134fb556821d7))
* **ci:** use rustls for self_update ([79259c5](https://github.com/mdn/rari/commit/79259c5a05a5961b490d29bb185a1b73654687d4))
* **ci:** use tag as trigger ([2a06ed1](https://github.com/mdn/rari/commit/2a06ed1bbd388376e1ab71d4a8e1531909f3ee14))
* **diff:** fast diff and various fixes ([560f198](https://github.com/mdn/rari/commit/560f1981be2067b57f7b98e9d7d9190dbc8c72ef))
* **diff:** fix subpages and banners ([309f6bf](https://github.com/mdn/rari/commit/309f6bf5fc81b9dff64e67e87a3f091a51f4de3c))
* **diff:** pretty html diff and fixes ([c32b5d8](https://github.com/mdn/rari/commit/c32b5d87ed96048f656cd8e77c3b49144d15e200))
* **generics:** support community page and fixes ([c955a66](https://github.com/mdn/rari/commit/c955a6615a3d4ae80c786d60d93ec275c80b09d9))
* **history:** enable translated content history ([7821bd2](https://github.com/mdn/rari/commit/7821bd204fd3600481b17f9cf6224d4f1d929338))
* **homepage:** use hyData ([2981fea](https://github.com/mdn/rari/commit/2981fea089104a017ddea2f8b56e25a6b1f8802c))
* **html:** don't remove p's in li's ([#19](https://github.com/mdn/rari/issues/19)) ([6cf911a](https://github.com/mdn/rari/commit/6cf911a526303072117f1106b448c8d454c0d2ea))
* **ids:** fix fixing ids ([18146d1](https://github.com/mdn/rari/commit/18146d12dc9bb14c18741bd1c06bd4e5bdbeba10))
* **ids:** more dl issues ([de1fd97](https://github.com/mdn/rari/commit/de1fd97a18432fb81425a0241bc59581732f05e9))
* **ids:** more on dl ids ([79901f1](https://github.com/mdn/rari/commit/79901f15b17988fc02e94fccb6dd1ce33cc2b393))
* **ids:** start with 2?! ([f2fa543](https://github.com/mdn/rari/commit/f2fa5438d009b14548b182f931f654ec11e5682b))
* **l10n:** fix trimming ([667666d](https://github.com/mdn/rari/commit/667666dcc147440e57ce9ba3f25c6d17ac135bfe))
* **locales:** rename all to for generics and spas ([e1721d1](https://github.com/mdn/rari/commit/e1721d171f5b0316a833c8bd32d5f10c91b35476))
* **many:** fix path related issues ([e2a6ebc](https://github.com/mdn/rari/commit/e2a6ebca54d52119be225d2ca03b1904f775aec4))
* **rewriter:** don't wrap eveything in em ([7fe90a7](https://github.com/mdn/rari/commit/7fe90a76a9d83f1b4826b29e1b2bb2c9f8abe00a))
* **search-index:** path ([bbe8f80](https://github.com/mdn/rari/commit/bbe8f80265ef5c83dad275579d7e8d1f912628ea))
* **shot_title:** derive short_title for tags ([28f461f](https://github.com/mdn/rari/commit/28f461fd30f7b10a7f9e2601c88da743e6518c69))
* **templ:** corrects ids from templates ([d4398c8](https://github.com/mdn/rari/commit/d4398c8f523ca577e55f3868bd218220803c357b))
* **templ:** cssinfo at 99% ([b19f4fc](https://github.com/mdn/rari/commit/b19f4fc721330a210623046e6a56c756489d5004))
* **templ:** escapte titles ([a0cdc7a](https://github.com/mdn/rari/commit/a0cdc7a91923d6ca2f3fbd86490be6d7963d9dee))
* **templ:** fix delimiter usage ([ac5d606](https://github.com/mdn/rari/commit/ac5d6067d36ca0f3e18c4fb733fc31cbfa4d3db6))
* **templ:** unescape strings in parser ([0d6b6ec](https://github.com/mdn/rari/commit/0d6b6ece9373c2843425460fb34a3d84373ddc36))
* **various:** fix issues from testing ([675a738](https://github.com/mdn/rari/commit/675a73829aef3601f3fed4fd0751b25b503f151b))

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
