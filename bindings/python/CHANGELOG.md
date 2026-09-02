# Changelog

## [2.2.0](https://github.com/phenixrizen/zen/compare/python-v2.1.0...python-v2.2.0) (2026-09-02)


### Features

* **python:** sqliteConfig installs the SQLite handler; fix: empty relation renders zero rows ([#8](https://github.com/phenixrizen/zen/issues/8)) ([b5e0f3d](https://github.com/phenixrizen/zen/commit/b5e0f3d0ed14b3201342b29512df056f1dbf061b))

## [2.1.0](https://github.com/phenixrizen/zen/compare/python-v2.0.2...python-v2.1.0) (2026-08-27)


### Features

* add async support to python binding ([#185](https://github.com/phenixrizen/zen/issues/185)) ([4c9c26b](https://github.com/phenixrizen/zen/commit/4c9c26b4296fb9b6d75a46215976fbe51637ba59))
* add validation methods to python bindings ([#305](https://github.com/phenixrizen/zen/issues/305)) ([94e1da2](https://github.com/phenixrizen/zen/commit/94e1da239af1a89e9de4c6687460a3fac5fa5dec))
* compact trace ([#384](https://github.com/phenixrizen/zen/issues/384)) ([80fe402](https://github.com/phenixrizen/zen/commit/80fe4021c216d267bee388a1959a5ae29f279f0d))
* compiled bytecode ([#307](https://github.com/phenixrizen/zen/issues/307)) ([ae40aff](https://github.com/phenixrizen/zen/commit/ae40aff1638bde0011954ba199b831657f155928))
* configurable arbitrary precision ([#433](https://github.com/phenixrizen/zen/issues/433)) ([46688a4](https://github.com/phenixrizen/zen/commit/46688a4d4ce72f23db22b4397827b28529a5d71d))
* custom node ([#138](https://github.com/phenixrizen/zen/issues/138)) ([daecf90](https://github.com/phenixrizen/zen/commit/daecf901e6576df0ddd9d24dbc2aed6774b4599f))
* date v2 ([#345](https://github.com/phenixrizen/zen/issues/345)) ([de6d13e](https://github.com/phenixrizen/zen/commit/de6d13e1c7f5b5e4b9480f4d0c39b998b208cf0f))
* function v2 ([#212](https://github.com/phenixrizen/zen/issues/212)) ([cc3d938](https://github.com/phenixrizen/zen/commit/cc3d938b2f21bda6b66e7c38b0cfc34df239a5c9))
* implement new loaders across languages ([#487](https://github.com/phenixrizen/zen/issues/487)) ([1818371](https://github.com/phenixrizen/zen/commit/1818371c908730b73e69e058d71099dccb537eab))
* passthrough nodes ([#261](https://github.com/phenixrizen/zen/issues/261)) ([4781214](https://github.com/phenixrizen/zen/commit/47812143a031e92fedde3b406502f3dc0cfb9dbd))
* policy engine ([#450](https://github.com/phenixrizen/zen/issues/450)) ([6c0ca51](https://github.com/phenixrizen/zen/commit/6c0ca513546dd8cb3664e6926da66f146d9726d9))
* precompile decision content ([#401](https://github.com/phenixrizen/zen/issues/401)) ([7bfebd9](https://github.com/phenixrizen/zen/commit/7bfebd90fed78d3250e25f92a4c971b71f850532))
* publish under phenixrizen package names ([01d81d0](https://github.com/phenixrizen/zen/commit/01d81d07b8fa7687a73bb562ba38ff9e49cb787c))
* py bindings refactoring ([#317](https://github.com/phenixrizen/zen/issues/317)) ([ff34aff](https://github.com/phenixrizen/zen/commit/ff34aff821537630d63d7b7b313ce177df87c836))
* pyexpr remove evaluate many ([#314](https://github.com/phenixrizen/zen/issues/314)) ([a16009b](https://github.com/phenixrizen/zen/commit/a16009be00b3ca33adf5fa43bde1e941dfcdce98))
* Python Stub File ([#181](https://github.com/phenixrizen/zen/issues/181)) ([15c4384](https://github.com/phenixrizen/zen/commit/15c4384494befca8d5bc1f1f8f9fcf63840c7264))
* rc variable ([#246](https://github.com/phenixrizen/zen/issues/246)) ([9159816](https://github.com/phenixrizen/zen/commit/91598166ce912b8d8f53441d5e9fa8a02bf9855a))
* refactor engine ([#390](https://github.com/phenixrizen/zen/issues/390)) ([9150982](https://github.com/phenixrizen/zen/commit/91509821be632bc7305648d2f6f4ce62f84b4c60))
* update pyo3 ([#308](https://github.com/phenixrizen/zen/issues/308)) ([be305a4](https://github.com/phenixrizen/zen/commit/be305a4fba5a04040f1f7dea8266dff41ae80e0a))
* zen expression rewrite ([#107](https://github.com/phenixrizen/zen/issues/107)) ([5f423b7](https://github.com/phenixrizen/zen/commit/5f423b7910feb62d28c84c159705dc8db296d469))


### Bug Fixes

* correct readme examples and java native library loader ([#518](https://github.com/phenixrizen/zen/issues/518)) ([8594f9a](https://github.com/phenixrizen/zen/commit/8594f9a5fbb478b9de1b692851c787064cc5ae66))
* doc update ([#120](https://github.com/phenixrizen/zen/issues/120)) ([90ad94d](https://github.com/phenixrizen/zen/commit/90ad94dfa7137442d11127290f68dda1edfd2dd4))
* docs ([#122](https://github.com/phenixrizen/zen/issues/122)) ([64cd27a](https://github.com/phenixrizen/zen/commit/64cd27ac1add38e52c7c2aba6f27a19ac5dbac9e))
* improve READMEs across bindings ([#516](https://github.com/phenixrizen/zen/issues/516)) ([692ca2f](https://github.com/phenixrizen/zen/commit/692ca2fe6b503a42da6b311dbefdf603a3ae5df4))
* number out of range panics ([#506](https://github.com/phenixrizen/zen/issues/506)) ([1372274](https://github.com/phenixrizen/zen/commit/13722740a1c1af8b2c5b2bbf93536d46b95af812))
* py evalaute options ([#358](https://github.com/phenixrizen/zen/issues/358)) ([4ba5bf2](https://github.com/phenixrizen/zen/commit/4ba5bf207b552dee2e3cca53b2f9318766550e55))
* py validate methods ([#316](https://github.com/phenixrizen/zen/issues/316)) ([d762431](https://github.com/phenixrizen/zen/commit/d7624311f60df5c262ed0cd5bd273e1f2362b2c7))
* python async ([#309](https://github.com/phenixrizen/zen/issues/309)) ([55a6320](https://github.com/phenixrizen/zen/commit/55a632058075a911deceb6130010ed26e90c2210))
* python asyncio ([#217](https://github.com/phenixrizen/zen/issues/217)) ([9e60fdc](https://github.com/phenixrizen/zen/commit/9e60fdc882b979b9a7c28fc51fdf41317d1d4796))
* rename templates crate ([#140](https://github.com/phenixrizen/zen/issues/140)) ([ebba323](https://github.com/phenixrizen/zen/commit/ebba3233668779fa510a7d059c027eea43137929))
* update dependencies ([#102](https://github.com/phenixrizen/zen/issues/102)) ([20a6856](https://github.com/phenixrizen/zen/commit/20a68564c60f77a91adf3c7df9d54d460b839e1c))
* upgrade crates ([#111](https://github.com/phenixrizen/zen/issues/111)) ([f1f4cb4](https://github.com/phenixrizen/zen/commit/f1f4cb4b08420604963716909c569d4a6fa67c9c))


### Performance

* binding variable conversion (python and nodejs) ([#474](https://github.com/phenixrizen/zen/issues/474)) ([b6fac7c](https://github.com/phenixrizen/zen/commit/b6fac7c7a64fa21bbcace0d7f51293d670980f62))
* general performance improvements ([#500](https://github.com/phenixrizen/zen/issues/500)) ([fe43b4c](https://github.com/phenixrizen/zen/commit/fe43b4c1d71fb28c247ffb0ebd9e332d34f5f907))

## [2.0.2](https://github.com/gorules/zen/compare/python-v2.0.1...python-v2.0.2) (2026-08-24)


### Bug Fixes

* correct readme examples and java native library loader ([#518](https://github.com/gorules/zen/issues/518)) ([8594f9a](https://github.com/gorules/zen/commit/8594f9a5fbb478b9de1b692851c787064cc5ae66))

## [2.0.1](https://github.com/gorules/zen/compare/python-v2.0.0...python-v2.0.1) (2026-08-22)


### Bug Fixes

* improve READMEs across bindings ([#516](https://github.com/gorules/zen/issues/516)) ([692ca2f](https://github.com/gorules/zen/commit/692ca2fe6b503a42da6b311dbefdf603a3ae5df4))

## [2.0.0](https://github.com/gorules/zen/compare/python-v1.0.0-beta.14...python-v2.0.0) (2026-08-20)


### Bug Fixes

* number out of range panics ([#506](https://github.com/gorules/zen/issues/506)) ([1372274](https://github.com/gorules/zen/commit/13722740a1c1af8b2c5b2bbf93536d46b95af812))

## [1.0.0-beta.14](https://github.com/gorules/zen/compare/python-v1.0.0-beta.13...python-v1.0.0-beta.14) (2026-08-18)

## [1.0.0-beta.13](https://github.com/gorules/zen/compare/python-v1.0.0-beta.12...python-v1.0.0-beta.13) (2026-08-07)


### Performance

* general performance improvements ([#500](https://github.com/gorules/zen/issues/500)) ([fe43b4c](https://github.com/gorules/zen/commit/fe43b4c1d71fb28c247ffb0ebd9e332d34f5f907))

## [1.0.0-beta.12](https://github.com/gorules/zen/compare/python-v1.0.0-beta.11...python-v1.0.0-beta.12) (2026-07-26)

## [1.0.0-beta.11](https://github.com/gorules/zen/compare/python-v1.0.0-beta.10...python-v1.0.0-beta.11) (2026-07-24)

## [1.0.0-beta.10](https://github.com/gorules/zen/compare/python-v1.0.0-beta.9...python-v1.0.0-beta.10) (2026-07-22)

## [1.0.0-beta.9](https://github.com/gorules/zen/compare/python-v1.0.0-beta.8...python-v1.0.0-beta.9) (2026-07-22)

## [1.0.0-beta.8](https://github.com/gorules/zen/compare/python-v1.0.0-beta.7...python-v1.0.0-beta.8) (2026-07-20)


### Features

* implement new loaders across languages ([#487](https://github.com/gorules/zen/issues/487)) ([1818371](https://github.com/gorules/zen/commit/1818371c908730b73e69e058d71099dccb537eab))

## [1.0.0-beta.7](https://github.com/gorules/zen/compare/python-v1.0.0-beta.6...python-v1.0.0-beta.7) (2026-07-16)

## [1.0.0-beta.6](https://github.com/gorules/zen/compare/python-v1.0.0-beta.5...python-v1.0.0-beta.6) (2026-07-10)

## [1.0.0-beta.5](https://github.com/gorules/zen/compare/python-v1.0.0-beta.4...python-v1.0.0-beta.5) (2026-07-08)

## [1.0.0-beta.4](https://github.com/gorules/zen/compare/python-v1.0.0-beta.3...python-v1.0.0-beta.4) (2026-07-07)

## [1.0.0-beta.3](https://github.com/gorules/zen/compare/python-v1.0.0-beta.2...python-v1.0.0-beta.3) (2026-06-29)


### Performance

* binding variable conversion (python and nodejs) ([#474](https://github.com/gorules/zen/issues/474)) ([b6fac7c](https://github.com/gorules/zen/commit/b6fac7c7a64fa21bbcace0d7f51293d670980f62))

## [1.0.0-beta.2](https://github.com/gorules/zen/compare/python-v1.0.0-beta.1...python-v1.0.0-beta.2) (2026-06-26)

## [1.0.0-beta.1](https://github.com/gorules/zen/compare/python-v1.0.0-beta.0...python-v1.0.0-beta.1) (2026-06-25)

## [1.0.0-beta.0](https://github.com/gorules/zen/compare/python-v0.53.0...python-v1.0.0-beta.0) (2026-06-25)


### Features

* policy engine ([#450](https://github.com/gorules/zen/issues/450)) ([6c0ca51](https://github.com/gorules/zen/commit/6c0ca513546dd8cb3664e6926da66f146d9726d9))
