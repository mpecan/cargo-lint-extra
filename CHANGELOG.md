# Changelog

## [0.1.4](https://github.com/mpecan/cargo-lint-extra/compare/cargo-lint-extra-v0.1.3...cargo-lint-extra-v0.1.4) (2026-03-12)


### Features

* add cargo-binstall metadata for prebuilt binary installation ([34c7b44](https://github.com/mpecan/cargo-lint-extra/commit/34c7b4415d801ecff6fbfc97de5bbceb2cf4f393))
* exit 0 for warnings, exit 1 for errors, add -W flag ([72386f5](https://github.com/mpecan/cargo-lint-extra/commit/72386f52008fff52b1cb6c3d56203b1de9f52036))


### Bug Fixes

* level=deny promotes soft violations, deprecation warning, docs ([a4fb8d6](https://github.com/mpecan/cargo-lint-extra/commit/a4fb8d675d2561c40545e2b30c20fb74642fa0e2))

## [0.1.3](https://github.com/mpecan/cargo-lint-extra/compare/cargo-lint-extra-v0.1.2...cargo-lint-extra-v0.1.3) (2026-03-12)


### Features

* add soft/hard limits to file-length rule ([e9a6d80](https://github.com/mpecan/cargo-lint-extra/commit/e9a6d800d6f8b0505e0c56cd8a2ffb50e8c6c133))

## [0.1.2](https://github.com/mpecan/cargo-lint-extra/compare/cargo-lint-extra-v0.1.1...cargo-lint-extra-v0.1.2) (2026-03-12)


### Features

* add comment-based inline suppression for individual lines, blocks, and functions ([c26ee5d](https://github.com/mpecan/cargo-lint-extra/commit/c26ee5d4feaeff5e3d8f7221ecb748a6e144090c))
* add test-code-specific rule overrides via [test] config section ([0230309](https://github.com/mpecan/cargo-lint-extra/commit/02303095a5746761264b5913639963f8bc8cd923))


### Bug Fixes

* add cross-compilation targets to rust-toolchain.toml ([ffffb7d](https://github.com/mpecan/cargo-lint-extra/commit/ffffb7db9ccfb1bcb8bcb107d0f1187f8e5ec766))


### Documentation

* add CLAUDE.md with project constitution and conventions ([c51f9fe](https://github.com/mpecan/cargo-lint-extra/commit/c51f9fe357d178ce6d193138616149ee561e83d3))

## [0.1.1](https://github.com/mpecan/cargo-lint-extra/compare/cargo-lint-extra-v0.1.0...cargo-lint-extra-v0.1.1) (2026-03-06)


### Features

* add inline-comments rule ([878f24c](https://github.com/mpecan/cargo-lint-extra/commit/878f24c747f6646c9518c60235a8ad439fb4526c))
* initial implementation of cargo-lint-extra ([e173088](https://github.com/mpecan/cargo-lint-extra/commit/e173088b66bd553adb72699029a382db06f7d144))


### Documentation

* add LICENSE, README, and CONTRIBUTING guide ([317ffb7](https://github.com/mpecan/cargo-lint-extra/commit/317ffb771edfe8af7b0b770b8134418d09de4520))


### Code Refactoring

* always exclude target/ directory from linting ([9ac7bbb](https://github.com/mpecan/cargo-lint-extra/commit/9ac7bbb02e5df50d29977da006cf0fcc5630ee84))
