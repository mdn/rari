# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
